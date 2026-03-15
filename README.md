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

**Phase 4: Deployment & Monetization** - In progress

- [x] Phase 4 Task 1: EdgeBot CLI (deploy, simulate, optimize commands) ✅ COMPLETED
- [ ] Phase 4 Task 2: License verification system
- [ ] Phase 4 Task 3: Dashboard frontend
- [ ] Phase 4 Task 4: Comprehensive documentation

See [TASKS.md](TASKS.md) for complete roadmap.

## EdgeBot CLI

The `edgebot-cli` is the main command-line interface for end-users, providing commands for deployment, simulation, optimization, and cross-compilation.

### Installation

Build from the workspace:

```bash
cargo build --release -p edgebot-cli
# Binary will be at: target/release/edgebot
```

Or install globally:

```bash
cargo install --path edgebot-cli
```

### Commands

#### 1. Compile for ARM targets

Cross-compile models for embedded ARM devices (Raspberry Pi, STM32, generic ARM).

```bash
# Auto-detect hardware and compile
edgebot compile --model model.onnx --output binary

# Compile for specific hardware
edgebot compile --model model.onnx --output binary --hardware raspberry-pi

# Compile for all supported targets
edgebot compile --model model.onnx --output-dir ./bin/ --all

# With optimization features
edgebot compile --model model.onnx --output binary --release --features "tch,cuda"
```

**Options:**
- `--model <path>`: Model file to embed (optional)
- `--output <path>`: Output binary path
- `--target <triple>`: Target triple (e.g., aarch64-unknown-linux-musl)
- `--hardware <type>`: Hardware type (raspberry-pi, stm32, generic-arm)
- `--all`: Build for all supported ARM targets
- `--release`: Build in release mode
- `--features <feat>`: Enable Cargo features
- `--static-link`: Statically link all dependencies

#### 2. Deploy to device

Deploy compiled binaries to remote devices via SSH or serial connection.

```bash
# Deploy via SSH (using SSH agent)
edgebot deploy --binary ./target/release/edgebot --target 192.168.1.100 --username pi

# Deploy with password authentication
edgebot deploy --binary ./edgebot --target 192.168.1.100 --username pi --password secret

# Deploy to custom destination path
edgebot deploy --binary ./edgebot --target 192.168.1.100 --username pi --destination /home/pi/apps/

# Serial deployment (placeholder)
edgebot deploy --binary ./edgebot --target /dev/ttyUSB0 --method serial
```

**Options:**
- `--binary <path>`: Binary file to deploy (required)
- `--target <addr>`: Target IP address or serial port (required)
- `--method <ssh|serial>`: Deployment method (default: ssh)
- `--destination <path>`: Remote path (default: /opt/edgebot/)
- `--username <user>`: SSH username (required for SSH)
- `--password <pass>`: SSH password (optional; uses SSH agent if omitted)

**Note:** Serial deployment is not yet fully implemented. Use SSH for now.

#### 3. Run simulation

Test models in either local Webots simulation or cloud simulation server.

##### Local Simulation (Webots)

```bash
# Run local simulation with a world file
edgebot simulate --model model.ebmodel --world worlds/warehouse.wbt --runs 10

# Output JSON results
edgebot simulate --model model.ebmodel --world worlds/warehouse.wbt --json

# Adjust simulation timestep
edgebot simulate --model model.ebmodel --world worlds/warehouse.wbt --timestep 16
```

##### Cloud Simulation

```bash
# Run on cloud simulation server
edgebot simulate --model model.ebmodel --cloud --server http://localhost:8080 --runs 100

# With custom world file uploaded to server
edgebot simulate --model model.ebmodel --world worlds/custom.wbt --cloud --server http://sim.edgebot.ai
```

