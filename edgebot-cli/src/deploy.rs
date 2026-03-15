use anyhow::{Context, Result};
use clap::Parser;
use std::path::Path;
use std::process::Command;
use ssh2::Session;
use std::io::Write;
use std::net::TcpStream;

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

/// Deploy a binary to a remote device via SSH
fn deploy_ssh(
    binary_path: &Path,
    target: &str,
    username: &str,
    password: Option<&str>,
    destination: &Path,
) -> Result<()> {
    // Establish TCP connection
    let tcp = TcpStream::connect(target)
        .with_context(|| format!("Failed to connect to {}", target))?;
    
    // Create SSH session
    let mut sess = Session::new().unwrap();
    sess.set_tcp_stream(tcp);
    sess.handshake().context("SSH handshake failed")?;
    
    // Authenticate
    if let Some(pwd) = password {
        sess.userauth_password(username, pwd)
            .context("SSH password authentication failed")?;
    } else {
        // Try agent authentication
        if !sess.userauth_agent(username).context("SSH agent authentication failed")? {
            anyhow::bail!("No authentication method succeeded");
        }
    }
    
    if !sess.authenticated() {
        anyhow::bail!("SSH authentication failed");
    }
    
    // Read binary file
    let bin_data = std::fs::read(binary_path)
        .with_context(|| format!("Failed to read binary: {:?}", binary_path))?;
    
    // Create remote directory if needed
    {
        let mut channel = sess.channel_session()
            .context("Failed to create SSH channel")?;
        let mkdir_cmd = format!("mkdir -p {}", destination.display());
        channel.exec(&mkdir_cmd)
            .context("Failed to execute mkdir command")?;
        channel.wait_close().ok();
    }
    
    // Upload binary via SCP
    let remote_path = destination.join("edgebot");
    let mut remote_file = sess.scp_send(
        remote_path.to_str().context("Invalid remote path")?,
        0o755, // permissions: rwxr-xr-x
        bin_data.len() as u64,
    ).context("SCP send failed")?;
    
    remote_file.write_all(&bin_data).context("Failed to write binary data")?;
    remote_file.flush().context("Failed to flush SCP data")?;
    remote_file.send_eof().context("Failed to send EOF")?;
    remote_file.wait_eof().ok();
    
    // Optionally set executable permissions
    {
        let mut channel = sess.channel_session()?;
        let chmod_cmd = format!("chmod +x {}", remote_path.display());
        channel.exec(&chmod_cmd)?;
        channel.wait_close().ok();
    }
    
    println!("✅ Deployed binary to {}@{}:{}", username, target, remote_path.display());
    Ok(())
}

/// Deploy via serial connection (placeholder - would use serial port library)
fn deploy_serial(_binary_path: &Path, _target: &str, _destination: &Path) -> Result<()> {
    anyhow::bail!("Serial deployment not yet implemented. Use SSH deployment instead.");
}

/// Main deploy function
pub fn run_deploy(args: DeployArgs) -> Result<()> {
    let binary_path = &args.binary;
    if !binary_path.exists() {
        anyhow::bail!("Binary file not found: {:?}", binary_path);
    }
    
    match args.method.as_str() {
        "ssh" => {
            let username = args.username.as_deref()
                .context("Username is required for SSH deployment")?;
            let password = args.password.as_deref();
            deploy_ssh(binary_path, &args.target, username, password, &args.destination)
        }
        "serial" => deploy_serial(binary_path, &args.target, &args.destination),
        _ => anyhow::bail!("Unsupported deployment method: {}", args.method),
    }
}
