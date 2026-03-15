//! LiDAR point cloud buffer handling with zero-copy support
//!
//! This module provides efficient zero-copy handling of LiDAR point cloud data.
//! Point clouds are stored in interleaved format (x, y, z, intensity, etc.)
//! and can be directly shared with ROS2 messages or converted to tensors.

use super::{BorrowedBuffer, BufferError, ZeroCopyBuffer};
use burn::tensor::Tensor;

/// A single 3D point with optional intensity and additional fields.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Point {
    /// X coordinate (forward direction for typical LiDAR)
    pub x: f32,
    /// Y coordinate (left/right direction)
    pub y: f32,
    /// Z coordinate (up/down direction)
    pub z: f32,
    /// Intensity/reflectance value (0-255 or 0-1)
    pub intensity: Option<f32>,
    /// Ring/laser number (for multi-beam LiDAR)
    pub ring: Option<u8>,
    /// Timestamp offset from start of scan
    pub timestamp: Option<f32>,
}

impl Point {
    /// Create a new basic point without intensity.
    pub fn new(x: f32, y: f32, z: f32) -> Self {
        Self {
            x,
            y,
            z,
            intensity: None,
            ring: None,
            timestamp: None,
        }
    }

    /// Create a full point with all fields.
    pub fn full(
        x: f32,
        y: f32,
        z: f32,
        intensity: f32,
        ring: u8,
        timestamp: f32,
    ) -> Self {
        Self {
            x,
            y,
            z,
            intensity: Some(intensity),
            ring: Some(ring),
            timestamp: Some(timestamp),
        }
    }

    /// Get the Euclidean distance from origin.
    pub fn distance(&self) -> f32 {
        (self.x * self.x + self.y * self.y + self.z * self.z).sqrt()
    }

    /// Get the azimuth angle (horizontal angle) in radians.
    pub fn azimuth(&self) -> f32 {
        self.y.atan2(self.x)
    }

    /// Get the elevation angle (vertical angle) in radians.
    pub fn elevation(&self) -> f32 {
        self.z.atan2((self.x * self.x + self.y * self.y).sqrt())
    }
}

/// Point cloud buffer with zero-copy operations.
///
/// `LidarBuffer` stores point cloud data in a structured format that can be
/// efficiently shared between ROS2 messages and inference pipelines.
pub struct LidarBuffer {
    /// Raw buffer containing point data in interleaved format
    buffer: ZeroCopyBuffer<u8>,
    /// Number of points in the cloud
    num_points: usize,
    /// Point format specification
    point_format: PointFormat,
}

/// Specification of the point data layout.
#[derive(Debug, Clone, Copy)]
pub struct PointFormat {
    /// Stride per point in bytes
    pub stride: usize,
    /// Offsets (in bytes) for each field
    pub x_offset: usize,
    pub y_offset: usize,
    pub z_offset: usize,
    pub intensity_offset: Option<usize>,
    pub ring_offset: Option<usize>,
    pub timestamp_offset: Option<usize>,
}

impl PointFormat {
    /// Standard 12-byte format: x, y, z as f32 (4 bytes each)
    pub fn xyz_float() -> Self {
        Self {
            stride: 12,
            x_offset: 0,
            y_offset: 4,
            z_offset: 8,
            intensity_offset: None,
            ring_offset: None,
            timestamp_offset: None,
        }
    }

    /// 16-byte format: x, y, z, intensity all as f32
    pub fn xyz_intensity_float() -> Self {
        Self {
            stride: 16,
            x_offset: 0,
            y_offset: 4,
            z_offset: 8,
            intensity_offset: Some(12),
            ring_offset: None,
            timestamp_offset: None,
        }
    }

    /// 32-byte format with all fields
    pub fn full_32byte() -> Self {
        Self {
            stride: 32,
            x_offset: 0,
            y_offset: 4,
            z_offset: 8,
            intensity_offset: Some(12),
            ring_offset: Some(16),
            timestamp_offset: Some(20),
        }
    }

    /// Calculate size for N points
    pub fn size_for_points(&self, num_points: usize) -> usize {
        self.stride * num_points
    }

    /// Get the number of channels (fields) in this format.
    pub fn num_channels(&self) -> usize {
        3 + self.intensity_offset.is_some() as usize
            + self.ring_offset.is_some() as usize
            + self.timestamp_offset.is_some() as usize
    }
}

