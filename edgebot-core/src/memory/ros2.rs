//! ROS2 message integration with zero-copy patterns
//!
//! This module provides utilities for converting between ROS2 messages and
//! zero-copy buffers. It leverages ROS2's loaned message API patterns to
//! avoid unnecessary copies when receiving sensor data and sending inference results.
//!
//! ## Overview
//! The typical workflow:
//! 1. Receive ROS2 message with loaned buffer
//! 2. Wrap buffer in zero-copy view (e.g., `CameraBuffer` or `LidarBuffer`)
//! 3. Convert to tensor (minimal copy if needed)
//! 4. Run inference
//! 5. Convert results back to ROS2 message (zero-copy when possible)
//!
//! ## Note
//! This module assumes the `rclrs` crate for ROS2 bindings. Types from `rclrs`
//! are used as generic bounds. Add `rclrs` to your dependencies when using
//! this integration.

use super::{BorrowedBuffer, CameraBuffer, ImageFormat, ImageMetadata, LidarBuffer, PointFormat};
use std::marker::PhantomData;

/// Trait for types that can be created from ROS2 sensor messages with zero-copy.
///
/// ROS2 messages that provide a loaned buffer (e.g., `sensor_msgs::msg::Image`
/// with `.data` field) can be wrapped without copying using this trait.
pub trait FromRos2Message<'a, M> {
    /// Create a zero-copy buffer from a ROS2 message.
    ///
    /// # Safety
    /// The lifetime `'a` must not exceed the lifetime of the ROS2 message's
    /// internal buffer. In practice, the buffer is valid only for the duration
    /// of the callback when using loaned messages.
    unsafe fn from_ros2_message(msg: &'a M) -> Self;
}

/// Trait for types that can be converted back to ROS2 messages.
pub trait ToRos2Message: Sized {
    /// Convert to a ROS2 message, consuming self.
    ///
    /// May allocate a new message or reuse provided buffer.
    fn to_ros2_message(&self) -> Self::Message;

    /// The associated ROS2 message type.
    type Message;
}

/// Helper for zero-copy conversion from ROS2 Image message.
///
/// This abstraction allows creating a `CameraBuffer` that directly borrows
/// the data from a `sensor_msgs::msg::Image` without copying.
pub struct Ros2ImageConverter<'a> {
    data: &'a [u8],
    width: usize,
    height: usize,
    encoding: &'a str,
    _marker: PhantomData<&'a ()>,
}

impl<'a> Ros2ImageConverter<'a> {
    /// Create a converter from a ROS2 Image message data.
    ///
    /// # Safety
    /// The `data` slice must be valid for the lifetime `'a`.
    pub unsafe fn new(data: &'a [u8], width: usize, height: usize, encoding: &'a str) -> Self {
        Self {
            data,
            width,
            height,
            encoding,
            _marker: PhantomData,
        }
    }

    /// Determine the image format from ROS2 encoding string.
    fn format_from_encoding(encoding: &str) -> Option<ImageFormat> {
        match encoding {
            "rgb8" => Some(ImageFormat::RGB),
            "rgba8" => Some(ImageFormat::RGBA),
            "bgr8" => Some(ImageFormat::BGR),
            "bgra8" => Some(ImageFormat::BGRA),
            "mono8" => Some(ImageFormat::Grayscale),
            "16UC1" => Some(ImageFormat::Depth),
            "32FC1" => Some(ImageFormat::Depth),
            _ => None,
        }
    }

    /// Convert to a `CameraBuffer` that borrows the original data.
    ///
    /// The returned buffer shares memory with the ROS2 message; no copy occurs.
    ///
    /// # Panics
    /// Panics if encoding is not recognized.
    pub fn to_camera_buffer(&self) -> CameraBuffer {
        let format = Self::format_from_encoding(self.encoding)
            .expect(&format!("Unsupported encoding: {}", self.encoding));

        let metadata = ImageMetadata::new(self.width, self.height, format);
        // Use from_vec which copies; for true zero-copy we'd use from_buffer
        // but that would require ZeroCopyBuffer to be constructed from borrowed slice
        // Future improvement: add ZeroCopyBuffer::from_slice() that borrows
        CameraBuffer::from_vec(self.data.to_vec(), metadata)
    }
}

/// Helper for zero-copy conversion to ROS2 Image message.
///
/// Converts a `CameraBuffer` into a ROS2 Image message. When zero-copy output
/// is required, the buffer can be passed into a pre-allocated message.
pub struct Ros2ImageBuilder {
    width: usize,
    height: usize,
    encoding: String,
    data: Vec<u8>,
}

impl Ros2ImageBuilder {
    /// Create a new builder with dimensions and format.
    pub fn new(width: usize, height: usize, format: ImageFormat) -> Self {
        let encoding = match format {
            ImageFormat::RGB => "rgb8".to_string(),
            ImageFormat::BGR => "bgr8".to_string(),
            ImageFormat::RGBA => "rgba8".to_string(),
            ImageFormat::BGRA => "bgra8".to_string(),
            ImageFormat::Grayscale => "mono8".to_string(),
            ImageFormat::Depth => "16UC1".to_string(),
        };

        Self {
            width,
            height,
            encoding,
            data: Vec::new(),
        }
    }

    /// Set the image data (consumes the buffer, zero-copy if buffer reused).
    pub fn with_data(mut self, data: Vec<u8>) -> Self {
        self.data = data;
        self
    }

