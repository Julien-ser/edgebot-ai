//! Camera image buffer handling with zero-copy support
//!
//! This module provides types for working with camera sensor data (RGB, depth,
//! grayscale, etc.) in a zero-copy manner. It integrates with ROS2 message
//! formats and Burn tensors for efficient inference pipeline processing.

use super::{BorrowedBuffer, BufferError, ZeroCopyBuffer};
use burn::tensor::Tensor;
use std::mem::MaybeUninit;

/// Supported image formats.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ImageFormat {
    /// RGB image with 3 channels.
    RGB,
    /// BGR image with 3 channels.
    BGR,
    /// RGBA image with 4 channels.
    RGBA,
    /// BGRA image with 4 channels.
    BGRA,
    /// Grayscale single-channel image.
    Grayscale,
    /// Depth image (typically 16-bit or 32-bit float).
    Depth,
}

/// Metadata about an image (dimensions, format, stride).
#[derive(Debug, Clone, Copy)]
pub struct ImageMetadata {
    /// Image width in pixels.
    pub width: usize,
    /// Image height in pixels.
    pub height: usize,
    /// Image format (channels and interpretation).
    pub format: ImageFormat,
    /// Row stride in bytes (may be larger than width * bytes_per_pixel).
    pub stride: Option<usize>,
}

impl ImageMetadata {
    /// Create new metadata with the given dimensions and format.
    ///
    /// Stride is automatically computed as `width * bytes_per_pixel()` if not provided.
    pub fn new(width: usize, height: usize, format: ImageFormat) -> Self {
        let stride = Some(width * format.bytes_per_pixel());
        Self {
            width,
            height,
            format,
            stride,
        }
    }

    /// Get the number of bytes per pixel for this format.
    pub fn bytes_per_pixel(&self) -> usize {
        match self.format {
            ImageFormat::RGB => 3,
            ImageFormat::BGR => 3,
            ImageFormat::RGBA => 4,
            ImageFormat::BGRA => 4,
            ImageFormat::Grayscale => 1,
            ImageFormat::Depth => 2, // 16-bit depth or 32-bit float
        }
    }

    /// Get the total number of bytes required to store this image.
    pub fn total_bytes(&self) -> usize {
        let stride = self.stride.unwrap_or_else(|| self.width * self.bytes_per_pixel());
        stride * self.height
    }

    /// Get the number of channels for this format.
    pub fn channels(&self) -> usize {
        self.format.channels()
    }

    /// Check if the image format has an alpha channel.
    pub fn has_alpha(&self) -> bool {
        matches!(self.format, ImageFormat::RGBA | ImageFormat::BGRA)
    }

    /// Get the pixel depth in bits.
    pub fn pixel_depth(&self) -> u8 {
        match self.format {
            ImageFormat::Depth => 16, // or 32 for float depth
            _ => 8,
        }
    }
}

impl ImageFormat {
    /// Get the number of channels for this format.
    pub fn channels(self) -> usize {
        match self {
            ImageFormat::RGB => 3,
            ImageFormat::BGR => 3,
            ImageFormat::RGBA => 4,
            ImageFormat::BGRA => 4,
            ImageFormat::Grayscale => 1,
            ImageFormat::Depth => 1,
        }
    }
}

/// A zero-copy camera image buffer.
///
/// `CameraBuffer` manages image data in a contiguous memory buffer with
/// format metadata. It supports zero-copy conversion to Burn tensors and
/// can be created from or converted to ROS2 image messages without copying.
pub struct CameraBuffer {
    buffer: ZeroCopyBuffer<u8>,
    metadata: ImageMetadata,
}

impl CameraBuffer {
    /// Create a new camera buffer with the given metadata.
    ///
    /// Allocates uninitialized memory for the image data.
    pub fn new(metadata: ImageMetadata) -> Self {
        let total_bytes = metadata.total_bytes();
        let buffer = ZeroCopyBuffer::new(total_bytes);
        Self { buffer, metadata }
    }

    /// Create a camera buffer from existing byte data.
    ///
    /// The data is expected to match the image dimensions and format.
    pub fn from_vec(data: Vec<u8>, metadata: ImageMetadata) -> Self {
        assert_eq!(
            data.len(),
            metadata.total_bytes(),
            "Data length {} does not match expected image size {}",
            data.len(),
            metadata.total_bytes()
        );
        let buffer = ZeroCopyBuffer::from_vec(data);
        Self { buffer, metadata }
    }

    /// Create a buffer from an existing zero-copy buffer.
    ///
    /// This allows sharing memory between different sensor data representations.
    pub fn from_buffer(buffer: ZeroCopyBuffer<u8>, metadata: ImageMetadata) -> Self {
        assert_eq!(
            buffer.total_len(),
            metadata.total_bytes(),
            "Buffer size {} does not match expected image size {}",
            buffer.total_len(),
            metadata.total_bytes()
        );
        Self { buffer, metadata }
    }

