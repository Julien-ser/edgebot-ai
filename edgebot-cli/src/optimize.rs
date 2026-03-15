use clap::Parser;

#[derive(Parser)]
pub struct OptimizeArgs {
    /// Input model file
    #[arg(short, long)]
    pub input: std::path::PathBuf,

    /// Output optimized model
    #[arg(short, long)]
    pub output: std::path::PathBuf,

    /// Quantization: int8, fp16, none
    #[arg(short, long, default_value = "none")]
    pub quantize: String,

    /// Pruning: none, magnitude, structured
    #[arg(short, long, default_value = "none")]
    pub prune: String,

    /// Pruning threshold (0.0-1.0)
    #[arg(short, long)]
    pub pruning_threshold: Option<f32>,

    /// Enable layer fusion
    #[arg(short, long)]
    pub fuse_layers: bool,

    /// Target device (cpu, cuda)
    #[arg(short, long, default_value = "cpu")]
    pub device: String,
}
