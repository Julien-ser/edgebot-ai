use anyhow::{Context, Result};
use clap::Parser;
use std::path::Path;
use std::time::Instant;
use edgebot_sim::webots::{Supervisor, Robot, WebotsError};

#[derive(Parser)]
pub struct SimulateArgs {
    /// Model file to test
    #[arg(short, long)]
    pub model: std::path::PathBuf,

    /// Simulation world file
    #[arg(short, long)]
    pub world: Option<std::path::PathBuf>,

    /// Cloud simulation (use remote server)
    #[arg(short, long)]
    pub cloud: bool,

    /// Cloud server URL (default: http://localhost:8080)
    #[arg(short, long, default_value = "http://localhost:8080")]
    pub server: String,

    /// Number of simulation runs
    #[arg(short, long, default_value = "1")]
    pub runs: u32,

    /// Output results as JSON
    #[arg(short, long)]
    pub json: bool,

    /// Timestep between simulation steps (ms)
    #[arg(short, long, default_value = "32")]
    pub timestep: i32,
}

/// Run local simulation using Webots
fn run_local_simulation(args: &SimulateArgs) -> Result<SimulationResult> {
    let world_path = args.world.as_deref()
        .context("World file is required for local simulation")?;
    
    if !world_path.exists() {
        anyhow::bail!("World file not found: {:?}", world_path);
    }
    
    // Launch Webots supervisor
    let mut supervisor = Supervisor::launch(
        world_path.to_str().context("Invalid world path")?,
        true, // headless mode
    ).context("Failed to launch Webots")?;
    
    // Spawn robot (assuming default robot prototype exists in world)
    let robot = supervisor.spawn_robot("EdgeBot", "edgebot_cli")
        .context("Failed to spawn robot")?;
    
    // Get camera device (if available)
    let camera = match robot.get_device("camera") {
        Ok(dev) => Some(dev.as_camera().context("Failed to get camera")?),
        Err(_) => None,
    };
    
    if let Some(camera) = &camera {
        camera.enable(args.timestep).context("Failed to enable camera")?;
    }
    
    // Load model for inference (simplified - would use edgebot-core)
    println!("Loading model: {:?}", args.model);
    // TODO: Integrate with edgebot-core InferenceEngine
    
    let mut steps = 0;
    let mut inference_times = Vec::new();
    let start_time = Instant::now();
    
    // Run simulation loop
    for run in 0..args.runs {
        println!("=== Run {}/{} ===", run + 1, args.runs);
        steps = 0;
        
        loop {
            // Step simulation
            if supervisor.step(args.timestep).is_err() {
                break; // Simulation ended
            }
            steps += 1;
            
            // Get camera image and run inference
            if let Some(camera) = &camera {
                let img_start = Instant::now();
                let _image = camera.get_image()
                    .context("Failed to get camera image")?;
                let img_time = img_start.elapsed();
                inference_times.push(img_time.as_secs_f64() * 1000.0); // ms
                
                // TODO: Run inference with loaded model
                // let output = inference_engine.forward(tensor)?;
            }
        }
        
        println!("Completed {} steps", steps);
    }
    
    let total_time = start_time.elapsed();
    
    Ok(SimulationResult {
        total_runs: args.runs,
        total_steps: steps * args.runs,
        total_time_secs: total_time.as_secs_f64(),
        avg_inference_ms: if inference_times.is_empty() {
            0.0
        } else {
            inference_times.iter().sum::<f64>() / inference_times.len() as f64
        },
    })
}

/// Run cloud simulation by sending request to edgebot-sim-server
async fn run_cloud_simulation(args: &SimulateArgs) -> Result<SimulationResult> {
    let client = reqwest::Client::new();
    
    // Prepare multipart form with model file
    let model_file = std::fs::read(&args.model)
        .with_context(|| format!("Failed to read model: {:?}", args.model))?;
    
    let world_data = if let Some(world_path) = &args.world {
        Some(std::fs::read(world_path)?)
    } else {
        None
    };
    
    let response = client
        .post(&format!("{}/simulate", args.server))
        .multipart(
            reqwest::multipart::Form::new()
                .text("runs", args.runs.to_string())
                .text("timestep", args.timestep.to_string())
                .file(&args.model, "model")
                .map(|f| f.file_name("model.ebmodel"))
                .map_err(|e| anyhow::anyhow!("Failed to attach model: {}", e))?
        )
        .send()
        .await
        .context("Failed to send simulation request")?;
    
    if !response.status().is_success() {
        let status = response.status();
        let text = response.text().await.unwrap_or_default();
        anyhow::bail!("Simulation request failed ({}): {}", status, text);
    }
    
    // Parse result
    let result: SimulationResult = response
        .json()
        .await
        .context("Failed to parse simulation result")?;
    
    Ok(result)
}

/// Simulation result metrics
#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct SimulationResult {
    pub total_runs: u32,
    pub total_steps: u64,
    pub total_time_secs: f64,
    pub avg_inference_ms: f64,
}

/// Main simulate function
pub async fn run_simulate(args: SimulateArgs) -> Result<()> {
    // Check pro license for cloud simulation
    if args.cloud {
        edgebot_licensing::check_cloud_sim()
            .context("Cloud simulation is a pro feature. Set EDGEBOT_LICENSE_KEY environment variable with a valid pro license.")?;
    }
    
    let result = if args.cloud {
        run_cloud_simulation(&args).await?
    } else {
        run_local_simulation(&args)?
    };
    
    if args.json {
        println!("{}", serde_json::to_string_pretty(&result)?);
    } else {
        println!("=== Simulation Results ===");
        println!("Runs: {}", result.total_runs);
        println!("Total steps: {}", result.total_steps);
        println!("Total time: {:.2}s", result.total_time_secs);
        println!("Avg inference: {:.2}ms", result.avg_inference_ms);
        println!("Steps/sec: {:.1}", result.total_steps as f64 / result.total_time_secs);
    }
    
    Ok(())
}
