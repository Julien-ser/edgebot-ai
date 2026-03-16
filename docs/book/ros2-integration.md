# ROS2 Integration

EdgeBot AI provides seamless ROS2 (Robot Operating System 2) integration through the `edgebot-ros2` crate. This enables you to:

- Subscribe to ROS2 topics (camera images, LiDAR, sensor data)
- Publish inference results (detections, pathfinding outputs)
- Zero-copy message passing for high-performance data transfer
- Run AI inference on live robot data

## Prerequisites

- ROS2 installation (Humble or Iron recommended)
- ROS2 environment sourced (`source /opt/ros/<distro>/setup.bash`)
- A ROS2 topic publishing sensor data (e.g., camera images)

## Setting Up ROS2

### 1. Install ROS2

Follow the official ROS2 installation guide for your platform:

- Ubuntu: https://docs.ros.org/en/humble/Installation/Ubuntu-Install-Debians.html
- macOS: https://docs.ros.org/en/humble/Installation/macOS-Install-Binaries.html
- Windows: https://docs.ros.org/en/humble/Installation/Windows-Install-Binaries.html

### 2. Source ROS2 Environment

```bash
source /opt/ros/humble/setup.bash
```

### 3. Build edgebot-ros2

```bash
cd edgebot-ai
cargo build -p edgebot-ros2 --release
```

## YOLO Inference Node Example

The `edgebot-ros2` crate includes a complete YOLO object detection node that:

- Subscribes to a camera topic (`/camera/color/image_raw`)
- Runs inference using `edgebot-core`
- Publishes detection results to `/edgebot/detections`

### Running the Node

```bash
# Build and run
cargo run -p edgebot-ros2 --bin yolo_node --release

# With custom camera topic
cargo run -p edgebot-ros2 --bin yolo_node -- --ros-args -p camera_topic:=/my_camera/image_raw
```

The node uses the following ROS2 message types:

- Input: `ros2_sensor_msgs/msg/Image` (camera images)
- Output: `vision_msgs/msg/Detection2DArray` (object detections)

### Node Parameters

| Parameter | Type | Default | Description |
|-----------|------|---------|-------------|
| `camera_topic` | string | `/camera/color/image_raw` | ROS2 topic for camera images |
| `model_path` | string | `models/yolov8.onnx` | Path to ONNX model file |
| `confidence_threshold` | float | `0.5` | Minimum confidence for detections |
| `output_topic` | string | `/edgebot/detections` | Topic to publish detection results |

Pass parameters via command line:

```bash
cargo run -p edgebot-ros2 --bin yolo_node -- --ros-args -p camera_topic:=/camera/depth/image_raw -p confidence_threshold:=0.7
```

## Zero-Copy Memory Interface

The `edgebot-core/memory` module provides safe abstractions for sharing sensor data without memory copies between ROS2 messages and inference pipelines.

### Key Types

- `ZeroCopyBuffer<T>`: Safe buffer using `MaybeUninit` for uninitialized memory
- `CameraBuffer`: Zero-copy image buffer (RGB, BGR, RGBA, grayscale, depth)
- `LidarBuffer`: Point cloud buffer for LiDAR data
- `BorrowedBuffer<'a, T>`: Temporary view into initialized data

### Example: Camera Image to Tensor

```rust
use edgebot_core::memory::{CameraBuffer, ImageFormat, ImageMetadata, ZeroCopyBuffer};
use edgebot_core::inference::InferenceEngine;
use burn::backend::tch::TchBackend;

// Assume you have ROS2 image data (from rclrs message)
fn process_image(ros2_image: &ros2_sensor_msgs::msg::Image) -> Result<(), Box<dyn std::error::Error>> {
    // Create metadata from ROS2 image
    let metadata = ImageMetadata::new(
        ros2_image.width as usize,
        ros2_image.height as usize,
        ImageFormat::from_ros_encoding(&ros2_image.encoding)?,
    );

    // Create zero-copy buffer pointing to ROS2 message data
    let mut buffer = CameraBuffer::from_ros_image(metadata, ros2_image);

    // Convert to Burn tensor (may involve minimal copy for format conversion)
    let device = TchBackend::Device::default();
    let tensor = buffer.to_tensor(&device);

    // Run inference
    let engine = InferenceEngine::<TchBackend>::load_onnx(
        Path::new("model.onnx"),
        &[1, 3, metadata.height as usize, metadata.width as usize],
        device,
    )?;

    let output = engine.forward(tensor)?;
    // Process output...

    Ok(())
}
```

### Supported Image Formats

| ROS2 Encoding | EdgeBot Format | Notes |
|---------------|----------------|-------|
| `rgb8` | `RGB` | 3-channel RGB |
| `bgr8` | `BGR` | 3-channel BGR |
| `rgba8` | `RGBA` | 4-channel RGBA |
| `bgra8` | `BGRA` | 4-channel BGRA |
| `mono8` | `Grayscale` | Single channel |
| `32FC1` | `Depth` | 32-bit float depth |
| `16UC1` | `Depth` | 16-bit unsigned depth |