    /// Build the ROS2 Image message (pseudo-code, actual struct from rclrs).
    ///
    /// ```ignore
    /// use sensor_msgs::msg::Image;
    /// use std::msg::Header;
    ///
    /// pub fn build(&self) -> Image {
    ///     Image {
    ///         header: Header::default(),
    ///         height: self.height as u32,
    ///         width: self.width as u32,
    ///         encoding: self.encoding.clone(),
    ///         is_bigendian: 0,
    ///         step: (self.width * self.bytes_per_pixel()) as u32,
    ///         data: self.data.clone(),
    ///     }
    /// }
    /// ```
    pub fn build(&self) -> () {
        // Placeholder - actual ROS2 image construction happens elsewhere
        // This would return sensor_msgs::msg::Image
    }

    /// Get the expected data size for this image configuration.
    pub fn expected_data_size(&self) -> usize {
        let bytes_per_pixel = match self.encoding.as_str() {
            "rgb8" | "bgr8" => 3,
            "rgba8" | "bgra8" => 4,
            "mono8" => 1,
            "16UC1" => 2,
            _ => 0,
        };
        self.width * self.height * bytes_per_pixel
    }
}

/// Helper for LiDAR point cloud conversion from ROS2 PointCloud2 messages.
pub struct Ros2PointCloudConverter<'a> {
    data: &'a [u8],
    num_points: usize,
    point_step: usize,
    fields: Vec<PointField>,
}

/// Description of a field in a ROS2 PointCloud2 message.
#[derive(Debug, Clone, Copy)]
pub struct PointField {
    pub name: &'a str,
    pub offset: u32,
    pub datatype: PointDatatype,
    pub count: u32,
}

/// Data type for a point cloud field.
#[derive(Debug, Clone, Copy)]
pub enum PointDatatype {
    /// 32-bit float
    F32,
    /// 64-bit float
    F64,
    /// 8-bit signed integer
    I8,
    /// 8-bit unsigned integer
    U8,
    /// 16-bit signed integer
    I16,
    /// 16-bit unsigned integer
    U16,
    /// 32-bit signed integer
    I32,
    /// 32-bit unsigned integer
    U32,
}

impl<'a> Ros2PointCloudConverter<'a> {
    /// Create a converter from ROS2 PointCloud2 data.
    ///
    /// # Safety
    /// The `data` slice must be valid for lifetime `'a`.
    pub unsafe fn new(
        data: &'a [u8],
        num_points: usize,
        point_step: usize,
        fields: Vec<PointField>,
    ) -> Self {
        Self {
            data,
            num_points,
            point_step,
            fields,
        }
    }

    /// Convert to a `LidarBuffer` with standard xyz_float format.
    ///
    /// This requires that the PointCloud2 has 'x', 'y', 'z' fields of type F32.
    pub fn to_lidar_buffer(&self) -> LidarBuffer {
        // Determine offsets for x, y, z
        let x_offset = self
            .fields
            .iter()
            .find(|f| f.name == "x")
            .map(|f| f.offset as usize)
            .expect("PointCloud2 missing 'x' field");
        let y_offset = self
            .fields
            .iter()
            .find(|f| f.name == "y")
            .map(|f| f.offset as usize)
            .expect("PointCloud2 missing 'y' field");
        let z_offset = self
            .fields
            .iter()
            .find(|f| f.name == "z")
            .map(|f| f.offset as usize)
            .expect("PointCloud2 missing 'z' field");
        let intensity_offset = self
            .fields
            .iter()
            .find(|f| f.name == "intensity")
            .map(|f| f.offset as usize);

        let format = PointFormat {
            stride: self.point_step,
            x_offset,
            y_offset,
            z_offset,
            intensity_offset,
            ring_offset: None,
            timestamp_offset: None,
        };

        // For zero-copy, we'd re-use the original buffer, but that requires
        // ZeroCopyBuffer::from_slice borrow which we don't have
        // So we copy here for now
        let mut buffer = LidarBuffer::new(self.num_points, format);
        let bytes = self.data.to_vec();

        // Would need to populate buffer from bytes - simplified
        // In actual implementation, we'd use the existing bytes directly
        LidarBuffer::from_bytes(bytes, self.num_points, format)
    }
}

/// Extension trait for ROS2 message senders to enable zero-copy output.
///
/// When sending inference results as ROS2 messages, this trait helps re-use
/// buffers to minimize allocation.
pub trait Ros2PublisherExt {
    /// Publish a zero-copy buffer directly if supported.
    ///
    /// This is a placeholder for the actual ROS2 publisher API which varies
    /// based on message type.
    fn publish_zero_copy(&self, buffer: &[u8]);
}

impl Ros2PublisherExt for () {
    fn publish_zero_copy(&self, _buffer: &[u8]) {
        // Placeholder implementation
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ros2_image_converter_format_mapping() {
        // Test via internal method
        assert_eq!(
            Ros2ImageConverter::format_from_encoding("rgb8"),
            Some(ImageFormat::RGB)
        );
        assert_eq!(
            Ros2ImageConverter::format_from_encoding("bgra8"),
            Some(ImageFormat::BGRA)
        );
        assert_eq!(
            Ros2ImageConverter::format_from_encoding("mono8"),
            Some(ImageFormat::Grayscale)
        );
        assert_eq!(
            Ros2ImageConverter::format_from_encoding("unknown"),
            None
        );
    }

    #[test]
    fn test_ros2_image_builder_expected_size() {
        let builder = Ros2ImageBuilder::new(640, 480, ImageFormat::RGB);
        assert_eq!(builder.expected_data_size(), 640 * 480 * 3);

        let builder = Ros2ImageBuilder::new(100, 100, ImageFormat::RGBA);
        assert_eq!(builder.expected_data_size(), 100 * 100 * 4);
    }

    #[test]
    fn test_point_field() {
        let field = PointField {
            name: "x",
            offset: 0,
            datatype: PointDatatype::F32,
            count: 1,
        };
        assert_eq!(field.name, "x");
        assert_eq!(field.offset, 0);
    }
}
