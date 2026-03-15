//! EdgeBot Core - Inference engine and model operations
//!
//! This crate provides the core AI inference functionality using the Burn framework.
//! It handles model loading, tensor operations, and zero-copy memory interfaces
//! for robotics applications.

pub mod inference;
pub mod memory;
pub mod optimizer;
pub mod task;

/// Core version
pub const VERSION: &str = env!("CARGO_PKG_VERSION");