impl LidarBuffer {
    /// Create a new point cloud buffer with the given capacity and format.
    ///
    /// Allocates uninitialized memory for all points.
    pub fn new(num_points: usize, format: PointFormat) -> Self {
        let total_bytes = format.size_for_points(num_points);
        let buffer = ZeroCopyBuffer::new(total_bytes);
        Self {
            buffer,
            num_points: 0, // Start with 0 initialized points
            point_format: format,
        }
    }

    /// Create a point cloud from raw byte data.
    ///
    /// The data should be properly formatted according to `point_format`.
    pub fn from_bytes(data: Vec<u8>, num_points: usize, format: PointFormat) -> Self {
        let expected = format.size_for_points(num_points);
        assert_eq!(
            data.len(),
            expected,
            "Data size {} does not match expected size {} for {} points with format stride {}",
            data.len(),
            expected,
            num_points,
            format.stride
        );
        let buffer = ZeroCopyBuffer::from_vec(data);
        Self {
            buffer,
            num_points,
            point_format: format,
        }
    }

    /// Create from an existing zero-copy buffer.
    pub fn from_buffer(
        buffer: ZeroCopyBuffer<u8>,
        num_points: usize,
        format: PointFormat,
    ) -> Self {
        let expected = format.size_for_points(num_points);
        assert_eq!(
            buffer.total_len(),
            expected,
            "Buffer size {} does not match expected size {}",
            buffer.total_len(),
            expected
        );
        Self {
            buffer,
            num_points,
            point_format: format,
        }
    }

    /// Get a borrowed view of the point cloud as byte slice.
    pub fn bytes(&self) -> BorrowedBuffer<u8> {
        BorrowedBuffer::new(self.buffer.initialized_slice())
    }

    /// Get the number of initialized points.
    pub fn num_points(&self) -> usize {
        self.num_points
    }

    /// Get the point format.
    pub fn format(&self) -> &PointFormat {
        &self.point_format
    }

    /// Get a specific point by index.
    ///
    /// # Panics
    /// Panics if index >= num_points.
    pub fn get_point(&self, index: usize) -> Point {
        assert!(index < self.num_points, "Point index out of bounds");
        let bytes = self.bytes().as_slice();
        let offset = index * self.point_format.stride;

        // SAFETY: We're reading from properly aligned offsets
        let x = unsafe { *(bytes.as_ptr().add(offset + self.point_format.x_offset) as *const f32) };
        let y = unsafe { *(bytes.as_ptr().add(offset + self.point_format.y_offset) as *const f32) };
        let z = unsafe { *(bytes.as_ptr().add(offset + self.point_format.z_offset) as *const f32) };

        let intensity = self.point_format.intensity_offset.map(|off| {
            unsafe { *(bytes.as_ptr().add(offset + off) as *const f32) }
        });

        let ring = self.point_format.ring_offset.map(|off| {
            unsafe { *(bytes.as_ptr().add(offset + off) as *const u8) }
        });

        let timestamp = self.point_format.timestamp_offset.map(|off| {
            unsafe { *(bytes.as_ptr().add(offset + off) as *const f32) }
        });

        Point {
            x,
            y,
            z,
            intensity,
            ring,
            timestamp,
        }
    }

