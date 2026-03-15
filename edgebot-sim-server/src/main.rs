use actix_multipart::Multipart;
use actix_web::{web, App, HttpResponse, HttpServer, Result as ActixResult};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use tempfile::TempDir;
use tracing::{info, error,instrument};
use uuid::Uuid;

use edgebot_sim::webots::{Supervisor, WebotsError};
use edgebot_core::inference::InferenceEngine;
use burn::backend::tch::TchBackend;

mod errors;
use errors::ServerError;

/// Job status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum JobStatus {
    Running,
    Completed,
    Failed,
}

/// Simulation job details
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SimulationJob {
    pub id: String,
    pub status: JobStatus,
    pub model_name: String,
    pub world_file: Option<String>,
    pub scenes: Vec<String>,
    pub metrics: Option<SimulationMetrics>,
    pub error: Option<String>,
}

/// Performance metrics collected from simulation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SimulationMetrics {
    pub fps: f64,
    pub memory_peak_mb: f64,
    pub inference_latency_ms: f64,
    pub total_frames: u32,
    pub scene_name: String,
}

/// Request to start a simulation
#[derive(Debug, Deserialize)]
pub struct SimulateRequest {
    pub world_file: String,
    pub scenes: Vec<String>,
    pub model_name: String,
}

/// Response with job ID
#[derive(Debug, Serialize)]
pub struct JobResponse {
    pub job_id: String,
    pub message: String,
}

/// Response with simulation results
#[derive(Debug, Serialize)]
pub struct ResultsResponse {
    pub job: SimulationJob,
}

/// Shared application state
#[derive(Clone)]
struct AppState {
    jobs: Arc<Mutex<HashMap<String, SimulationJob>>>,
    models_dir: PathBuf,
}

/// Server configuration
#[derive(Deserialize)]
pub struct ServerConfig {
    pub host: String,
    pub port: u16,
    pub models_dir: PathBuf,
}

impl ServerConfig {
    pub fn from_env() -> Self {
        let host = std::env::var("HOST").unwrap_or_else(|_| "0.0.0.0".to_string());
        let port = std::env::var("PORT")
            .ok()
            .and_then(|p| p.parse().ok())
            .unwrap_or(8080);
        let models_dir = std::env::var("MODELS_DIR")
            .map(PathBuf::from)
            .unwrap_or_else(|_| PathBuf::from("./models"));

        Self {
            host,
            port,
            models_dir,
        }
    }
}

/// Handles model upload and simulation start
#[instrument(skip(state, payload), fields(job_id))]
async fn simulate_handler(
    state: web::Data<AppState>,
    mut payload: Multipart,
) -> ActixResult<HttpResponse> {
    info!("Received simulation request");

    let mut temp_dir = match TempDir::new() {
        Ok(dir) => dir,
        Err(e) => {
            error!("Failed to create temp dir: {}", e);
            return Ok(HttpResponse::InternalServerError().json(ServerError::new("Failed to create temp directory")));
        }
    };

    let mut uploaded_model_path = None;

    // Process multipart form data
    while let Ok(Some(field)) = payload.try_next().await {
        let content_disposition = field.content_disposition();
        let name = content_disposition
            .get_name()
            .map(|s| s.to_string())
            .unwrap_or_else(|| "unknown".to_string());

        if name == "model" {
            let file_name = field
                .content_disposition()
                .get_filename()
                .map(|s| s.to_string())
                .unwrap_or_else(|| format!("model_{}.onnx", Uuid::new_v4()));

            let file_path = temp_dir.path().join(&file_name);

            // Write file to temp directory
            if let Err(e) = actix_multipart::save::save_async(field, file_path.clone()).await {
                error!("Failed to save uploaded file: {}", e);
                return Ok(HttpResponse::BadRequest().json(ServerError::new("Failed to save uploaded file")));
            }

            uploaded_model_path = Some(file_path);
            info!("Model uploaded to: {:?}", file_path);
        } else if name == "world_file" {
            // Handle world file upload if provided
            // For now, we'll use provided world file path from request body
        }
    }

    let model_path = match uploaded_model_path {
        Some(path) => path,
        None => return Ok(HttpResponse::BadRequest().json(ServerError::new("No model file provided"))),
    };

    // Generate job ID
    let job_id = Uuid::new_v4().to_string();

    // Extract request parameters from query or body
    // For simplicity, we'll use default values or read from form
    // A more complete implementation would parse the full request body
    let world_file = "worlds/default.wbt".to_string();
    let scenes = vec!["scene1".to_string()];

    // Create job entry
    let job = SimulationJob {
        id: job_id.clone(),
        status: JobStatus::Running,
        model_name: "uploaded_model".to_string(),
        world_file: Some(world_file),
        scenes: scenes.clone(),
        metrics: None,
        error: None,
    };

    // Store job
    state.jobs.lock().unwrap().insert(job_id.clone(), job.clone());

    // Spawn async task to run simulation
    let state_clone = state.clone();
    let model_path_clone = model_path.clone();
    let scenes_clone = scenes.clone();

    actix_web::rt::spawn(async move {
        match run_simulation(&state_clone, &job_id, &model_path_clone, &world_file, &scenes_clone).await {
            Ok(_) => info!("Simulation {} completed", job_id),
            Err(e) => error!("Simulation {} failed: {}", job_id, e),
        }
    });

    let response = JobResponse {
        job_id,
        message: "Simulation started".to_string(),
    };

    Ok(HttpResponse::Accepted().json(response))
}

