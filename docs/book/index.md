# EdgeBot AI Book

Welcome to EdgeBot AI, a Rust-based platform for deploying lightweight AI models on robots and IoT devices. This book will guide you through setup, integration, deployment, and advanced workflows.

## What is EdgeBot AI?

EdgeBot AI provides:

- **Zero-copy memory safety** using Rust's ownership model
- **Cross-platform support** (x86_64, ARM, WebAssembly)
- **ROS2 integration** for robotics ecosystems
- **Burn framework** for efficient inference
- **Simulation-ready** with Webots integration
- **Freemium model**: Free core SDK, Pro features for cloud simulation and advanced optimization

## Quick Start

### Prerequisites

- Rust 1.70+ with `rustc`, `cargo`, `rustfmt`, `clippy`
- For WASM target: `rustup target add wasm32-unknown-unknown`
- Optional: ROS2 installation for ROS2 integration
- Optional: Webots for simulation

### Installation

```bash
# Clone the repository
git clone https://github.com/edgebot-ai/edgebot-ai.git
cd edgebot-ai

# Build all crates
cargo build

# Run tests
cargo test --workspace

# Build for WASM (requires wasm32 target)
cargo build --target wasm32-unknown-unknown --release -p edgebot-wasm
```

### Your First Inference

```rust
use edgebot_core::inference::InferenceEngine;
use burn::backend::tch::TchBackend;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let device = TchBackend::Device::default();

    // Load an ONNX model
    let engine = InferenceEngine::<TchBackend>::load_onnx(
        Path::new("model.onnx"),
        &[1, 3, 224, 224],
        device,
    )?;

    // Create input tensor
    let input = burn::tensor::Tensor::<TchBackend>::random(
        [1, 3, 224, 224],
        burn::tensor::Distribution::Uniform(-1.0, 1.0),
        engine.device(),
    );

    // Run inference
    let output = engine.forward(input)?;
    println!("Output shape: {:?}", output.dims());

    Ok(())
}
```

## Project Structure

This is a Cargo workspace with multiple specialized crates:

| Crate | Purpose |
|-------|---------|
| `edgebot-core` | Core inference engine, memory safety, optimizer, task abstractions |
| `edgebot-sim` | Simulation environment with Webots integration |
| `edgebot-ros2` | ROS2 bridge for robot communication |
| `edgebot-wasm` | WebAssembly runtime for browser/IoT |
| `edgebot-sim-server` | Cloud simulation service (Actix Web API) |
| `edgebot-cli` | Command-line interface for deployment, simulation, optimization |
| `edgebot-licensing` | License verification for Pro features |
| `edgebot-dashboard` | Web dashboard (Yew/ WASM) for monitoring |

## Next Steps

- [ROS2 Integration](ros2-integration.md): Connect your robots to EdgeBot AI
- [WebAssembly Deployment](wasm-deployment.md): Deploy models to browsers and IoT devices
- [Pro Workflow](pro-workflow.md): Cloud simulation and advanced optimization
- [API Reference](reference.md): Detailed API documentation

## Getting Help

- **Documentation**: This book and `cargo doc` (see below)
- **Examples**: Check each crate's `examples/` directory
- **Issues**: Report bugs at https://github.com/edgebot-ai/edgebot-ai/issues
- **Community**: Join our Discord (link coming soon)

## Generating API Documentation

You can generate full API documentation for all crates locally:

```bash
# Generate docs for the entire workspace
cargo doc --workspace --open

# Generate docs for a specific crate
cargo doc -p edgebot-core --open

# Generate docs with all features enabled
cargo doc --workspace --all-features --open
```

The documentation will be built in `target/doc/` and opened in your browser.
