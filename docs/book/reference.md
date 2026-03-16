# API Reference

This chapter provides an overview of the EdgeBot AI APIs. For the most up-to-date documentation, generate local docs with `cargo doc --workspace --open`.

## Core Crates

### edgebot-core

The core inference engine and abstractions.

#### InferenceEngine

```rust
pub struct InferenceEngine<B: Backend> {
    // Fields omitted
}

impl<B: Backend> InferenceEngine<B> {
    pub fn load_onnx(path: &Path, input_shape: &[usize], device: B::Device) -> Result<Self, InferenceError>
    pub fn load_bin(path: &Path, device: B::Device) -> Result<Self, InferenceError>
    pub fn forward(&self, input: Tensor<B>) -> Result<Tensor<B>, InferenceError>
    pub fn device(&self) -> &B::Device
}
```

- `load_onnx`: Load ONNX model from file
- `load_bin`: Load Burn binary format
- `forward`: Run inference on input tensor
- `device`: Get the computation device

#### Memory Safety (Zero-Copy)

```rust
pub struct CameraBuffer {
    metadata: ImageMetadata,
    buffer: ZeroCopyBuffer<u8>,
}

impl CameraBuffer {
    pub fn new(metadata: ImageMetadata) -> Self
    pub fn from_ros_image(metadata: ImageMetadata, msg: &ros2_sensor_msgs::msg::Image) -> Self
    pub fn bytes(&self) -> &[u8]
    pub fn bytes_mut(&mut self) -> &mut [u8]
    pub fn to_tensor(&self, device: &D) -> Tensor<B>
}
```

- `from_ros_image`: Zero-copy from ROS2 image message
- `bytes()` / `bytes_mut()`: Access raw buffer
- `to_tensor()`: Convert to Burn tensor

```rust
pub struct LidarBuffer {
    points: Vec<Point3D<f32>>,
    intensities: Vec<f32>,
    // ...
}

impl LidarBuffer {
    pub fn from_ros_pointcloud(msg: &ros2_sensor_msgs::msg::PointCloud2) -> Self
    pub fn points(&self) -> impl Iterator<Item = Point3D<f32>>
    pub fn to_tensor(&self, device: &D) -> Tensor<B>
}
```

#### Optimizer

```rust
pub struct ModelOptimizer;

impl ModelOptimizer {
    pub fn optimize(
        input: &Path,
        output: &Path,
        config: OptimizationConfig,
    ) -> Result<OptimizationReport, OptimizerError>
}

pub struct OptimizationConfig {
    pub quantization: QuantizationMethod,
    pub pruning: PruningStrategy,
    pub pruning_threshold: f32,
    pub fuse_layers: bool,
    pub device: Device,
}

pub enum QuantizationMethod { None, Int8, Fp16 }
pub enum PruningStrategy { None, Magnitude, Structured }
```

The `edgebot-optimize` binary is built from `src/optimizer/mod.rs`.

#### Task Abstractions

```rust
pub trait ModelTask: Send + Sync {
    fn preprocess(&self, input: &[u8]) -> Result<Tensor<B>, TaskError>;
    fn postprocess(&self, output: Tensor<B>) -> Result<TaskOutput, TaskError>;
}

pub struct YoloTask {
    config: YoloConfig,
    // ...
}

pub struct AStarTask {
    // Pathfinding configuration
}
```

## edgebot-ros2

ROS2 integration with `rclrs`.

### YoloNode

Pre-built node for object detection:

```bash
cargo run -p edgebot-ros2 --bin yolo_node --release -- --ros-args -p camera_topic:=/camera/image_raw
```

Parameters:

- `camera_topic` (string): Camera topic name
- `model_path` (string): ONNX model path
- `confidence_threshold` (float): Detection threshold

### ROS2 Helper Types

```rust
pub mod ros2 {
    pub struct Ros2ImageConverter;
    pub struct Ros2PointCloudConverter;

    impl Ros2ImageConverter {
        pub fn to_camera_buffer(msg: &ros2_sensor_msgs::msg::Image) -> CameraBuffer;
        pub fn from_camera_buffer(buffer: &CameraBuffer, msg: &mut ros2_sensor_msgs::msg::Image);
    }
}
```

## edgebot-wasm

WebAssembly runtime.

### Browser API

```rust
pub struct JsWasmRuntime {
    // Internal state
}

impl JsWasmRuntime {
    pub fn new() -> Self
    pub fn load_model(&mut self, name: &str, bytes: Vec<u8>) -> Result<(), WasmError>
    pub fn infer(&self, model_name: &str, inputs: &[WasmInferenceInput]) -> Result<Vec<WasmInferenceOutput>, WasmError>
    pub fn unload_model(&mut self, name: &str)
    pub fn list_models(&self) -> Vec<String>
}

pub struct WasmInferenceInput {
    pub name: String,
    pub data: Vec<f32>,
    pub shape: Vec<usize>,
}

pub struct WasmInferenceOutput {
    pub name: String,
    pub data: Vec<f32>,
    pub shape: Vec<usize>,
}
```

