pub mod compile;
pub mod deploy;
pub mod optimize;
pub mod simulate;

// Re-export main types for easier testing
pub use compile::{detect_local_hardware, get_target_for_hardware, HardwarePlatform, TargetTriple};
