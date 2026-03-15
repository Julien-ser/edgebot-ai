# EdgeBot AI

[![CI](https://github.com/edgebot-ai/edgebot-ai/workflows/CI%20(Rust)/badge.svg)](https://github.com/edgebot-ai/edgebot-ai/actions)
[![Rust](https://img.shields.io/badge/rust-1.70%2B-orange.svg)](https://www.rust-lang.org)

A Rust-based platform for deploying lightweight AI models on robots and IoT devices. EdgeBot AI provides zero-copy memory safety, WebAssembly compilation, and seamless ROS2 integration for edge inference.

## Mission

Build toolkits for deploying AI models on robots and IoT devices with:
- **Zero-copy memory safety** using Rust's ownership model
- **Cross-platform support** (x86_64, ARM, WebAssembly)
- **ROS2 integration** for robotics ecosystems
- **Burn framework** for efficient inference
- **Simulation-ready** with Webots integration

## Project Structure

This is a Cargo workspace with multiple crates:

| Crate | Purpose | Status |
|-------|---------|--------|
| `edgebot-core` | Core inference engine with Burn backend | 📦 Planning |
| `edgebot-sim` | Simulation environment (Webots integration) | 📦 Planning |
| `edgebot-ros2` | ROS2 bridge for robot communication | 📦 Planning |
| `edgebot-wasm` | WebAssembly runtime for browser/IoT | 📦 Planning |

## Prerequisites

- Rust 1.70+ with `rustc`, `cargo`, `rustfmt`, `clippy`
- For WASM target: `rustup target add wasm32-unknown-unknown`
- A `rust-toolchain.toml` is included to ensure consistent toolchain versions.
- For ROS2 integration: `ros2` installation (optional)

## Setup

```bash
# Clone and enter workspace
git clone https://github.com/edgebot-ai/edgebot-ai.git
cd edgebot-ai

# Build all crates
cargo build

# Run tests
cargo test --workspace

# Build for WASM (requires wasm32 target)
cargo build --target wasm32-unknown-unknown --release -p edgebot-wasm
```

## Development

```bash
# Format code
cargo fmt

# Lint
cargo clippy --workspace -- -D warnings

# Build with optimizations
cargo build --release

# Run benchmarks (requires criterion)
cargo bench -p edgebot-core
```

## Current Phase: Phase 1 - Planning & Setup

- [x] Task 1: Workspace architecture and crate structure
- [x] Task 2: CI pipeline and toolchain setup
- [ ] Task 3: Burn framework integration
- [ ] Task 4: Zero-copy memory safety interface

See [TASKS.md](TASKS.md) for complete roadmap.

## Architecture Highlights

### Zero-Copy Memory Safety
Uses `std::mem::MaybeUninit` and `ndarray` for sharing sensor data (camera images, LiDAR) without copies between ROS2 messages and inference pipelines.

### Burn Integration
Supports multiple backends (Autocast, Tch) and model formats (ONNX, Burn `.bin`). Enables efficient quantization and pruning for edge deployment.

### ROS2 Bridge
Leverages `rclrs` for type-safe ROS2 communication with zero-copy message passing patterns.

### WebAssembly Target
Compiles to `wasm32-unknown-unknown` for browser simulation and `wasm32-wasi` for IoT execution.

## Inference Engine Example

```rust
use edgebot_core::inference::InferenceEngine;
use burn::backend::tch::TchBackend;

fn main() {
    // Choose device (CPU or GPU)
    let device = TchBackend::Device::default();

    // Load an ONNX model with known input shape
    let engine = InferenceEngine::<TchBackend>::load_onnx(
        Path::new("model.onnx"),
        &[1, 3, 224, 224], // batch, channels, height, width
        device,
    ).expect("Failed to load ONNX model");

    // Or load a Burn binary model
    // let engine = InferenceEngine::<TchBackend>::load_bin(Path::new("model.bin"), device)?;

    // Create input tensor (example: random image)
    let input = burn::tensor::Tensor::<TchBackend>::random(
        [1, 3, 224, 224],
        burn::tensor::Distribution::Uniform(-1.0, 1.0),
        engine.device(),
    );

    // Run inference
    let output = engine.forward(input).expect("Inference failed");
    println!("Output shape: {:?}", output.dims());
}
```

You can also use the `Autocast` backend for mixed precision:

```rust
use burn::backend::autocast::Autocast;
use burn::backend::tch::TchBackend;
type Backend = Autocast<TchBackend>;
let device = Backend::Device::default();
let engine = InferenceEngine::<Backend>::load_onnx(...)?;
```

## License

MIT OR Apache-2.0. See [LICENSE](LICENSE) for details.

## Contributing

We welcome contributions! See [CONTRIBUTING.md](CONTRIBUTING.md) for guidelines.

---

**Status:** Early development (Phase 1). API subject to change.