### LiDAR Point Clouds

```rust
use edgebot_core::memory::{LidarBuffer, PointCloudFormat};

fn process_lidar(point_cloud: &ros2_sensor_msgs::msg::PointCloud2) {
    let buffer = LidarBuffer::from_ros_pointcloud(point_cloud);

    // Access point data efficiently
    for point in buffer.points() {
        println!("x: {}, y: {}, z: {}, intensity: {}",
                 point.x, point.y, point.z, point.intensity);
    }

    // Convert to tensor for inference if needed
    // let tensor = buffer.to_tensor(&device);
}
```

Supported point fields: `xyz`, `intensity`, `ring`, `timestamp`.

## Custom ROS2 Nodes

You can build custom ROS2 nodes using the `rclrs` crate directly, combining it with `edgebot-core` for inference:

### Project Setup

Add to your `Cargo.toml`:

```toml
[dependencies]
rclrs = "0.3"
edgebot-core = { path = "../edgebot-core" }
```

### Minimal Node Template

```rust
use rclrs::RclRust;
use std::time::Duration;
use edgebot_core::inference::InferenceEngine;
use burn::backend::tch::TchBackend;

struct InferenceNode {
    engine: InferenceEngine<TchBackend>,
    camera_sub: rclrs::Subscription<ros2_sensor_msgs::msg::Image>,
    detection_pub: rclrs::Publisher<vision_msgs::msg::Detection2DArray>,
}

impl InferenceNode {
    fn new(engine: InferenceEngine<TchBackend>) -> Self {
        let ctx = RclRust::get_instance();
        let node = ctx.create_node("inference_node").unwrap();

        let camera_sub = node.create_subscription::<ros2_sensor_msgs::msg::Image>(
            "camera",
            |msg| {
                // Callback: process image
                Self::process_image(msg, &engine);
            },
            10,
        ).unwrap();

        let detection_pub = node.create_publisher::<vision_msgs::msg::Detection2DArray>(
            "detections",
            10,
        ).unwrap();

        Self { engine, camera_sub, detection_pub }
    }

    fn process_image(&self, msg: &ros2_sensor_msgs::msg::Image) {
        // Zero-copy conversion
        let tensor = CameraBuffer::from_ros_image(msg).to_tensor(&self.engine.device());

        // Inference
        let output = self.engine.forward(tensor).unwrap();

        // Publish results
        let detections = self.decode_output(output);
        self.detection_pub.publish(&detections).unwrap();
    }

    fn decode_output(&self, output: burn::tensor::Tensor<TchBackend>) -> ros2_vision_msgs::msg::Detection2DArray {
        // Implement decoding logic
        unimplemented!()
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let rcl_rust = RclRust::init(None)?;
    let node = rcl_rust.create_node("inference_node")?;

    // Load model
    let device = TchBackend::Device::default();
    let engine = InferenceEngine::<TchBackend>::load_onnx(
        Path::new("model.onnx"),
        &[1, 3, 224, 224],
        device,
    )?;

    let _inference_node = InferenceNode::new(engine);

    // Spin
    let executor = rcl_rust.create_executor()?;
    executor.add_node(node)?;
    executor.spin()?;

    Ok(())
}
```

## Best Practices

1. **Use Zero-Copy**: Always use `CameraBuffer::from_ros_image()` and `LidarBuffer::from_ros_pointcloud()` for zero-copy access instead of copying message data.
2. **Reuse Buffers**: Keep buffers alive between callbacks to avoid repeated allocations.
3. **Batch Inference**: For high-throughput applications, accumulate multiple sensor readings and run batched inference.
4. **Device Selection**: Use the appropriate Burn backend (Tch for CPU, Autocast for mixed precision) based on your robot's hardware.
5. **Memory Safety**: Rust's ownership system ensures no data races, but be careful with ROS2 loaned messages—copy data immediately if you need to hold it beyond the callback.

## Troubleshooting

| Issue | Solution |
|-------|----------|
| `rclrs` not found | Source your ROS2 environment (`source /opt/ros/humble/setup.bash`) before building |
| Camera image format not supported | Convert in ROS2 using `image_proc` or add format support in `edgebot-core::memory` |
| Inference too slow | Use quantization (int8), reduce input resolution, or batch multiple inputs |
| Memory leaks in callback | Avoid storing references to loaned messages; copy data into owned buffers |
| WASM build fails for WASI target | WASI doesn't support all std features; use `wasm32-wasi` target only for headless IoT |

## Next Steps

- [WebAssembly Deployment](wasm-deployment.md): Compile and run EdgeBot models in browser/WASI environments
- [Pro Workflow](pro-workflow.md): Use cloud simulation and advanced optimization
- [API Reference](reference.md): Explore the full API surface
