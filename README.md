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
| `edgebot-core` | Core inference engine + memory safety + optimizer + tasks | ✅ Phase 2 (Core SDK completed) |
| `edgebot-sim` | Simulation environment with Webots integration (headless testing) | ✅ Phase 3 (Webots integration completed) |
| `edgebot-ros2` | ROS2 bridge for robot communication | 📦 Phase 2 |
| `edgebot-wasm` | WebAssembly runtime for browser/IoT | ✅ Phase 2 (Runtime done) |
| `edgebot-sim-server` | Cloud simulation service with Actix Web API | ✅ Phase 3 |
| `edgebot-cli` | Command-line interface for compilation, deployment, simulation, and optimization | 🆕 Phase 4 (In progress) |

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

# Generate JSON report for pro tier optimization
cargo bench -p edgebot-core -- --output-format json > benchmark_results/raw.json
# The pro tier optimization report is auto-generated at benchmark_results/inference_pro_report.json
```

## Current Status

**Phase 3: Simulation & Compilation** - In progress

- [x] Phase 3 Task 1: Webots simulation integration ✅ COMPLETED
- [x] Phase 3 Task 2: Cloud simulation service ✅ COMPLETED
- [x] Phase 3 Task 3: ARM cross-compilation toolchain ✅ COMPLETED
- [x] Phase 3 Task 4: Profiling & benchmarking suite (criterion) ✅ COMPLETED

**Phase 4: Deployment & Monetization** - Starting

- [ ] Phase 4 Task 1: Full EdgeBot CLI (deploy, simulate, optimize commands)
- [ ] Phase 4 Task 2: License verification system
- [ ] Phase 4 Task 3: Dashboard frontend
- [ ] Phase 4 Task 4: Comprehensive documentation

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

### Webots Simulation

The `edgebot-sim` crate provides integration with the Webots robotics simulator for testing AI models on virtual robots in a controlled environment. It offers a safe, Python-like Rust API and supports headless (no-GUI) mode for automated testing and CI.

#### Features

- **Supervisor control**: Launch simulations, spawn robots, and manipulate the scene.
- **Sensor access**: Read data from cameras, LiDAR, distance sensors, IMU, GPS, etc.
- **Headless mode**: Run simulations without a display, perfect for servers and CI.
- **Remote control**: Connect to a running Webots instance or launch a new one directly from Rust.
- **Zero-copy**: Efficient memory access to sensor data buffers.

#### Usage Example

```rust
use edgebot_sim::webots::{Supervisor, Robot, Device, WebotsError};

fn main() -> Result<(), WebotsError> {
    // Launch Webots in headless mode with a world file
    let supervisor = Supervisor::launch("worlds/warehouse.wbt", true)?;

    // Spawn a robot from a prototype
    let robot = supervisor.spawn_robot("prototypes/turtlebot3.proto", "test_bot")?;

    // Step simulation to allow robot to initialize
    supervisor.step(32)?;

    // Get devices
    let camera = robot.get_device("camera")?.as_camera()?;
    let lidar = robot.get_device("lidar")?.as_lidar()?;
    let wheel_motor = robot.get_device("wheel_left")?;

    // Enable sensors with appropriate sampling period
    camera.enable(32);
    lidar.enable(32);

    // Main simulation loop
    for _ in 0..1000 {
        supervisor.step(32)?;

        // Retrieve camera image
        let image = camera.get_image()?; // RGBA buffer
        // Run inference with edgebot-core here...

        // Retrieve lidar scan
        let ranges = lidar.get_range_image()?; // Vec<f32>

        // Simple obstacle avoidance example
        if ranges.iter().any(|&r| r < 0.3) {
            // Stop or reverse
        }
    }

    // Clean shutdown
    supervisor.terminate()?;
    Ok(())
}
```

#### Headless Mode

Headless mode runs Webots without a graphical interface, ideal for automated testing and CI pipelines. Set the `WEBOTS_HOME` environment variable to your Webots installation directory:

```bash
export WEBOTS_HOME=/usr/local/webots
cargo run --bin my_simulation_test
```

The `Supervisor::launch` function automatically starts Webots with `--batch` and `--no-rendering` flags.

#### Remote Control

You can also connect to an already running Webots instance (with remote control enabled on port 1234):

```rust
let supervisor = Supervisor::connect("localhost", 1234)?;
```

#### API Reference

Key types:
- `Supervisor`: Main simulation controller. Provides `spawn_robot`, `step`, `get_root`, `load_world`, etc.
- `Robot`: Handle to a robot in the scene. Provides `get_device`, `get_node`, etc.
- `Device`: Base device; can be cast to specific device types (`as_camera`, `as_lidar`, etc.).
- `Node`: Scene tree node for robot and object manipulation.
- `Field`: Access to node fields (position, rotation, children, etc.).

Common methods:
- `Supervisor::launch(world_path, headless)` -> Launch Webots and connect.
- `Supervisor::spawn_robot(proto_url, name)` -> Create a new robot from prototype.
- `Supervisor::step(ms)` -> Advance simulation.
- `Device::enable(sampling_period)` -> Start sampling a sensor.
- `Camera::get_image()` -> Get RGBA image bytes.
- `Lidar::get_range_image()` -> Get distance measurements.

For a full list of methods, see the API documentation or the source code in `edgebot-sim/src/webots.rs`.

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

### WebAssembly Runtime

The `edgebot-wasm` crate enables deployment of EdgeBot AI models in browser and IoT environments via WebAssembly. It provides:

- **Browser support** (`wasm32-unknown-unknown`) with `wasm-bindgen` for JavaScript interop
- **WASI support** (`wasm32-wasi`) for headless IoT devices
- Zero-copy memory interfaces for efficient data passing between JS/Rust and WASM
- Unified API for both targets with runtime selection

#### Usage in JavaScript (Browser)

```javascript
// Import the WASM module (after building with `build-wasm.sh`)
import init, { JsWasmRuntime } from './edgebot-wasm-browser.js';

