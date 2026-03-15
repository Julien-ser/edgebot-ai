use anyhow::{Context, Result};
use std::path::PathBuf;
use std::process::Command;
use std::fs;
use std::env;

/// Hardware platform types supported by EdgeBot CLI
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum HardwarePlatform {
    RaspberryPi,
    Stm32,
    GenericArm,
    Unknown,
}

/// ARM target triples for cross-compilation
#[derive(Debug, Clone)]
pub struct TargetTriple {
    pub triple: String,
    pub description: String,
    pub features: Vec<String>,
}

impl TargetTriple {
    pub fn new(triple: &str, description: &str, features: Vec<&str>) -> Self {
        Self {
            triple: triple.to_string(),
            description: description.to_string(),
            features: features.iter().map(|&s| s.to_string()).collect(),
        }
    }
}

/// Returns the appropriate target triple for detected hardware
pub fn get_target_for_hardware(hardware: &HardwarePlatform) -> TargetTriple {
    match hardware {
        HardwarePlatform::RaspberryPi => TargetTriple::new(
            "aarch64-unknown-linux-musl",
            "Raspberry Pi (64-bit, musl libc, static linking)",
            vec!["musl", "static"],
        ),
        HardwarePlatform::Stm32 => TargetTriple::new(
            "thumbv7em-none-eabihf",
            "STM32 (Cortex-M4/M7, no-std, hardware FPU)",
            vec!["no-std", "hard-float"],
        ),
        HardwarePlatform::GenericArm => TargetTriple::new(
            "aarch64-unknown-linux-gnu",
            "Generic ARM64 (GNU libc, dynamic linking)",
            vec!["gnu"],
        ),
        HardwarePlatform::Unknown => TargetTriple::new(
            "aarch64-unknown-linux-musl",
            "Default ARM64 (musl, safe fallback)",
            vec!["musl", "static"],
        ),
    }
}

/// Detects the hardware platform from the local system
/// (Used for auto-detection when target hardware is not specified)
pub fn detect_local_hardware() -> HardwarePlatform {
    // Check /proc/cpuinfo for Raspberry Pi
    if let Ok(cpuinfo) = fs::read_to_string("/proc/cpuinfo") {
        if cpuinfo.contains("BCM2708") || 
           cpuinfo.contains("BCM2711") || 
           cpuinfo.contains("BCM2835") || 
           cpuinfo.contains("BCM2712") ||
           cpuinfo.contains("Raspberry Pi") {
            return HardwarePlatform::RaspberryPi;
        }
    }

    // Check /proc/device-tree/compatible for STM32 or other embedded
    if let Ok(compatible) = fs::read_to_string("/proc/device-tree/compatible") {
        if compatible.contains("stm32") || compatible.contains("STM32") {
            return HardwarePlatform::Stm32;
        }
    }

    // Check environment variable override
    if let Ok(hw) = env::var("EDGEBOT_HARDWARE") {
        match hw.to_lowercase().as_str() {
            "raspberry-pi" | "rpi" | "raspberrypi" => return HardwarePlatform::RaspberryPi,
            "stm32" => return HardwarePlatform::Stm32,
            "generic-arm" | "arm" => return HardwarePlatform::GenericArm,
            _ => {}
        }
    }

    HardwarePlatform::Unknown
}

/// Build the cross-compilation command using the `cross` crate or system cross tool
pub fn build_cross_compile_command(
    target: &str,
    output: &PathBuf,
    release: bool,
    features: &[String],
    model: Option<&PathBuf>,
    static_link: bool,
    docker_image: Option<&str>,
) -> Result<Command> {
    let mut cmd = Command::new("cross");
    
    // Basic cargo build command
    cmd.arg("build");
    
    // Add target triple
    cmd.arg("--target").arg(target);
    
    // Release mode
    if release {
        cmd.arg("--release");
    }
    
    // Features
    if !features.is_empty() {
        let features_str = features.join(",");
        cmd.arg("--features").arg(features_str);
    }
    
    // Specify output binary path
    // We'll use a temp binary name and then copy to desired location
    let bin_name = "edgebot-compiled";
    cmd.arg("-p").arg("edgebot-cli");
    
    // Docker image override
    if let Some(image) = docker_image {
        cmd.arg("--docker-image").arg(image);
    }
    
    // Cross-specific options
    if static_link {
        // For musl targets, static linking is default
        // For GNU targets, we may need additional linker flags
        if target.contains("linux-gnu") {
            // Add static linking flags for glibc
            env::set_var("CARGO_TARGET_AARCH64_UNKNOWN_LINUX_GNU_LINKER", "aarch64-linux-gnu-gcc");
            env::set_var("RUSTFLAGS", "-C target-feature=+crt-static");
        }
    }
    
    Ok(cmd)
}

