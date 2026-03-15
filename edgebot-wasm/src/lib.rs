//! EdgeBot WebAssembly - WASM runtime for browser and IoT
//!
//! This crate provides WebAssembly compilation support for EdgeBot AI,
//! enabling deployment in browser environments and WASI-compatible IoT devices.
//! Uses wasm-bindgen for JavaScript interop.

pub mod runtime;

/// WASM runtime version
pub const VERSION: &str = env!("CARGO_PKG_VERSION");
