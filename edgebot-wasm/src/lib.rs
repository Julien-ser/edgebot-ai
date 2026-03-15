//! EdgeBot WebAssembly - WASM runtime for browser and IoT
//!
//! This crate provides WebAssembly compilation support for EdgeBot AI,
//! enabling deployment in browser environments and WASI-compatible IoT devices.
//! Uses wasm-bindgen for JavaScript interop.

#![cfg_attr(target_arch = "wasm32", allow(unused_imports))]

pub mod runtime;

/// WASM runtime version
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

// Re-export key types for convenient access
pub use runtime::{
    WasmRuntime, WasmRuntimeError, WasmModel, WasmInferenceInput, WasmInferenceOutput,
    WasmTarget, BrowserRuntime, WasiRuntime,
};

// Browser-specific bindings via wasm-bindgen
#[cfg(target_arch = "wasm32")]
mod bindings {
    use wasm_bindgen::prelude::*;
    use super::runtime::{WasmRuntime, WasmTarget, WasmInferenceInput, WasmInferenceOutput};

    /// Browser-friendly WASM runtime with JavaScript bindings
    #[wasm_bindgen]
    pub struct JsWasmRuntime {
        inner: WasmRuntime,
    }

    #[wasm_bindgen]
    impl JsWasmRuntime {
        /// Create a new runtime for the browser
        #[wasm_bindgen(constructor)]
        pub fn new() -> Self {
            Self {
                inner: WasmRuntime::new(WasmTarget::Browser),
            }
        }

        /// Load a model from a byte array (Uint8Array in JS)
        pub fn load_model(&mut self, name: &str, bytes: Vec<u8>) -> Result<(), JsValue> {
            self.inner.load_model(name, bytes, None)
                .map_err(|e| JsValue::from_str(&e.to_string()))
        }

        /// Run inference with input tensors
        pub fn infer(&self, model_name: &str, inputs: Vec<JsInferenceInput>) -> Result<Vec<WasmInferenceOutput>, JsValue> {
            let rust_inputs: Vec<WasmInferenceInput> = inputs.into_iter()
                .map(|js| WasmInferenceInput {
                    name: js.name,
                    data: js.data,
                    shape: js.shape,
                })
                .collect();

            self.inner.infer(model_name, &rust_inputs, None)
                .map_err(|e| JsValue::from_str(&e.to_string()))
        }

        /// List all loaded models
        pub fn list_models(&self) -> Vec<String> {
            self.inner.list_models(None)
        }

        /// Unload a model
        pub fn unload_model(&mut self, name: &str) -> Result<(), JsValue> {
            self.inner.unload_model(name, None)
                .map_err(|e| JsValue::from_str(&e.to_string()))
        }

        /// Get runtime version
        pub fn version() -> String {
            VERSION.to_string()
        }

        /// Get target platform info
        pub fn target(&self) -> String {
            format!("{:?}", self.inner.default_target())
        }
    }

    /// JavaScript-friendly inference input
    #[wasm_bindgen]
    #[derive(Debug, Clone)]
    pub struct JsInferenceInput {
        pub name: String,
        pub data: Vec<f32>,
        pub shape: Vec<usize>,
    }

    /// JavaScript-friendly inference output
    #[wasm_bindgen]
    #[derive(Debug, Clone)]
    pub struct JsInferenceOutput {
        pub name: String,
        pub data: Vec<f32>,
        pub shape: Vec<usize>,
    }

    // Convert between Rust and JS types
    impl From<WasmInferenceOutput> for JsInferenceOutput {
        fn from(output: WasmInferenceOutput) -> Self {
            Self {
                name: output.name,
                data: output.data,
                shape: output.shape,
            }
        }
    }
}

// WASI-specific bindings
#[cfg(target_os = "wasi")]
mod wasi_bindings {
    use super::runtime::{WasmRuntime, WasmTarget, WasmInferenceInput, WasmInferenceOutput};

    /// WASI-friendly runtime for IoT devices
    pub struct WasiJsRuntime {
        inner: WasmRuntime,
    }

    impl WasiJsRuntime {
        /// Create a new runtime for WASI
        pub fn new() -> Self {
            Self {
                inner: WasmRuntime::new(WasmTarget::Wasi),
            }
        }

        /// Load a model from file path (WASI filesystem access)
        pub fn load_model_from_path(&mut self, name: &str, path: &str) -> Result<(), String> {
            use std::fs;
            fs::read(path)
                .map_err(|e| e.to_string())
                .and_then(|bytes| self.inner.load_model(name, bytes, None)
                    .map_err(|e| e.to_string()))
        }

        /// Run inference (same as browser version)
        pub fn infer(
            &self,
            model_name: &str,
            inputs: Vec<WasmInferenceInput>,
        ) -> Result<Vec<WasmInferenceOutput>, String> {
            self.inner.infer(model_name, &inputs, None)
                .map_err(|e| e.to_string())
        }

        /// List models
        pub fn list_models(&self) -> Vec<String> {
            self.inner.list_models(None)
        }

        /// Version info
        pub fn version() -> &'static str {
            VERSION
        }
    }
}

// Re-export key types for convenient access
pub use runtime::{
    WasmRuntime, WasmRuntimeError, WasmModel, WasmInferenceInput, WasmInferenceOutput,
    WasmTarget, BrowserRuntime, WasiRuntime,
};
