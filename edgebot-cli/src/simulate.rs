use clap::Parser;

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

    /// Number of simulation runs
    #[arg(short, long, default_value = "1")]
    pub runs: u32,

    /// Output results as JSON
    #[arg(short, long)]
    pub json: bool,
}
