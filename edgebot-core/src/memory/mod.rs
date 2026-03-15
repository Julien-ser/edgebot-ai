//! Zero-copy memory safety interface for sensor data sharing
//!
//! This module provides safe abstractions for sharing sensor data (camera images,
//! LiDAR point clouds) between ROS2 messages and inference pipelines without
//! unnecessary memory copies. It uses Rust's ownership model and `MaybeUninit`
//! to ensure memory safety while achieving maximum performance.
//!
//! ## Key Features
//! - Zero-copy buffer sharing using `MaybeUninit`
//! - Safe conversion between ROS2 message buffers and internal representations
//! - Camera image buffers (RGB, depth, etc.)
//! - LiDAR point cloud buffers
//! - Integration with Burn tensors for direct inference
//!
//! ## Usage Example
//!
//! ```rust
//! use edgebot_core::memory::{ZeroCopyBuffer, CameraBuffer, SensorBuffer};
//! use edgebot_core::memory::tensor::IntoTensor;
//! use burn::backend::tch::TchBackend;
//!
//! // Create a zero-copy buffer from raw sensor data
//! let raw_data: Vec<u8> = vec![0u8; 640 * 480 * 3]; // RGB image
//! let mut buffer = CameraBuffer::from_vec(raw_data, 640, 480, 3);
//!
//! // Convert to Burn tensor without copying
//! let device = burn::backend::tch::TchBackend::Device::default();
//! let tensor = buffer.to_tensor(&device);
//!
//! // Use tensor in inference engine
//! // let output = inference_engine.forward(tensor);
//! ```
//!
//! ## Safety
//! All zero-copy operations are safe Rust abstractions. The `MaybeUninit` type
//! ensures that uninitialized memory is never read, and all buffers maintain
//! proper alignment and lifetime guarantees.

pub mod buffer;
pub mod camera;
pub mod lidar;
pub mod tensor;
pub mod ros2;

// Re-export key types for convenient access
pub use buffer::{ZeroCopyBuffer, BufferError, BorrowedBuffer};
pub use camera::{CameraBuffer, ImageFormat, ImageMetadata};
pub use lidar::{LidarBuffer, PointCloud, Point};
pub use tensor::{IntoTensor, TensorConverter};
pub use ros2::{Ros2Converter, Ros2Message};
