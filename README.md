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
| `edgebot-core` | Core inference engine + memory safety + optimizer + tasks | 📦 Phase 2 (Optimizer done) |
| `edgebot-sim` | Simulation environment (Webots integration) | 📦 Phase 3 |
| `edgebot-ros2` | ROS2 bridge for robot communication | 📦 Phase 2 |
| `edgebot-wasm` | WebAssembly runtime for browser/IoT | 📦 Phase 2 |

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

## Current Status

**Phase 2: Core SDK Development** - In progress

- [x] Phase 2 Task 1: Model optimizer (quantization, pruning, layer fusion)
- [x] Phase 2 Task 2: ROS2 bridge
- [ ] Phase 2 Task 3: WebAssembly runtime
- [ ] Phase 2 Task 4: ModelTask trait abstraction

See [TASKS.md](TASKS.md) for complete roadmap.

## Architecture Highlights

### Zero-Copy Memory Safety

The `edgebot-core/memory` module provides safe abstractions for sharing sensor data without memory copies between ROS2 messages and inference pipelines:

```rust
use edgebot_core::memory::{CameraBuffer, ImageFormat, ImageMetadata, ZeroCopyBuffer};

// Create camera buffer from raw sensor data (zero-copy)
let metadata = ImageMetadata::new(640, 480, ImageFormat::RGB);
let mut buffer = CameraBuffer::new(metadata);

// Fill buffer with ROS2 image data
buffer.bytes_mut().copy_from_slice(&ros2_image_data);

// Convert to Burn tensor (minimal copy if needed)
let device = burn::backend::tch::TchBackend::Device::default();
let tensor = buffer.to_tensor(&device);

// Run inference
// let output = inference_engine.forward(tensor);
```

**Key Types:**

- `ZeroCopyBuffer<T>`: Safe buffer using `MaybeUninit` for uninitialized memory management
- `CameraBuffer`: Zero-copy image buffer supporting RGB, BGR, RGBA, grayscale, depth
- `LidarBuffer`: Point cloud buffer for LiDAR data (xyz, intensity, ring, timestamp)
- `BorrowedBuffer<'a, T>`: Temporary view into initialized buffer data
- ROS2 integration helpers (`Ros2ImageConverter`, `Ros2PointCloudConverter`)

### Burn Integration

The inference engine (`edgebot-core::inference`) supports multiple backends (Tch, Autocast) and model formats (ONNX, Burn binary):

```rust
use edgebot_core::inference::InferenceEngine;
use burn::backend::tch::TchBackend;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let device = TchBackend::Device::default();

    // Load ONNX model
    let engine = InferenceEngine::<TchBackend>::load_onnx(
        Path::new("model.onnx"),
        &[1, 3, 224, 224], // input shape
        device,
    )?;

    // Or load Burn binary
    // let engine = InferenceEngine::<TchBackend>::load_bin(Path::new("model.bin"), device)?;

    // Create input tensor (example)
    let input = burn::tensor::Tensor::<TchBackend>::random(
        [1, 3, 224, 224],
        burn::tensor::Distribution::Uniform(-1.0, 1.0),
        engine.device(),
    );

    // Run inference
    let output = engine.forward(input)?;
    println!("Output shape: {:?}", output.dims());

    Ok(())
}
```

### Model Optimizer

Optimize models for edge deployment with quantization, pruning, and layer fusion using the `edgebot-optimize` CLI:

```bash
# Build the optimizer
cargo build -p edgebot-core --bin edgebot-optimize --release

# Optimize a model with int8 quantization
./target/release/edgebot-optimize \
  --input model.onnx \
  --output model.ebmodel \
  --quantize int8 \
  --fuse-layers

# With pruning (magnitude-based, 50% threshold)
./target/release/edgebot-optimize \
  --input model.onnx \
  --output model.ebmodel \
  --quantize fp16 \
  --prune magnitude \
  --pruning-threshold 0.5 \
  --device cpu
```

**Output:** `.ebmodel` bundle containing optimized model + metadata (JSON with embedded binary).

**CLI Options:**
- `--quantize`: none/int8/fp16 (default: none)
- `--prune`: none/magnitude/structured
- `--pruning-threshold`: fraction of weights to prune (0.0-1.0)
- `--fuse-layers`: enable layer fusion (Conv+ReLU, etc.)
- `--device`: target device (cpu/cuda)

**Optimization Stats:** The CLI prints size reduction, speedup estimates, and saves detailed stats in the .ebmodel bundle.

### ROS2 Bridge

The `edgebot-ros2` crate provides ROS2 integration using the `rclrs` crate. It enables publishing and subscribing to ROS2 topics with zero-copy message passing for sensor data (camera images, LiDAR). A YOLO inference example node is included, demonstrating how to subscribe to camera images, run inference with edgebot-core, and publish detection results.

#### Running the YOLO Example

```bash
# Build the example node
cargo build -p edgebot-ros2 --bin yolo_node --release

# Run (requires a ROS2 environment and a camera topic publishing images)
cargo run -p edgebot-ros2 --bin yolo_node -- --ros-args -p camera_topic:=/camera/color/image_raw
```

Note: The example currently uses a hardcoded model path `models/yolov8.onnx`. Place a suitable ONNX model in that location or modify the source code to point to your model.

### WebAssembly Target

The `edgebot-wasm` crate compiles to `wasm32-unknown-unknown` for browser-based simulation and `wasm32-wasi` for edge IoT devices. Zero-copy memory interfaces ensure efficient data passing between JavaScript and Rust.

## License

MIT OR Apache-2.0. See [LICENSE](LICENSE) for details.

## Contributing

We welcome contributions! See [CONTRIBUTING.md](CONTRIBUTING.md) for guidelines.

---

**Status:** Early development (Phase 1). API subject to change.