/// Execute the cross-compilation build
pub fn cross_compile(
    target: &str,
    output: &PathBuf,
    release: bool,
    features: &[String],
    model: Option<&PathBuf>,
    static_link: bool,
    docker_image: Option<&str>,
    verbose: bool,
) -> Result<()> {
    // First, check if cross is installed
    if !Command::new("cross").output().is_ok() {
        anyhow::bail!("Cross compiler not found. Install with: `cargo install cross` or use Docker manually.");
    }
    
    // Build command
    let mut cmd = build_cross_compile_command(
        target,
        output,
        release,
        features,
        model,
        static_link,
        docker_image,
    )?;
    
    if verbose {
        eprintln!("Running: {:?}", cmd);
    }
    
    let output = cmd.output().context("Failed to execute cross build")?;
    
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        anyhow::bail!("Cross compilation failed:\n{}", stderr);
    }
    
    // Determine the compiled binary location based on target and profile
    let profile = if release { "release" } else { "debug" };
    let compiled_binary = format!("target/{}/{}/{}", target, profile, "edgebot-cli");
    
    // Copy to desired output location
    let source_path = PathBuf::from(compiled_binary);
    if !source_path.exists() {
        anyhow::bail!("Compiled binary not found at: {:?}", source_path);
    }
    
    fs::copy(&source_path, output).context("Failed to copy compiled binary to output location")?;
    
    if verbose {
        eprintln!("✅ Compiled binary saved to: {:?}", output);
    }
    
    // If a model was specified, embed it into the binary or copy alongside
    if let Some(model_path) = model {
        if model_path.exists() {
            let model_output = output.with_extension("ebmodel");
            fs::copy(model_path, &model_output).context("Failed to copy model file")?;
            if verbose {
                eprintln!("✅ Model saved to: {:?}", model_output);
            }
        } else {
            anyhow::bail!("Model file not found: {:?}", model_path);
        }
    }
    
    Ok(())
}

/// Compile for all supported ARM targets (batch compilation)
pub fn compile_all_targets(
    output_dir: &PathBuf,
    release: bool,
    features: &[String],
    model: Option<&PathBuf>,
    verbose: bool,
) -> Result<()> {
    let targets = vec![
        ("aarch64-unknown-linux-musl", HardwarePlatform::RaspberryPi),  // RPi 64-bit
        ("armv7-unknown-linux-musleabihf", HardwarePlatform::GenericArm), // ARMv7 with hard float
        ("thumbv7em-none-eabihf", HardwarePlatform::Stm32),  // STM32 Cortex-M4/M7
    ];
    
    for (target, hardware) in targets {
        eprintln!("🔨 Compiling for {} ({:?})...", target, hardware);
        
        let output_file = output_dir.join(format!("edgebot-{}", 
            match hardware {
                HardwarePlatform::RaspberryPi => "rpi64",
                HardwarePlatform::Stm32 => "stm32",
                HardwarePlatform::GenericArm => "armv7",
                _ => "generic",
            }));
        
        if let Err(e) = cross_compile(
            target,
            &output_file,
            release,
            features,
            model,
            true, // static_link
            None, // docker_image
            verbose,
        ) {
            eprintln!("⚠️  Failed to compile for {}: {}", target, e);
            continue;
        }
        
        eprintln!("✅ Successfully compiled for {}", target);
    }
    
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_detect_local_hardware() {
        // On non-Raspberry Pi systems, should return Unknown
        let hw = detect_local_hardware();
        // We can't assert specific values since it depends on the system
        // But we can verify the function runs without panicking
        assert!(matches!(hw, HardwarePlatform::Unknown | HardwarePlatform::RaspberryPi | HardwarePlatform::Stm32 | HardwarePlatform::GenericArm));
    }
    
    #[test]
    fn test_get_target_for_hardware() {
        let rpi_target = get_target_for_hardware(&HardwarePlatform::RaspberryPi);
        assert_eq!(rpi_target.triple, "aarch64-unknown-linux-musl");
        assert!(rpi_target.features.contains(&"musl".to_string()));
        
        let stm32_target = get_target_for_hardware(&HardwarePlatform::Stm32);
        assert_eq!(stm32_target.triple, "thumbv7em-none-eabihf");
        assert!(stm32_target.features.contains(&"no-std".to_string()));
    }
}