// Initialize the runtime
await init();

// Create runtime
const runtime = new JsWasmRuntime();

// Load a model (Uint8Array of .ebmodel or .onnx bytes)
const modelBytes = await fetch('model.ebmodel').then(r => r.arrayBuffer());
runtime.load_model('yolo', new Uint8Array(modelBytes));

// Run inference
const inputs = [{
    name: 'input',
    data: [/* float32 array */],
    shape: [1, 3, 640, 640]
}];
const outputs = runtime.infer('yolo', inputs);

console.log('Inference output:', outputs[0].data);
```

#### Usage in Rust (WASI)

```rust
use edgebot_wasm::{WasmRuntime, WasmTarget, WasmInferenceInput};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create WASI runtime
    let mut runtime = WasmRuntime::new(WasmTarget::Wasi);

    // Load model from file (WASI filesystem access)
    let model_bytes = std::fs::read("/models/yolo.ebmodel")?;
    runtime.load_model("yolo", model_bytes, None)?;

    // Prepare input
    let input = WasmInferenceInput {
        name: "input".to_string(),
        data: vec![0.0; 3 * 640 * 640],
        shape: vec![1, 3, 640, 640],
    };

    // Run inference
    let outputs = runtime.infer("yolo", &[input], None)?;
    println!("Output shape: {:?}", outputs[0].shape);

    Ok(())
}
```

#### Building WASM Modules

The `edgebot-wasm/build-wasm.sh` script builds optimized WASM binaries for both targets:

```bash
# Build browser target (default)
./edgebot-wasm/build-wasm.sh browser

# Build WASI target
./edgebot-wasm/build-wasm.sh wasi

# Build both targets
./edgebot-wasm/build-wasm.sh all

# Debug build
./edgebot-wasm/build-wasm.sh browser --debug

# With additional size optimizations
./edgebot-wasm/build-wasm.sh browser --optimize
```

Output files are placed in `target/wasm/`:
- `edgebot-wasm-browser.wasm` - Browser module (requires JS glue code)
- `edgebot-wasm-wasi.wasm` - WASI standalone module

#### Requirements

Add to your Cargo.toml:
```toml
[dependencies]
edgebot-wasm = { path = "edgebot-wasm" }

# For browser builds
[target.'cfg(target_arch = "wasm32")'.dependencies]
wasm-bindgen = "0.2"
```

Install wasm32 targets:
```bash
rustup target add wasm32-unknown-unknown wasm32-wasi
```

#### API Reference

**Core Types:**
- `WasmRuntime`: Unified runtime for model loading and inference
- `WasmTarget`: Platform target (`Browser` or `Wasi`)
- `WasmInferenceInput`: Input tensor with name, data (Vec<f32>), and shape
- `WasmInferenceOutput`: Inference result with name, data, and shape

**Key Methods:**
- `WasmRuntime::new(target)` - Create runtime for specific platform
- `load_model(name, bytes)` - Load model from bytes (requires .ebmodel or supported format)
- `infer(model_name, inputs)` - Run inference with vector of inputs
- `list_models()` - List loaded model names
- `unload_model(name)` - Free model resources

**Browser-Specific:**
- `JsWasmRuntime`: Web-friendly runtime (automatically selected via `#[cfg(target_arch = "wasm32")]`)
- `new()` constructor available from JavaScript
- Methods return `Result<..., JsValue>` for proper error handling in JS

**WASI-Specific:**
- `WasiJsRuntime::new()` - Create WASI runtime
- `load_model_from_path(name, path)` - Load model from filesystem
- Automatic support for WASI environment (files, stdin/stdout)

#### Performance Notes

- Browser target uses WGPU for GPU acceleration (via Burn's wgpu backend)
- WASI target uses CPU-optimized backends (Autocast, Tch)
- Zero-copy memory interfaces minimize data marshaling overhead
- Release builds with `opt-level = "z"` produce ~50% smaller WASM binaries
- LTO and codegen-units=1 further reduce size

## License

MIT OR Apache-2.0. See [LICENSE](LICENSE) for details.

## Contributing

We welcome contributions! See [CONTRIBUTING.md](CONTRIBUTING.md) for guidelines.

---

**Status:** Early development (Phase 1). API subject to change.