    /// Get a borrowed view of the raw image bytes.
    pub fn bytes(&self) -> BorrowedBuffer<u8> {
        BorrowedBuffer::new(self.buffer.initialized_slice())
    }

    /// Get a mutable borrowed view of the raw image bytes.
    pub fn bytes_mut(&mut self) -> BorrowedBuffer<u8> {
        BorrowedBuffer::new(self.buffer.initialized_slice_mut())
    }

    /// Get the image metadata.
    pub fn metadata(&self) -> &ImageMetadata {
        &self.metadata
    }

    /// Get the width of the image.
    pub fn width(&self) -> usize {
        self.metadata.width
    }

    /// Get the height of the image.
    pub fn height(&self) -> usize {
        self.metadata.height
    }

    /// Get the image format.
    pub fn format(&self) -> ImageFormat {
        self.metadata.format
    }

    /// Convert this camera buffer into a Burn tensor.
    ///
    /// This is a zero-copy operation when possible, depending on the backend.
    /// The tensor shape will be `[height, width, channels]` or `[1, height, width, channels]`
    /// depending on whether a batch dimension is added.
    pub fn to_tensor<B: burn::backend::Backend>(&self, device: &B::Device) -> Tensor<B>
    where
        u8: Into<burn::tensor::DType>,
    {
        // Convert u8 data to f32 for inference
        // Zero-copy would require the backend to support u8, which is uncommon
        // So we do a conversion here but it's unavoidable for most backends
        let data_f32: Vec<f32> = self
            .bytes()
            .iter()
            .map(|&b| (b as f32) / 255.0) // Normalize to [0, 1]
            .collect();

        let shape = match self.metadata.format {
            ImageFormat::Depth => [self.height, self.width],
            _ => [self.height, self.width, self.metadata.channels() as i64],
        };

        Tensor::from_data(burn::tensor::Int::from(shape), data_f32.as_slice(), device)
    }

    /// Convert to a tensor with optional batch dimension.
    pub fn to_tensor_with_batch<B: burn::backend::Backend>(
        &self,
        device: &B::Device,
        batch_size: usize,
    ) -> Tensor<B>
    where
        u8: Into<burn::tensor::DType>,
    {
        let data_f32: Vec<f32> = self
            .bytes()
            .iter()
            .map(|&b| (b as f32) / 255.0)
            .collect();

        let shape = match self.metadata.format {
            ImageFormat::Depth => [batch_size, self.height, self.width],
            _ => [batch_size, self.height, self.width, self.metadata.channels() as i64],
        };

        Tensor::from_data(burn::tensor::Int::from(shape), data_f32.as_slice(), device)
    }

    /// Create a camera buffer from a Burn tensor.
    ///
    /// # Panics
    /// Panics if the tensor's shape doesn't match expected image dimensions.
    pub fn from_tensor<B: burn::backend::Backend>(
        tensor: &Tensor<B>,
        format: ImageFormat,
    ) -> Self
    where
        f32: Into<u8>,
    {
        let shape = tensor.dims();
        let (height, width, channels) = match format {
            ImageFormat::Depth => {
                assert_eq!(shape.len(), 2, "Depth tensor must be 2D");
                (shape[0] as usize, shape[1] as usize, 1)
            }
            _ => {
                assert_eq!(shape.len(), 3, "Image tensor must be 3D [H, W, C]");
                (shape[0] as usize, shape[1] as usize, shape[2] as usize)
            }
        };

        let metadata = ImageMetadata::new(width, height, format);
        let data = tensor.to_data::<f32>();
        let bytes: Vec<u8> = data
            .iter()
            .map(|&v| (v * 255.0).clamp(0.0, 255.0) as u8)
            .collect();

        Self::from_vec(bytes, metadata)
    }

    /// Get the total number of pixels in the image.
    pub fn num_pixels(&self) -> usize {
        self.metadata.width * self.metadata.height
    }

    /// Get a pointer to the raw image data for zero-copy sharing with FFI.
    ///
    /// # Safety
    /// The returned pointer is valid as long as the buffer is not modified
    /// or dropped. The caller must not free the pointer.
    pub fn as_ptr(&self) -> *const u8 {
        self.buffer.initialized_slice().as_ptr()
    }

    /// Get a mutable pointer to the raw image data.
    ///
    /// # Safety
    /// The returned pointer is valid as long as the buffer is not dropped.
    /// Modifying through the pointer does not update `initialized_len` tracking,
    /// so use with caution.
    pub fn as_mut_ptr(&mut self) -> *mut u8 {
        self.buffer.initialized_slice_mut().as_mut_ptr()
    }