/// Runs the simulation in a background task
async fn run_simulation(
    state: &AppState,
    job_id: &str,
    model_path: &PathBuf,
    world_file: &str,
    scenes: &[String],
) -> Result<(), ServerError> {
    let mut jobs = state.jobs.lock().unwrap();
    let job = jobs.get_mut(job_id).ok_or_else(|| ServerError::new("Job not found"))?;

    // Determine if we need headless mode (cloud environment)
    let headless = std::env::var("WEBOTS_HEADLESS")
        .map(|v| v == "true" || v == "1")
        .unwrap_or(true);

    // Launch Webots supervisor
    let supervisor = match Supervisor::launch(world_file, headless) {
        Ok(s) => s,
        Err(e) => {
            job.status = JobStatus::Failed;
            job.error = Some(format!("Failed to launch Webots: {}", e));
            return Err(ServerError::new(&format!("Webots launch failed: {}", e)));
        }
    };

    let mut total_fps = 0.0;
    let mut total_memory_mb = 0.0;
    let mut total_inference_latency = 0.0;
    let mut frame_count = 0;

    // Load the model (simplified - would need integration with edgebot-core)
    // For now, we'll simulate inference

    // Run simulation for each scene
    for scene in scenes {
        info!("Running scene: {}", scene);

        // In a full implementation:
        // 1. Load scene into Webots
        // 2. Load model using edgebot-core
        // 3. Run simulation loop, collecting metrics
        // 4. Record performance data

        // Simplified simulation loop
        let simulation_steps = 1000;
        let mut scene_fps = 0.0;
        let mut scene_memory = 0.0;
        let mut scene_latency = 0.0;

        for step in 0..simulation_steps {
            match supervisor.step(32) {
                Ok(_) => {
                    // Simulate processing
                    // In real implementation:
                    // - Get sensor data (camera, lidar)
                    // - Run inference
                    // - Apply motor commands

                    // For metrics, we're simulating data
                    let sim_fps = 30.0; // Simulated FPS
                    let sim_memory = 50.0; // Simulated memory in MB
                    let sim_latency = 15.0; // Simulated inference latency in ms

                    scene_fps += sim_fps;
                    scene_memory += sim_memory;
                    scene_latency += sim_latency;
                    frame_count += 1;
                }
                Err(e) => {
                    error!("Simulation step failed at step {}: {}", step, e);
                    break;
                }
            }
        }

        if frame_count > 0 {
            scene_fps /= simulation_steps as f64;
            scene_memory /= simulation_steps as f64;
            scene_latency /= simulation_steps as f64;
        }

        total_fps += scene_fps;
        total_memory_mb += scene_memory;
        total_inference_latency += scene_latency;

        info!("Scene {} completed: FPS={:.2}, Memory={:.2}MB, Latency={:.2}ms",
              scene, scene_fps, scene_memory, scene_latency);
    }

    // Calculate averages
    let avg_fps = if !scenes.is_empty() {
        total_fps / scenes.len() as f64
    } else {
        0.0
    };

    let avg_memory = if !scenes.is_empty() {
        total_memory_mb / scenes.len() as f64
    } else {
        0.0
    };

    let avg_latency = if frame_count > 0 {
        total_inference_latency / frame_count as f64
    } else {
        0.0
    };

    // Update job with metrics
    job.status = JobStatus::Completed;
    job.metrics = Some(SimulationMetrics {
        fps: avg_fps,
        memory_peak_mb: avg_memory,
        inference_latency_ms: avg_latency,
        total_frames: frame_count,
        scene_name: scenes.first().unwrap_or(&"default".to_string()).clone(),
    });

    drop(jobs); // Release lock

    info!("Simulation {} finished successfully. Metrics: FPS={:.2}, Memory={:.2}MB, Latency={:.2}ms",
          job_id, avg_fps, avg_memory, avg_latency);

    Ok(())
}

/// Handles retrieving simulation results
async fn results_handler(
    state: web::Data<AppState>,
    job_id: web::Path<String>,
) -> ActixResult<HttpResponse> {
    let jobs = state.jobs.lock().unwrap();
    let job = match jobs.get(&job_id) {
        Some(job) => job,
        None => return Ok(HttpResponse::NotFound().json(ServerError::new("Job not found"))),
    };

    let response = ResultsResponse {
        job: job.clone(),
    };

    Ok(HttpResponse::Ok().json(response))
}

/// Lists all jobs
async fn list_jobs_handler(
    state: web::Data<AppState>,
) -> ActixResult<HttpResponse> {
    let jobs = state.jobs.lock().unwrap();
    let all_jobs: Vec<SimulationJob> = jobs.values().cloned().collect();

    Ok(HttpResponse::Ok().json(all_jobs))
}

/// Health check endpoint
async fn health_handler() -> ActixResult<HttpResponse> {
    Ok(HttpResponse::Ok().json(serde_json::json!({
        "status": "healthy",
        "service": "edgebot-sim-server",
        "timestamp": chrono::Utc::now().to_rfc3339()
    })))
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    // Initialize tracing
    tracing_subscriber::fmt::init();

    let config = ServerConfig::from_env();

    // Ensure models directory exists
    std::fs::create_dir_all(&config.models_dir).expect("Failed to create models directory");

    let app_state = AppState {
        jobs: Arc::new(Mutex::new(HashMap::new())),
        models_dir: config.models_dir.clone(),
    };

    info!("Starting EdgeBot Simulation Server on {}:{}", config.host, config.port);
    info!("Models directory: {:?}", config.models_dir);

    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(app_state.clone()))
            .route("/health", web::get().to(health_handler))
            .route("/simulate", web::post().to(simulate_handler))
            .route("/results/{job_id}", web::get().to(results_handler))
            .route("/jobs", web::get().to(list_jobs_handler))
    })
    .bind((config.host.as_str(), config.port))?
    .run()
    .await
}