### WASI API

```rust
pub enum WasmTarget { Browser, Wasi }

pub struct WasmRuntime {
    target: WasmTarget,
    // ...
}

impl WasmRuntime {
    pub fn new(target: WasmTarget) -> Self
    pub fn load_model(&mut self, name: &str, bytes: Vec<u8>, config: Option<WasmConfig>) -> Result<(), WasmError>
    pub fn infer(&mut self, model_name: &str, inputs: &[WasmInferenceInput], config: Option<WasmConfig>) -> Result<Vec<WasmInferenceOutput>, WasmError>
}

pub struct WasmConfig {
    pub memory_pages: u32,
    pub max_stack: u32,
}
```

## edgebot-cli

Command-line interface with `clap`.

### Commands

```bash
edgebot compile [OPTIONS]
edgebot deploy [OPTIONS]
edgebot simulate [OPTIONS]
edgebot optimize [OPTIONS]
```

See `edgebot-cli/Cargo.toml` and `src/cli.rs` for argument definitions.

## edgebot-licensing

License verification.

```rust
pub fn verify_pro_access() -> Result<ProLicense, LicenseError>
pub fn has_feature(feature: ProFeature) -> bool
pub fn is_valid() -> bool

pub struct ProLicense {
    pub customer_id: String,
    pub issued_at: u64,
    pub expires_at: u64,
    pub features: Vec<String>,
}

pub enum ProFeature {
    CloudSimulation,
    AdvancedOptimization,
    PrioritySupport,
}
```

## edgebot-sim

Webots simulation integration.

```rust
pub struct Supervisor {
    // ...
}

impl Supervisor {
    pub fn launch(world: &str, headless: bool) -> Result<Self, WebotsError>
    pub fn spawn_robot(&self, proto: &str, name: &str) -> Result<Robot, WebotsError>
    pub fn step(&self, ms: i32) -> Result<(), WebotsError>
    pub fn terminate(&self) -> Result<(), WebotsError>
    pub fn connect(host: &str, port: u16) -> Result<Self, WebotsError>
}

pub struct Robot {
    // ...
}

impl Robot {
    pub fn get_device(&self, name: &str) -> Result<Device, WebotsError>
    pub fn get_node(&self, name: &str) -> Result<Node, WebotsError>
}
```

## edgebot-sim-server

Actix Web API for cloud simulation.

### Endpoints

| Method | Endpoint | Description |
|--------|----------|-------------|
| POST | `/simulate` | Upload model/world, queue simulation |
| GET | `/jobs/{id}` | Get job status |
| GET | `/jobs/{id}/results` | Fetch simulation results |
| GET | `/metrics/{model}` | Get historical performance data |
| POST | `/auth/license` | Verify Pro license |

See `edgebot-sim-server/src/controllers/` for implementation.

## edgebot-dashboard

Yew frontend for analytics.

### Components

- `Dashboard`: Main layout
- `SimulationList`: List of simulation jobs
- `MetricsChart`: Performance charts (using `plotters` or `chart.js`)
- `LicenseStatus`: Pro subscription status

Configuration via environment variables:

```bash
EDGEBOT_SIM_SERVER_URL=https://sim.edgebot.ai
EDGEBOT_LICENSE_KEY=your_key_here
```

## Common Types

```rust
pub struct ImageMetadata {
    pub width: usize,
    pub height: usize,
    pub format: ImageFormat,
}

pub enum ImageFormat {
    RGB, BGR, RGBA, BGRA, Grayscale, Depth,
}

pub struct Point3D<T> {
    pub x: T,
    pub y: T,
    pub z: T,
}

pub struct OptimizationReport {
    pub original_size: usize,
    pub optimized_size: usize,
    pub size_reduction_percent: f32,
    pub estimated_speedup: f32,
    pub operations: Vec<OptimizationOperation>,
}
```

## Error Handling

Most crates use `thiserror` for error types. Common errors:

```rust
pub enum InferenceError {
    ModelNotFound,
    InvalidModelFormat,
    DeviceError(String),
    InferenceFailed(String),
}

pub enum OptimizerError {
    LoadFailed(String),
    QuantizationFailed(String),
    PruningFailed(String),
}

pub enum LicenseError {
    NotSet,
    InvalidFormat,
    InvalidSignature,
    Expired,
    FeatureNotEnabled(String),
}
```

Use `Result<T, E>` types and handle errors appropriately.

## Further Reading

- `cargo doc --workspace --open` for full API docs
- Source code in each crate's `src/` directory
- Examples in each crate's `examples/` directory
- Integration tests in `tests/` directories
