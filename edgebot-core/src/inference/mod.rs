//! Inference engine for loading and running AI models using Burn backends.
//!
//! Supports multiple backends (Autocast, Tch) and model formats (ONNX, Burn binary).
//! Provides zero-copy tensor operations for efficient edge deployment.

use burn::{
    backend::{Backend, Autocast},
    nn::{Module, Model},
    record::{FullPrecisionSettings, LoadError},
    tensor::Tensor,
};
use burn_import::onnx::{import_onnx, Error as OnnxError};
use std::path::Path;
use thiserror::Error;

/// Errors that can occur during inference operations.
#[derive(Error, Debug)]
pub enum InferenceError {
    /// IO error (file not found, permission denied, etc.)
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    /// ONNX import error
    #[error("ONNX import error: {0}")]
    Onnx(#[from] OnnxError),

    /// Burn tensor or model error
    #[error("Burn error: {0}")]
    Burn(String),

    /// Unsupported model format
    #[error("Unsupported model format: {0}")]
    UnsupportedFormat(String),
}

/// Inference engine holding a loaded model and its associated device.
///
/// Generic over Burn backend `B`. The backend determines tensor computation
/// backend (e.g., Tch for PyTorch, Autocast for mixed precision).
pub struct InferenceEngine<B: Backend> {
    model: Model<B>,
    device: B::Device,
}

impl<B: Backend> InferenceEngine<B> {
    /// Create a new inference engine with a pre-loaded model and device.
    pub fn new(model: Model<B>, device: B::Device) -> Self {
        Self { model, device }
    }

    /// Run inference on the given input tensor.
    ///
    /// Returns the output tensor produced by the model.
    pub fn forward(&self, input: Tensor<B>) -> Result<Tensor<B>, InferenceError> {
        self.model
            .forward(input)
            .map_err(|e| InferenceError::Burn(e.to_string()))
    }

    /// Load a model from an ONNX file.
    ///
    /// * `path` - Path to the ONNX model file.
    /// * `input_shape` - Shape of the input tensor (e.g., &[1, 3, 224, 224]).
    /// * `device` - Device to run inference on (CPU or GPU).
    pub fn load_onnx(
        path: &Path,
        input_shape: &[usize],
        device: B::Device,
    ) -> Result<Self, InferenceError> {
        let model = import_onnx::<B>(path, input_shape)?;
        Ok(Self::new(model, device))
    }

    /// Load a model from a Burn binary (.bin) file.
    ///
    /// * `path` - Path to the binary model file.
    /// * `device` - Device to run inference on.
    pub fn load_bin(path: &Path, device: B::Device) -> Result<Self, InferenceError> {
        let record = burn::record::load(path, &FullPrecisionSettings::default())
            .map_err(|e| match e {
                LoadError::InvalidRecord(msg) => InferenceError::Burn(msg),
                LoadError::Io(err) => InferenceError::Io(err),
            })?;
        let model = Model::from_record(record, &device)
            .map_err(|e| InferenceError::Burn(e.to_string()))?;
        Ok(Self::new(model, device))
    }

    /// Load a model automatically based on file extension.
    ///
    /// Supported formats:
    /// - `.onnx` - ONNX model (requires input_shape)
    /// - `.bin` - Burn binary model
    ///
    /// * `path` - Path to the model file.
    /// * `input_shape` - Required for ONNX models, ignored for .bin.
    /// * `device` - Device to run inference on.
    pub fn load(
        path: &Path,
        input_shape: &[usize],
        device: B::Device,
    ) -> Result<Self, InferenceError> {
        let ext = path
            .extension()
            .and_then(|s| s.to_str())
            .ok_or_else(|| InferenceError::UnsupportedFormat("missing or invalid extension".to_string()))?;

        match ext {
            "onnx" => Self::load_onnx(path, input_shape, device),
            "bin" => Self::load_bin(path, device),
            _ => Err(InferenceError::UnsupportedFormat(format!(
                "{} format not supported",
                ext
            ))),
        }
    }

    /// Get a reference to the underlying model.
    pub fn model(&self) -> &Model<B> {
        &self.model
    }

    /// Get the device used for inference.
    pub fn device(&self) -> &B::Device {
        &self.device
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use burn::backend::tch::TchBackend;

    #[test]
    fn test_load_and_forward_bin() {
        // Create a simple linear model: y = Wx + b
        let device = TchBackend::Device::default();
        let input_features = 3;
        let output_features = 2;

        // Build model
        let linear = burn::nn::Linear::new(
            burn::nn::LinearConfig::new(input_features, output_features)
        );
        let model = Model::new(linear);

        // Save to temporary file
        let temp_dir = std::env::temp_dir();
        let path = temp_dir.join("test_linear_model.bin");
        {
            let record = model.to_record();
            burn::record::save(&record, &path).expect("Failed to save model");
        }

        // Load using InferenceEngine
        let engine = InferenceEngine::<TchBackend>::load_bin(&path, device.clone())
            .expect("Failed to load model");

        // Create random input tensor
        let input = Tensor::<TchBackend>::random(
            [1, input_features],
            burn::tensor::Distribution::Uniform(-1.0, 1.0),
            &device,
        );

        // Run inference
        let output = engine.forward(input).expect("Forward failed");

        // Verify output shape
        assert_eq!(output.dims(), &[1, output_features]);
    }

    #[test]
    fn test_unsupported_format() {
        let device = TchBackend::Device::default();
        let fake_path = Path::new("model.xyz");
        let result = InferenceEngine::<TchBackend>::load(fake_path, &[1, 3], device);
        assert!(matches!(result, Err(InferenceError::UnsupportedFormat(_))));
    }

    #[test]
    fn test_missing_extension() {
        let device = TchBackend::Device::default();
        let fake_path = Path::new("no_extension");
        let result = InferenceEngine::<TchBackend>::load(fake_path, &[1, 3], device);
        assert!(matches!(result, Err(InferenceError::UnsupportedFormat(_))));
    }
}
