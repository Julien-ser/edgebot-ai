use std::path::PathBuf;

use burn::{
    backend::{Backend, Autocast},
    nn::{Linear, LinearConfig, Sequential},
    tensor::Tensor,
};
use criterion::{black_box, criterion_group, criterion_main, BatchSize, BenchmarkId, Criterion};
use rand::Rng;
use burn::backend::tch::TchBackend;

// Type alias for the benchmark backend
type TestBackend = Autocast<TchBackend>;

/// Model configuration for benchmarking
#[derive(Debug, Clone, Copy, Clone)]
struct ModelConfig {
    name: &'static str,
    input_features: usize,
    output_features: usize,
    hidden_layers: usize,
    layer_size: usize,
}

/// Create a synthetic sequential model
fn create_model<B: Backend>(config: ModelConfig) -> Sequential<B> {
    let mut seq = Sequential::new();
    
    // Input layer
    let input_layer = Linear::new(LinearConfig::new(config.input_features, config.layer_size));
    seq = seq.push(input_layer);
    
    // Hidden layers
    for _ in 1..config.hidden_layers {
        let hidden = Linear::new(LinearConfig::new(config.layer_size, config.layer_size));
        seq = seq.push(hidden);
    }
    
    // Output layer
    let output = Linear::new(LinearConfig::new(config.layer_size, config.output_features));
    seq = seq.push(output);
    
    seq
}

/// Generate random input tensor
fn generate_input<B: Backend>(batch_size: usize, input_features: usize, device: &B::Device) -> Tensor<B> {
    let total = batch_size * input_features;
    let mut rng = rand::thread_rng();
    let data: Vec<f32> = (0..total).map(|_| rng.gen_range(-1.0..1.0)).collect();
    
    Tensor::from_data(data.as_slice().into(), device)
}

/// Benchmark inference latency for a given configuration
fn bench_inference<B: Backend>(c: &mut Criterion, config: ModelConfig, batch_size: usize) {
    let device = B::Device::default();
    let model = create_model::<B>(config);
    
    // Warm-up run
    let warm_input = generate_input::<B>(1, config.input_features, &device);
    for _ in 0..3 {
        black_box(model.forward(warm_input.clone()));
    }
    
    let input = generate_input::<B>(batch_size, config.input_features, &device);
    let input_clone = input.clone();
    
    c.bench_with_input(
        BenchmarkId::new("inference_latency", format!("{}-batch{}", config.name, batch_size)),
        &input_clone,
        |b, input| {
            b.iter_batched(
                || input.clone(),
                |inp| black_box(model.forward(inp)),
                BatchSize::SmallInput,
            )
        },
    );
}

/// Estimate model parameter count (for reporting)
fn estimate_params(config: &ModelConfig) -> usize {
    let mut total = 0;
    let mut prev = config.input_features;
    
    for i in 0..config.hidden_layers + 1 {
        let curr = if i < config.hidden_layers {
            config.layer_size
        } else {
            config.output_features
        };
        total += prev * curr + curr; // weights + biases
        prev = curr;
    }
    
    total
}

/// Generate pro tier optimization report as JSON
fn generate_pro_report(
    results_dir: &str,
    model_configs: &[ModelConfig],
    batch_sizes: &[usize],
) -> std::io::Result<()> {
    use serde_json::json;
    
    // Create results directory if needed
    std::fs::create_dir_all(results_dir)?;
    
    // Generate recommendations based on model configurations
    let recommendations: Vec<serde_json::Value> = model_configs.iter()
        .map(|config| {
            let params = estimate_params(config);
            let size_kb = (params * 4) / 1024; // f32 = 4 bytes
            json!({
                "model": config.name,
                "parameters": params,
                "estimated_size_kb": size_kb,
                "recommended_quantization": if params > 1_000_000 { "int8" } else { "f16" },
                "recommended_pruning_threshold": if params > 5_000_000 { 0.3 } else { 0.0 },
                "optimal_batch_sizes": batch_sizes.iter()
                    .filter(|&&bs| bs * config.input_features * 4 < 1024 * 1024) // < 1MB input
                    .collect::<Vec<&usize>>(),
                "target_hardware": ["raspberry-pi-4", "jetson-nano", "stm32"],
            })
        })
        .collect();
    
    let report = json!({
        "edgebot_version": env!("CARGO_PKG_VERSION"),
        "backend": "Autocast (Tch)",
        "models": recommendations,
        "optimization_tips": [
            "Quantize large models (>1M params) to int8 for 75% memory reduction",
            "Use batch size 1-4 for real-time inference on edge devices",
            "Apply layer fusion for GPU targets (CUDA, Metal)",
            "Consider pruning models with >5M parameters at 30% sparsity",
        ]
    });
    
    let path = format!("{}/inference_pro_report.json", results_dir);
    let content = serde_json::to_string_pretty(&report)?;
    std::fs::write(&path, content)?;
    
    println!("✅ Pro tier optimization report generated: {}", path);
    Ok(())
}

/// Main benchmark suite
fn inference_benchmarks(c: &mut Criterion) {
    println!("\n🚀 EdgeBot AI Inference Benchmark Suite");
    println!("=========================================\n");
    
    // Define configurations
    let model_configs = vec![
        ModelConfig {
            name: "tiny",
            input_features: 64,
            output_features: 10,
            hidden_layers: 1,
            layer_size: 32,
        },
        ModelConfig {
            name: "small",
            input_features: 128,
            output_features: 32,
            hidden_layers: 2,
            layer_size: 64,
        },
        ModelConfig {
            name: "medium",
            input_features: 256,
            output_features: 64,
            hidden_layers: 3,
            layer_size: 128,
        },
    ];
    
    let batch_sizes = vec![1, 2, 4, 8];
    
    // Run benchmarks
    for config in model_configs.iter() {
        println!("📊 Benchmarking {} ({} params)", 
            config.name, 
            estimate_params(config)
        );
        
        for &batch in &batch_sizes {
            bench_inference::<TestBackend>(c, *config, batch);
        }
    }
    
    // Generate pro report
    if let Err(e) = generate_pro_report("benchmark_results", &model_configs, &batch_sizes) {
        eprintln!("⚠️  Failed to generate pro report: {}", e);
    }
}

criterion_group!(benches, inference_benchmarks);
criterion_main!(benches);
