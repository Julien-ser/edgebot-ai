use clap::Parser;
use edgebot_core::optimizer::{Optimizer, OptimizationConfig, QuantizationMethod, PruningStrategy};
use std::path::PathBuf;
use tracing::{info, error};
use burn::backend::tch::TchBackend;

/// EdgeBot AI Model Optimizer
///
/// Optimizes neural network models for edge deployment with quantization,
/// pruning, and layer fusion.
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Input model path (ONNX or .bin format)
    #[arg(short, long)]
    input: PathBuf,

    /// Output optimized model path (.ebmodel format)
    #[arg(short, long)]
    output: PathBuf,

    /// Quantization method: none, int8, fp16
    #[arg(short, long, default_value = "none")]
    quantize: QuantizationMethod,

    /// Pruning strategy: none, magnitude, structured
    #[arg(short, long, default_value = "none")]
    prune: PruningStrategy,

    /// Pruning threshold (0.0-1.0) - fraction of weights to prune
    #[arg(short, long, default_value = "0.5")]
    pruning_threshold: f32,

    /// Enable layer fusion
    #[arg(short, long)]
    fuse_layers: bool,

    /// Target device: cpu, cuda
    #[arg(short, long, default_value = "cpu")]
    device: String,

    /// Verbose logging
    #[arg(short, long)]
    verbose: bool,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();

    // Initialize logger
    if args.verbose {
        std::env::set_var("RUST_LOG", "info,debug");
    } else {
        std::env::set_var("RUST_LOG", "warn");
    }
    env_logger::init();

    info!("Starting EdgeBot model optimization");
    info!("Input: {}", args.input.display());
    info!("Output: {}", args.output.display());
    info!("Quantization: {:?}", args.quantize);
    info!("Pruning: {:?} (threshold: {})", args.prune, args.pruning_threshold);
    info!("Layer fusion: {}", args.fuse_layers);
    info!("Target device: {}", args.device);

    // Validate inputs
    if !args.input.exists() {
        error!("Input model not found: {}", args.input.display());
        std::process::exit(1);
    }

    if args.pruning_threshold < 0.0 || args.pruning_threshold > 1.0 {
        error!("Pruning threshold must be between 0.0 and 1.0");
        std::process::exit(1);
    }

    // Build optimization configuration
    let config = OptimizationConfig {
        quantization: args.quantize,
        pruning: if args.prune == PruningStrategy::None { None } else { Some(args.prune) },
        pruning_threshold: args.pruning_threshold,
        layer_fusion: args.fuse_layers,
        target_device: args.device,
    };

    // Run optimization
    let optimizer = Optimizer::<TchBackend>::new(config);
    match optimizer.optimize_model(&args.input, &args.output) {
        Ok(_) => {
            info!("Optimization completed successfully!");
            info!("Optimized model saved to: {}", args.output.display());
            Ok(())
        }
        Err(e) => {
            error!("Optimization failed: {}", e);
            Err(e.into())
        }
    }
}
