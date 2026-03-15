//! EdgeBot ROS2 - Bridge for Robot Operating System 2
//!
//! This crate provides ROS2 integration using rclrs, enabling seamless
//! communication between EdgeBot AI and ROS2-based robots.
//! Features zero-copy message passing for performance.

pub mod bridge;

/// ROS2 integration version
pub const VERSION: &str = env!("CARGO_PKG_VERSION");