    /// Get an iterator over all points.
    pub fn iter_points(&self) -> impl Iterator<Item = Point> + '_ {
        (0..self.num_points).map(move |i| self.get_point(i))
    }

    /// Add a point to the buffer.
    ///
    /// Returns `Ok(())` on success or `Err(Point)` if buffer is full.
    pub fn push_point(&mut self, point: Point) -> Result<(), Point> {
        if self.num_points >= self.buffer.total_len() / self.point_format.stride {
            return Err(point);
        }

        let offset = self.num_points * self.point_format.stride;
        let bytes = self.buffer.initialized_slice_mut();

        // Write x, y, z
        unsafe {
            let ptr = bytes.as_mut_ptr().add(offset) as *mut f32;
            ptr.write(point.x);
            ptr.add(1).write(point.y);
            ptr.add(2).write(point.z);
        }

        // Write intensity if present
        if let (Some(off), Some(intensity)) = (self.point_format.intensity_offset, point.intensity) {
            unsafe {
                *(bytes.as_mut_ptr().add(offset + off) as *mut f32) = intensity;
            }
        }

        // Write ring if present
        if let (Some(off), Some(ring)) = (self.point_format.ring_offset, point.ring) {
            unsafe {
                *(bytes.as_mut_ptr().add(offset + off) as *mut u8) = ring;
            }
        }

        // Write timestamp if present
        if let (Some(off), Some(timestamp)) =
            (self.point_format.timestamp_offset, point.timestamp)
        {
            unsafe {
                *(bytes.as_mut_ptr().add(offset + off) as *mut f32) = timestamp;
            }
        }

        self.num_points += 1;
        Ok(())
    }

    /// Convert the point cloud to a Burn tensor.
    ///
    /// The tensor shape will be `[num_points, num_features]` where features
    /// could be just xyz, or xyz + intensity, etc., depending on the format.
    pub fn to_tensor<B: burn::backend::Backend>(&self, device: &B::Device) -> Tensor<B> {
        let num_features = self.point_format.num_channels();
        let total_elements = self.num_points * num_features;
        let mut data = Vec::with_capacity(total_elements);

        for point in self.iter_points() {
            data.push(point.x);
            data.push(point.y);
            data.push(point.z);
            if let Some(intensity) = point.intensity {
                data.push(intensity);
            }
            // Note: This doesn't handle ring/timestamp in tensor yet
        }

        Tensor::from_data(
            burn::tensor::Int::from([self.num_points as i64, num_features as i64]),
            data.as_slice(),
            device,
        )
    }

    /// Create a LidarBuffer from a tensor.
    ///
    /// Assumes tensor shape is `[num_points, num_features]` where features
    /// are ordered as x, y, z, (optional: intensity).
    pub fn from_tensor<B: burn::backend::Backend>(
        tensor: &Tensor<B>,
        format: PointFormat,
    ) -> Self
    where
        f32: Into<u8>,
    {
        let shape = tensor.dims();
        assert_eq!(shape.len(), 2, "Point cloud tensor must be 2D [N, F]");
        let num_points = shape[0] as usize;
        let data = tensor.to_data::<f32>();

        let mut buffer = LidarBuffer::new(num_points, format);
        let mut iter = data.iter();

        for i in 0..num_points {
            let x = *iter.next().unwrap();
            let y = *iter.next().unwrap();
            let z = *iter.next().unwrap();
            let intensity = format.intensity_offset.map(|_| *iter.next().unwrap());

            let point = Point {
                x,
                y,
                z,
                intensity,
                ring: None,
                timestamp: None,
            };

            buffer.push_point(point).expect("Buffer overflow");
        }

        buffer
    }

    /// Get the raw pointer to data for FFI/ROS2 zero-copy operations.
    pub fn as_ptr(&self) -> *const u8 {
        self.bytes().as_slice().as_ptr()
    }

    /// Get mutable raw pointer.
    pub fn as_mut_ptr(&mut self) -> *mut u8 {
        self.bytes().as_slice().as_mut_ptr()
    }

    /// Filter points by a predicate (e.g., range filter).
    ///
    /// This creates a new vector of points; the underlying buffer is not modified.
    pub fn filter<P>(&self, predicate: P) -> Vec<Point>
    where
        P: Fn(&Point) -> bool,
    {
        self.iter_points().filter(|p| predicate(p)).collect()
    }

    /// Estimate the maximum range of points in the cloud.
    pub fn max_range(&self) -> f32 {
        self.iter_points()
            .map(|p| p.distance())
            .fold(0.0f32, |a, b| a.max(b))
    }

    /// Get the centroid (average position) of the point cloud.
    pub fn centroid(&self) -> Point {
        if self.num_points == 0 {
            return Point::new(0.0, 0.0, 0.0);
        }

        let mut sum_x = 0.0;
        let mut sum_y = 0.0;
        let mut sum_z = 0.0;

        for point in self.iter_points() {
            sum_x += point.x;
            sum_y += point.y;
            sum_z += point.z;
        }

        let n = self.num_points as f32;
        Point::new(sum_x / n, sum_y / n, sum_z / n)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_point_creation() {
        let p = Point::new(1.0, 2.0, 3.0);
        assert_eq!(p.x, 1.0);
        assert_eq!(p.y, 2.0);
        assert_eq!(p.z, 3.0);
        assert!(p.intensity.is_none());
        assert_eq!(p.distance(), (14.0f32).sqrt());
    }

    #[test]
    fn test_point_azimuth_elevation() {
        let p = Point::new(1.0, 1.0, 1.0);
        assert!((p.azimuth() - std::f32::consts::FRAC_PI_4).abs() < 1e-5);
        assert!((p.elevation() - 0.61548).abs() < 1e-2);
    }

    #[test]
    fn test_point_format_sizes() {
        let format_xyz = PointFormat::xyz_float();
        assert_eq!(format_xyz.size_for_points(10), 120);

        let format_full = PointFormat::full_32byte();
        assert_eq!(format_full.size_for_points(10), 320);
    }

    #[test]
    fn test_lidar_buffer_new() {
        let format = PointFormat::xyz_float();
        let buffer = LidarBuffer::new(100, format);
        assert_eq!(buffer.num_points(), 0);
        assert_eq!(buffer.format(), &format);
    }

    #[test]
    fn test_lidar_buffer_from_bytes() {
        let mut data = Vec::new();
        for i in 0..3 {
            data.extend_from_slice(&[i as f32, (i + 1) as f32, (i + 2) as f32]); // x, y, z
        }
        let format = PointFormat::xyz_float();
        let buffer = LidarBuffer::from_bytes(data, 3, format);
        assert_eq!(buffer.num_points(), 3);

        // Check first point
        let p0 = buffer.get_point(0);
        assert_eq!(p0.x, 0.0);
        assert_eq!(p0.y, 1.0);
        assert_eq!(p0.z, 2.0);
    }

    #[test]
    fn test_lidar_buffer_push_point() {
        let format = PointFormat::xyz_float();
        let mut buffer = LidarBuffer::new(10, format);

        let p = Point::new(1.0, 2.0, 3.0);
        assert!(buffer.push_point(p).is_ok());
        assert_eq!(buffer.num_points(), 1);

        let retrieved = buffer.get_point(0);
        assert_eq!(retrieved.x, 1.0);
        assert_eq!(retrieved.y, 2.0);
        assert_eq!(retrieved.z, 3.0);
    }

    #[test]
    fn test_lidar_buffer_iter_points() {
        let format = PointFormat::xyz_float();
        let mut buffer = LidarBuffer::new(3, format);

        buffer.push_point(Point::new(0.0, 0.0, 0.0)).unwrap();
        buffer.push_point(Point::new(1.0, 1.0, 1.0)).unwrap();
        buffer.push_point(Point::new(2.0, 2.0, 2.0)).unwrap();

        let points: Vec<_> = buffer.iter_points().collect();
        assert_eq!(points.len(), 3);
        assert_eq!(points[2].x, 2.0);
    }

    #[test]
    fn test_lidar_buffer_max_range() {
        let format = PointFormat::xyz_float();
        let mut buffer = LidarBuffer::new(3, format);

        buffer.push_point(Point::new(0.0, 0.0, 0.0)).unwrap();
        buffer.push_point(Point::new(3.0, 4.0, 0.0)).unwrap(); // distance 5
        buffer.push_point(Point::new(0.0, 0.0, 10.0)).unwrap(); // distance 10

        assert!((buffer.max_range() - 10.0).abs() < 1e-5);
    }

    #[test]
    fn test_lidar_buffer_centroid() {
        let format = PointFormat::xyz_float();
        let mut buffer = LidarBuffer::new(3, format);

        buffer.push_point(Point::new(0.0, 0.0, 0.0)).unwrap();
        buffer.push_point(Point::new(2.0, 0.0, 0.0)).unwrap();
        buffer.push_point(Point::new(1.0, 0.0, 0.0)).unwrap();

        let centroid = buffer.centroid();
        assert!((centroid.x - 1.0).abs() < 1e-5);
        assert!((centroid.y - 0.0).abs() < 1e-5);
        assert!((centroid.z - 0.0).abs() < 1e-5);
    }

    #[test]
    fn test_lidar_buffer_filter() {
        let format = PointFormat::xyz_float();
        let mut buffer = LidarBuffer::new(5, format);

        for i in 0..5 {
            buffer.push_point(Point::new(i as f32, 0.0, 0.0)).unwrap();
        }

        let filtered: Vec<_> = buffer.filter(|p| p.x >= 2.0).collect();
        assert_eq!(filtered.len(), 3);
        assert!(filtered.iter().all(|p| p.x >= 2.0));
    }

    #[test]
    #[should_panic]
    fn test_lidar_buffer_get_point_out_of_bounds() {
        let format = PointFormat::xyz_float();
        let buffer = LidarBuffer::new(1, format);
        buffer.get_point(1);
    }
}
