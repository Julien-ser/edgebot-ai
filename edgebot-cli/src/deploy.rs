use clap::Parser;

#[derive(Parser)]
pub struct DeployArgs {
    /// Binary file to deploy
    #[arg(short, long)]
    pub binary: std::path::PathBuf,

    /// Target device address (IP or serial port)
    #[arg(short, long)]
    pub target: String,

    /// Deployment method (ssh, serial)
    #[arg(short, long, default_value = "ssh")]
    pub method: String,

    /// Remote destination path
    #[arg(short, long, default_value = "/opt/edgebot/")]
    pub destination: std::path::PathBuf,

    /// SSH username
    #[arg(short, long)]
    pub username: Option<String>,

    /// SSH password (use SSH agent if not provided)
    #[arg(short, long)]
    pub password: Option<String>,
}
