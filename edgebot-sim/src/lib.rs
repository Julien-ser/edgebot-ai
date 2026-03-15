//! EdgeBot Simulation - Virtual environment for testing
//!
//! This crate provides simulation capabilities including Webots integration
//! and headless simulation modes for testing AI models on virtual robots.

pub mod webots;

/// Simulation version
pub const VERSION: &str = env!("CARGO_PKG_VERSION");
