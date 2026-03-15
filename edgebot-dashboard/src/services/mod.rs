//! API services for communicating with EdgeBot backends

use serde::{Deserialize, Serialize};
use std::env;

/// Simulation job data from the sim-server API
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SimulationJob {
    pub id: String,
    pub status: String,
    pub model_name: String,
    pub world_file: Option<String>,
    pub scenes: Vec<String>,
    pub metrics: Option<SimulationMetrics>,
    pub error: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SimulationMetrics {
    pub fps: f64,
    pub memory_peak_mb: f64,
    pub inference_latency_ms: f64,
    pub total_frames: u32,
    pub scene_name: String,
}

/// Client for the EdgeBot Simulation Server API
pub struct SimServerClient {
    base_url: String,
}

impl SimServerClient {
    /// Create a new client with the given server URL
    pub fn new(base_url: &str) -> Self {
        Self {
            base_url: base_url.to_string(),
        }
    }

    /// Create a client using EDGEBOT_SIM_SERVER_URL env var or default
    pub fn from_env() -> Self {
        let base_url = env::var("EDGEBOT_SIM_SERVER_URL")
            .unwrap_or_else(|_| "http://localhost:8080".to_string());
        Self::new(&base_url)
    }

    /// Fetch all simulation jobs
    pub async fn list_jobs(&self) -> Result<Vec<SimulationJob>, String> {
        let url = format!("{}/jobs", self.base_url);
        let response = gloo_net::http::Request::get(&url)
            .send()
            .await
            .map_err(|e| e.to_string())?;

        if !response.ok() {
            return Err(format!("HTTP {}: {}", response.status(), response.status_text()));
        }

        response.json::<Vec<SimulationJob>>().await.map_err(|e| e.to_string())
    }

    /// Fetch results for a specific job
    pub async fn get_job(&self, job_id: &str) -> Result<SimulationJob, String> {
        let url = format!("{}/results/{}", self.base_url, job_id);
        let response = gloo_net::http::Request::get(&url)
            .send()
            .await
            .map_err(|e| e.to_string())?;

        if !response.ok() {
            return Err(format!("HTTP {}: {}", response.status(), response.status_text()));
        }

        #[derive(Deserialize)]
        struct ResultsResponse {
            job: SimulationJob,
        }

        let results: ResultsResponse = response.json().await.map_err(|e| e.to_string())?;
        Ok(results.job)
    }

    /// Trigger a new simulation (simplified - you'd need to handle model upload)
    pub async fn start_simulation(
        &self,
        model_data: Vec<u8>,
        model_name: &str,
        world_file: Option<&str>,
    ) -> Result<String, String> {
        use gloo_net::http::FormData;
        use web_sys::Blob;

        let form = FormData::new().map_err(|e| e.to_string())?;

        // Create blob for model file
        let blob = Blob::new_with_u8_vector_array(&wasm_bindgen::JsValue::from_f64(
            model_data.len() as f64
        )).map_err(|e| e.to_string())?;
        form.append_with_str("model", &blob).map_err(|e| e.to_string())?;

        if let Some(world) = world_file {
            // In a real implementation, you'd upload the world file too
            // For now, just set the world file name
        }

        let response = gloo_net::http::Request::post(&format!("{}/simulate", self.base_url))
            .body(Some(form))
            .unwrap()
            .send()
            .await
            .map_err(|e| e.to_string())?;

        if !response.ok() {
            return Err(format!("HTTP {}: {}", response.status(), response.status_text()));
        }

        #[derive(Deserialize)]
        struct JobResponse {
            job_id: String,
            message: String,
        }

        let job_response: JobResponse = response.json().await.map_err(|e| e.to_string())?;
        Ok(job_response.job_id)
    }

    /// Health check
    pub async fn health(&self) -> Result<serde_json::Value, String> {
        let url = format!("{}/health", self.base_url);
        let response = gloo_net::http::Request::get(&url)
            .send()
            .await
            .map_err(|e| e.to_string())?;

        if !response.ok() {
            return Err(format!("HTTP {}", response.status()));
        }

        response.json::<serde_json::Value>().await.map_err(|e| e.to_string())
    }
}

/// Client for EdgeBot licensing system (local verification)
pub struct LicensingClient;

impl LicensingClient {
    /// Check if pro features are available
    /// Returns Ok(()) if valid pro license exists, Err otherwise
    pub fn check_pro_access(&self) -> Result<(), String> {
        edgebot_licensing::verify_pro_access(None)
            .map_err(|e| e.to_string())
    }

    /// Check specifically for cloud simulation feature
    pub fn check_cloud_sim(&self) -> Result<(), String> {
        edgebot_licensing::check_cloud_sim()
            .map_err(|e| e.to_string())
    }

    /// Check specifically for optimization feature
    pub fn check_optimization(&self) -> Result<(), String> {
        edgebot_licensing::check_optimization()
            .map_err(|e| e.to_string())
    }

    /// Get detailed license info (customer ID, features, expiry)
    pub fn get_license_info(&self) -> Result<Option<LicenseInfo>, String> {
        use edgebot_licensing::LicenseError;
        
        let license_key = env::var("EDGEBOT_LICENSE_KEY")
            .ok()
            .filter(|k| !k.is_empty())
            .map(|key| {
                // Parse the license key to extract info (without verification)
                // This is a simplified version - in reality you'd want to decode and parse
                // but for dashboard display we just show that a key exists
                LicenseInfo {
                    has_key: true,
                    key_preview: format!("...{}", &key.chars().rev().take(8).collect::<String>()),
                    features: Vec::new(), // Would need full verification to get features
                    expiry: None,
                }
            });

        Ok(license_key)
    }
}

#[derive(Debug, Clone)]
pub struct LicenseInfo {
    pub has_key: bool,
    pub key_preview: String,
    pub features: Vec<String>,
    pub expiry: Option<i64>,
}

/// Model metrics from benchmarking
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelMetrics {
    pub model_name: String,
    pub inference_latency_ms: f64,
    pub memory_footprint_mb: f64,
    pub model_size_mb: f64,
    pub platform: String,
}

/// Client for querying model metrics (from benchmark results)
pub struct MetricsClient {
    results_path: String,
}

impl MetricsClient {
    pub fn new(results_path: &str) -> Self {
        Self {
            results_path: results_path.to_string(),
        }
    }

    /// Load metrics from benchmark JSON results
    /// In a real implementation, this would query a database or API
    /// For now, we'll just return sample data or read from a file in WASI
    pub async fn get_metrics(&self, model_name: &str) -> Result<Option<ModelMetrics>, String> {
        // In the browser (wasm32-unknown-unknown), we can't read files directly
        // So this would need to be served via HTTP from a backend
        // For now, return placeholder metrics
        Ok(Some(ModelMetrics {
            model_name: model_name.to_string(),
            inference_latency_ms: 15.2,
            memory_footprint_mb: 42.5,
            model_size_mb: 12.3,
            platform: "wasm32-unknown-unknown".to_string(),
        }))
    }

    /// List all available models with metrics
    pub async fn list_models(&self) -> Result<Vec<ModelMetrics>, String> {
        // Would fetch from /metrics endpoint in a full implementation
        Ok(vec![
            ModelMetrics {
                model_name: "yolov8.onnx".to_string(),
                inference_latency_ms: 12.5,
                memory_footprint_mb: 38.2,
                model_size_mb: 18.7,
                platform: "x86_64-unknown-linux-gnu".to_string(),
            },
            ModelMetrics {
                model_name: "mobilenet-v3.onnx".to_string(),
                inference_latency_ms: 8.3,
                memory_footprint_mb: 24.1,
                model_size_mb: 9.4,
                platform: "wasm32-unknown-unknown".to_string(),
            },
        ])
    }
}