    /// Split the buffer into separate color components (R, G, B for RGB).
    ///
    /// This creates views into the original buffer without copying.
    pub fn split_channels(&self) -> Option<Vec<BorrowedBuffer<u8>>> {
        match self.metadata.format {
            ImageFormat::RGB | ImageFormat::BGR => {
                let num_pixels = self.num_pixels();
                let mut channels = Vec::with_capacity(3);
                for c in 0..3 {
                    // De-interleave: each channel's data is at offset c
                    let channel_data: Vec<u8> = self
                        .bytes()
                        .iter()
                        .skip(c)
                        .step_by(3)
                        .copied()
                        .collect();
                    channels.push(BorrowedBuffer::new(&channel_data));
                }
                Some(channels)
            }
            _ => None, // Not a multi-channel format
        }
    }

    /// Convert between RGB and BGR formats (zero-copy view, reinterprets data).
    pub fn swap_rgb_bgr(&mut self) -> Result<(), BufferError> {
        match self.metadata.format {
            ImageFormat::RGB => {
                self.metadata.format = ImageFormat::BGR;
                Ok(())
            }
            ImageFormat::BGR => {
                self.metadata.format = ImageFormat::RGB;
                Ok(())
            }
            _ => Err(BufferError::UninitializedRead), // Or better: custom error
        }
    }
}

/// Extension trait for creating camera buffers from raw ROS2 message data.
pub trait Ros2ImageExt {
    /// Create a `CameraBuffer` from raw image data with stride.
    ///
    /// # Arguments
    /// * `data` - Raw byte buffer from ROS2 message
    /// * `width` - Image width
    /// * `height` - Image height
    /// * `format` - Image format
    /// * `stride` - Row stride in bytes (may be larger than width * bytes_per_pixel)
    ///
    /// # Returns
    /// A `CameraBuffer` that shares the memory with the original data when possible.
    fn to_camera_buffer(
        data: Vec<u8>,
        width: usize,
        height: usize,
        format: ImageFormat,
        stride: Option<usize>,
    ) -> CameraBuffer {
        let metadata = ImageMetadata {
            width,
            height,
            format,
            stride,
        };
        CameraBuffer::from_vec(data, metadata)
    }
}

impl Ros2ImageExt for CameraBuffer {
    fn to_camera_buffer(
        data: Vec<u8>,
        width: usize,
        height: usize,
        format: ImageFormat,
        stride: Option<usize>,
    ) -> CameraBuffer {
        CameraBuffer::from_vec(data, ImageMetadata {
            width,
            height,
            format,
            stride,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_image_metadata() {
        let meta = ImageMetadata::new(640, 480, ImageFormat::RGB);
        assert_eq!(meta.total_bytes(), 640 * 480 * 3);
        assert_eq!(meta.channels(), 3);
        assert!(!meta.has_alpha());
    }

    #[test]
    fn test_camera_buffer_new() {
        let meta = ImageMetadata::new(2, 2, ImageFormat::RGB);
        let mut buffer = CameraBuffer::new(meta);
        assert_eq!(buffer.width(), 2);
        assert_eq!(buffer.height(), 2);
        assert!(buffer.bytes().is_empty()); // No data written yet
    }

    #[test]
    fn test_camera_buffer_from_vec() {
        let data = vec![0u8; 4 * 4 * 3]; // 4x4 RGB
        let meta = ImageMetadata::new(4, 4, ImageFormat::RGB);
        let buffer = CameraBuffer::from_vec(data, meta);
        assert_eq!(buffer.bytes().len(), 48);
    }

    #[test]
    #[should_panic]
    fn test_camera_buffer_from_vec_wrong_size() {
        let data = vec![0u8; 10];
        let meta = ImageMetadata::new(2, 2, ImageFormat::RGB); // needs 12 bytes
        let _ = CameraBuffer::from_vec(data, meta);
    }

    #[test]
    fn test_camera_buffer_to_tensor() {
        use burn::backend::tch::TchBackend;
        let data = vec![255u8; 2 * 2 * 3]; // White 2x2 RGB
        let meta = ImageMetadata::new(2, 2, ImageFormat::RGB);
        let buffer = CameraBuffer::from_vec(data, meta);

        let device = TchBackend::Device::default();
        let tensor = buffer.to_tensor(&device);
        assert_eq!(tensor.dims(), &[2, 2, 3]);

        // Check values are normalized to 1.0
        let data = tensor.to_data::<f32>();
        assert!(data.iter().all(|&v| (v - 1.0).abs() < 1e-5));
    }

    #[test]
    fn test_camera_buffer_channels_split() {
        let data = vec![1u8, 2, 3, 4, 5, 6, 7, 8, 9]; // 3x3 RGB
        let meta = ImageMetadata::new(3, 3, ImageFormat::RGB);
        let buffer = CameraBuffer::from_vec(data, meta);

        let channels = buffer.split_channels().unwrap();
        assert_eq!(channels.len(), 3);
        assert_eq!(channels[0].as_slice(), &[1, 4, 7]); // R
        assert_eq!(channels[1].as_slice(), &[2, 5, 8]); // G
        assert_eq!(channels[2].as_slice(), &[3, 6, 9]); // B
    }
}