**Options:**
- `--model <path>`: Model file to test (required)
- `--world <path>`: Webots world file (required for local, optional for cloud)
- `--cloud`: Use cloud simulation server
- `--server <url>`: Cloud server URL (default: http://localhost:8080)
- `--runs <n>`: Number of simulation runs (default: 1)
- `--json`: Output results as JSON
- `--timestep <ms>`: Simulation timestep in milliseconds (default: 32)

**Output:** Simulation metrics including total steps, runtime, average inference time.

#### 4. Optimize models

Optimize models for edge deployment with quantization, pruning, and layer fusion.

```bash
# Basic optimization with int8 quantization and layer fusion
edgebot optimize --input model.onnx --output model.ebmodel --quantize int8 --fuse-layers

# Advanced: fp16 quantization + magnitude pruning (50% threshold)
edgebot optimize \
  --input model.onnx \
  --output model.ebmodel \
  --quantize fp16 \
  --prune magnitude \
  --pruning-threshold 0.5 \
  --device cpu

# No optimization (just convert format)
edgebot optimize --input model.onnx --output model.ebmodel --quantize none
```

**Options:**
- `--input <path>`: Input model file (ONNX or Burn .bin) (required)
- `--output <path>`: Output optimized model (.ebmodel) (required)
- `--quantize <none|int8|fp16>`: Quantization method (default: none)
- `--prune <none|magnitude|structured>`: Pruning strategy (default: none)
- `--pruning-threshold <0.0-1.0>`: Fraction of weights to prune (default: 0.5)
- `--fuse-layers`: Enable layer fusion (Conv+ReLU, etc.)
- `--device <cpu|cuda>`: Target device (default: cpu)

**Output:** `.ebmodel` bundle containing optimized model and metadata. The CLI reports size reduction and optimization statistics.

### Usage

**Free Tier:**
- All core SDK features are free and open source (MIT/Apache-2.0)
- Local simulation with Webots is unlimited
- Basic model optimization (quantization none, no pruning)
- Cross-compilation for ARM targets

**Pro Tier ($29/month):**
- Cloud simulation with batch runs (100+ scenarios)
- Advanced model optimization: int8/fp16 quantization, pruning, layer fusion
- Priority support and custom integrations
- Offline activation tokens available

To enable pro features, set the `EDGEBOT_LICENSE_KEY` environment variable:

```bash
export EDGEBOT_LICENSE_KEY="your_license_key_here"
```

The license key is an Ed25519 signed token verified offline. Contact sales@edgebot.ai to obtain a pro license.

## EdgeBot Dashboard

The EdgeBot Dashboard is a modern web application built with Yew and Rust WebAssembly. It provides a comprehensive interface for monitoring simulation results, tracking model performance metrics, and managing your Pro subscription.

### Features

- **Simulation Monitoring**: View real-time and historical simulation jobs, including FPS, inference latency, memory usage, and detailed performance breakdowns.
- **Model Metrics**: Track inference latency, memory footprint, and model size across different platforms (x86_64, ARM, WASM).
- **License Management**: Check your subscription status, view active features, and manage your EDGEBOT_LICENSE_KEY.

### Building

Build the dashboard using **Trunk** (the recommended approach):

```bash
# Install trunk (once)
cargo install trunk

# Build for production
cd edgebot-dashboard
trunk build --release
```

Alternatively, use the included build script:

```bash
cd edgebot-dashboard
./build-dashboard-wasm.sh --release
```

The compiled static files will be in the `dist/` directory.

### Running Locally

Start a local development server with hot reloading:

```bash
cd edgebot-dashboard
trunk serve --open
```

Or serve the built files:

```bash
cd edgebot-dashboard/dist
python3 -m http.server 8000
# Open http://localhost:8080
```

### Deployment

The dashboard is a static site and can be deployed to any static hosting service.

#### GitHub Pages

1. Build the dashboard: `trunk build --release`
2. Copy the build output to the `docs/` directory (which GitHub Pages uses):
   ```bash
   cp -r dist/* ../docs/
   ```
3. Commit and push to GitHub. GitHub Pages will automatically serve from the `docs/` folder.
   Alternatively, use the provided GitHub Actions workflow (`.github/workflows/dashboard.yml`) for automatic deployment on push to `main`.

> **Note**: If your repository is served from a subpath (e.g., `https://username.github.io/edgebot-ai/`), you may need to set `public_url` in `edgebot-dashboard/trunk.toml` accordingly (e.g., `public_url = "/edgebot-ai"`).

#### Netlify

- Build command: `trunk build --release`
- Publish directory: `edgebot-dashboard/dist`
- Add an environment variable `EDGEBOT_SIM_SERVER_URL` if connecting to a remote simulation server.

### Configuration

- **Simulation Server**: By default, the dashboard connects to `http://localhost:8080`. Override via the `EDGEBOT_SIM_SERVER_URL` environment variable.
- **License**: Pro license status is verified locally. Set `EDGEBOT_LICENSE_KEY` in the environment (or in your shell) to enable Pro features.

### Architecture

The dashboard integrates with the EdgeBot ecosystem via two main APIs:

- **SimServerClient**: Fetches simulation jobs and results from the `edgebot-sim-server`.
- **LicensingClient**: Checks local license verification using the `edgebot-licensing` crate.

All data is fetched asynchronously using `wasm_bindgen_futures` and displayed using reactive Yew components.

## Monetization & License Verification

EdgeBot AI uses a freemium model:

- **Free Core SDK**: All core inference, memory safety, ROS2 integration, and WebAssembly compilation remain open source under MIT/Apache-2.0.
- **Pro Features**: Cloud simulation and advanced optimization require a paid subscription.

### License Verification System

The `edgebot-licensing` crate implements Ed25519-based license verification:

- License keys are signed by EdgeBot AI's private key
- Supports offline activation (no phone home)
- Includes expiry dates and feature flags
- Fast cryptographic verification using `ed25519-dalek`

**Command-line usage:**

```bash
# Cloud simulation requires pro license
edgebot simulate --model model.ebmodel --cloud --server https://sim.edgebot.ai

# Model optimization requires pro license
edgebot optimize --input model.onnx --output model.ebmodel --quantize int8 --fuse-layers
```

If no valid license is found, the commands will return an error with instructions.

**Environment Variable:**

Set `EDGEBOT_LICENSE_KEY` with your license key:

```bash
export EDGEBOT_LICENSE_KEY="signature_base64:payload_base64"
```

The license key format is two base64-encoded parts separated by a colon:
- `signature`: Ed25519 signature over the payload
- `payload`: JSON containing customer_id, timestamp, features, and optional expiry

**Development License:**

During development, you can generate test licenses using the `generate_dev_license` function (only available in debug builds):

```rust
#[cfg(debug_assertions)]
let license = edgebot_licensing::generate_dev_license(
    "test_customer",
    vec!["cloud_sim", "optimization"],
    "your_secret_key_base64"
)?;
```

### Obtaining a Pro License

Visit https://edgebot.ai/pricing to subscribe. After payment, you'll receive a license key via email. The key is valid for the subscription period and can be used on multiple machines.

For enterprise deployments, offline activation tokens and custom feature flags are available. Contact sales@edgebot.ai.

### Architecture Highlights

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
