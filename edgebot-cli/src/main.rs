mod compile;
mod deploy;
mod simulate;
mod optimize;

use clap::Parser;
use compile::{compile_all_targets, cross_compile, detect_local_hardware, HardwarePlatform};
use std::path::PathBuf;

#[derive(Parser)]
#[command(name = "edgebot")]
#[command(about = "EdgeBot AI CLI - Deploy, simulate, optimize, and compile AI models for edge devices", long_about = None)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,

    /// Verbose output
    #[arg(short, long, global = true)]
    pub verbose: bool,
}

#[derive(clap::Subcommand)]
pub enum Commands {
    /// Compile model for embedded ARM targets (Raspberry Pi, STM32)
    Compile(compile::CompileArgs),

    /// Deploy binary to device via serial/SSH
    Deploy(deploy::DeployArgs),

    /// Run simulation (local or cloud)
    Simulate(simulate::SimulateArgs),

    /// Optimize model for edge deployment
    Optimize(optimize::OptimizeArgs),
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();
    
    match cli.command {
        Commands::Compile(ref args) => {
            // Determine target
            let target = if let Some(ref t) = args.target {
                t.clone()
            } else {
                // Auto-detect hardware
                let hardware = detect_local_hardware();
                let triple = compile::get_target_for_hardware(&hardware);
                println!("Auto-detected hardware: {:?}", hardware);
                println!("Using target: {}", triple.triple);
                triple.triple
            };
            
            // If hardware is specified, validate or override target
            let (final_target, hardware) = if let Some(ref hw) = args.hardware {
                let hardware = match hw.to_lowercase().as_str() {
                    "raspberry-pi" | "rpi" | "raspberrypi" => HardwarePlatform::RaspberryPi,
                    "stm32" => HardwarePlatform::Stm32,
                    "generic-arm" | "arm" => HardwarePlatform::GenericArm,
                    _ => HardwarePlatform::Unknown,
                };
                let triple = compile::get_target_for_hardware(&hardware);
                (triple.triple, hardware)
            } else {
                let hardware = detect_local_hardware();
                (target.clone(), hardware)
            };
            
            // Handle --all flag or specific target compilation
            if args.target.as_deref() == Some("all") {
                println!("Compiling for all supported ARM targets...");
                let output_dir = args.output.parent().unwrap_or_else(|| std::path::Path::new("."));
                compile_all_targets(
                    &PathBuf::from(output_dir),
                    args.release,
                    &args.features,
                    args.model.as_ref(),
                    cli.verbose,
                )?;
            } else {
                // Single target compilation
                println!("Compiling for target: {}", final_target);
                cross_compile(
                    &final_target,
                    &args.output,
                    args.release,
                    &args.features,
                    args.model.as_ref().map(|m| m.as_path()),
                    args.static_link,
                    args.docker_image.as_deref(),
                    cli.verbose,
                )?;
            }
            
            println!("✅ Compilation complete!");
        }
        Commands::Deploy(ref args) => {
            deploy::run_deploy(args.clone())?;
        }
        Commands::Simulate(ref args) => {
            // Simulate is async, need to block on runtime
            let rt = tokio::runtime::Runtime::new()?;
            rt.block_on(simulate::run_simulate(args.clone()))?;
        }
        Commands::Optimize(ref args) => {
            optimize::run_optimize(args.clone())?;
        }
    }
    
    Ok(())
}
