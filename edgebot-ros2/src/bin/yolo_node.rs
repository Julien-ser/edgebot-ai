//! EdgeBot YOLO ROS2 Node Example
//!
//! Demonstrates how to use the ROS2 bridge to subscribe to camera images,
//! run YOLO inference, and publish detection results.

use edgebot_ros2::Ros2Bridge;
use edgebot_core::inference::InferenceEngine;
use edgebot_core::memory::camera::{CameraBuffer, ImageFormat, ImageMetadata};
use std::sync::Arc;
use std::path::Path;

use ros2_sensor_msgs::msg::Image as Ros2Image;
use ros2_vision_msgs::msg::Detection2DArray as Ros2Detections;
use ros2_std_msgs::msg::Header;

use burn::backend::tch::TchBackend;
use burn::tensor::Tensor;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize ROS2 bridge
    let bridge = Ros2Bridge::new("edgebot_yolo_node", "")?;
    let node = bridge.node();

    println!("EdgeBot YOLO node starting...");

    // Load YOLO model
    let model_path = "models/yolov8.onnx";
    let input_shape = [1, 3, 640, 640];
    let device = TchBackend::Device::default();
    let engine = InferenceEngine::<TchBackend>::load_onnx(
        Path::new(model_path),
        &input_shape,
        device.clone(),
    )?;
    println!("YOLO model loaded from {}", model_path);

    // Quality of Service profile
    let qos = rclrs::Qos::default();

    // Publisher for detection results
    let publisher = bridge.create_publisher::<Ros2Detections>("/detections", qos);

    // Subscriber for camera images
    bridge.create_subscription::<Ros2Image, _>("/camera/image_raw", qos, |msg| {
        // Convert ROS2 Image to CameraBuffer
        let buffer = ros2_image_to_camera_buffer(msg);
        // Convert to Burn tensor
        let tensor = buffer.to_tensor(&device);
        // Run inference
        match engine.forward(tensor) {
            Ok(output) => {
                // Convert model output to detections
                let detections = output_to_detections(&output, buffer.metadata());
                // Publish
                if let Err(e) = publisher.publish(&detections) {
                    eprintln!("Failed to publish detections: {:?}", e);
                }
            }
            Err(e) => eprintln!("Inference error: {:?}", e),
        }
    });

    println!("Subscribed to /camera/image_raw, publishing detections to /detections");
    println!("Press Ctrl+C to exit.");

    // Spin the ROS2 event loop
    rclrs::spin(node).expect("ROS2 spin failed");
    Ok(())
}

/// Convert a ROS2 Image message to a CameraBuffer.
///
/// This performs a copy of the image data. For zero-copy, use loaned API.
fn ros2_image_to_camera_buffer(msg: &Ros2Image) -> CameraBuffer {
    let width = msg.width as usize;
    let height = msg.height as usize;
    let format = match msg.encoding.as_str() {
        "rgb8" => ImageFormat::RGB,
        "rgba8" => ImageFormat::RGBA,
        "bgr8" => ImageFormat::BGR,
        "bgra8" => ImageFormat::BGRA,
        "mono8" => ImageFormat::Grayscale,
        "16UC1" => ImageFormat::Depth,
        "32FC1" => ImageFormat::DepthF32,
        _ => panic!("Unsupported encoding: {}", msg.encoding),
    };
    let metadata = ImageMetadata::new(width, height, format);
    // Copy data from the message into the buffer
    CameraBuffer::from_vec(msg.data.clone(), metadata)
}

/// Placeholder: Convert YOLO model output to ROS2 Detection2DArray.
fn output_to_detections(_output: &Tensor<TchBackend>, _metadata: ImageMetadata) -> Ros2Detections {
    // In a real implementation, decode the YOLO output tensor to extract
    // bounding boxes, confidence scores, and class IDs.
    // For now, return an empty detection array.
    Ros2Detections {
        header: Header {
            stamp: Default::default(),
            frame_id: "camera".to_string(),
        },
        detections: vec![],
    }
}
