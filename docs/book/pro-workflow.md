# Pro Workflow

EdgeBot AI's Pro tier ($29/month) unlocks advanced features for professional robotics deployments:

- **Cloud simulation**: Run batch simulations with hundreds of scenarios
- **Advanced model optimization**: int8/fp16 quantization, pruning, layer fusion
- **Priority support**: Direct access to EdgeBot AI engineers
- **Offline activation tokens**: Use in air-gapped environments

This guide covers license verification, activation, and using Pro features.

## License Verification System

The `edgebot-licensing` crate implements Ed25519-based license verification:

- Licenses are signed by EdgeBot AI's private key
- Supports offline activation (no phone home required)
- Includes expiry dates and feature flags
- Fast cryptographic verification using `ed25519-dalek`

### License Format

A license key is two base64-encoded parts separated by a colon:

```
signature_base64:payload_base64
```

- `signature`: Ed25519 signature over the payload
- `payload`: JSON containing:
  ```json
  {
    "customer_id": "cust_12345",
    "issued_at": 1704067200,
    "expires_at": 1706755200,
    "features": ["cloud_sim", "optimization", "priority_support"],
    "max_jobs": 1000
  }
  ```

### Setting Your License

```bash
# Set environment variable
export EDGEBOT_LICENSE_KEY="your_license_key_here"

# Or pass to commands directly
EDGEBOT_LICENSE_KEY="your_key" edgebot simulate --cloud ...
```

The EdgeBot CLI and crates automatically read this environment variable.

## Getting a Pro License

1. Visit [https://edgebot.ai/pricing](https://edgebot.ai/pricing)
2. Subscribe to Pro plan ($29/month)
3. Receive license key via email
4. Set `EDGEBOT_LICENSE_KEY` in your environment or shell profile

### Enterprise Deployments

For air-gapped environments, contact sales@edgebot.ai to obtain:

- Offline activation tokens
- Custom feature flags
- Volume discounts
- SLA guarantees

## Cloud Simulation

The cloud simulation service (`edgebot-sim-server`) allows you to:

- Upload models and run batch simulations (100+ scenarios)
- Get detailed performance metrics (FPS, memory, latency)
- Parallelize simulation workloads across multiple server instances
- Store and compare results over time

### Running Cloud Simulations

```bash
# Cloud simulation requires Pro license
edgebot simulate --model model.ebmodel --cloud --server https://sim.edgebot.ai --runs 100

# With custom world file
edgebot simulate --model model.ebmodel --world worlds/custom.wbt --cloud --server https://sim.edgebot.ai

# Output JSON for parsing
edgebot simulate --model model.ebmodel --cloud --json > results.json
```

### Cloud Simulation API

The server exposes REST endpoints:

- `POST /simulate`: Upload model and world, queue simulation job
- `GET /jobs/{id}`: Check job status
- `GET /jobs/{id}/results`: Fetch results when complete
- `GET /metrics/{model}`: Get historical metrics for a model

Example using curl:

```bash
curl -X POST https://sim.edgebot.ai/simulate \
  -F "model=@model.ebmodel" \
  -F "world=@world.wbt" \
  -F "runs=100"
```

For full API spec, see `edgebot-sim-server/API.md`.

## Advanced Model Optimization

Pro users can access advanced optimization features:

- **Quantization**: int8 (8-bit integer), fp16 (half-precision)
- **Pruning**: Magnitude-based, structured pruning
- **Layer Fusion**: Combine Conv+ReLU, MatMul+Add, etc.

### Optimizing Models

```bash
# Full optimization with all Pro features
edgebot optimize \
  --input model.onnx \
  --output model.ebmodel \
  --quantize int8 \
  --prune magnitude \
  --pruning-threshold 0.5 \
  --fuse-layers

# fp16 quantization only
edgebot optimize --input model.onnx --output model.ebmodel --quantize fp16

# Quantization-aware training (QAT) support
export EDGEBOT_QAT_MODEL="path/to/qat_model.onnx"
edgebot optimize --input model.onnx --output model.ebmodel --quantize int8 --qat
```

### Optimization Report

After optimization, a detailed report is generated:

```
Optimization Report:
--------------------
Original size: 45.2 MB
Optimized size: 10.1 MB (77.6% reduction)

Inference speedup (estimated): 2.3x
Quantization: int8
Pruning: 50% of weights removed
Layer fusion: 12 layers fused

Export path: model.ebmodel
```

The `.ebmodel` bundle contains:

- Optimized model weights (ONNX or Burn binary)
- Optimization metadata (JSON)
- Version compatibility info

### Benchmarking

Use the built-in criterion benchmarks to validate optimizations:

```bash
# Run benchmarks comparing original vs optimized
cargo bench -p edgebot-core -- --baseline model.onnx --comparison model.ebmodel

# Output JSON for pro tier analytics
cargo bench -p edgebot-core -- --output-format json > benchmark.json

# Pro tier auto-generates optimization recommendations
# at benchmark_results/inference_pro_report.json
```

## EdgeBot CLI Pro Commands

All Pro features are integrated into the `edgebot` CLI:

### `edgebot optimize`

Pro-only optimization flags:

```bash
--quantize <int8|fp16>    # Enable quantization (int8, fp16)
--prune <magnitude|structured>  # Pruning strategy
--pruning-threshold <0.0-1.0>   # Fraction to prune
--fuse-layers             # Enable layer fusion
--qat                     # Use quantization-aware training model
```

If any of these flags are used without a valid Pro license, the command will fail with an error.

### `edgebot simulate --cloud`

Cloud simulation is Pro-only. Free tier users can only run local simulations with Webots.

### `edgebot deploy`

Deployment to ARM devices is free, but advanced deployment features (fleet management, OTA updates) are Pro-only (coming soon).

## License Verification in Code

You can verify license status programmatically in your Rust applications:

```rust
use edgebot_licensing::{verify_pro_access, LicenseError, ProFeature};

fn main() -> Result<(), LicenseError> {
    // Check if Pro license is valid
    let license = verify_pro_access()?;

    println!("Customer: {}", license.customer_id);
    println!("Expires: {}", license.expires_at);
    println!("Features: {:?}", license.features);

    // Check specific feature
    if license.has_feature(ProFeature::CloudSimulation) {
        // Enable cloud features
        run_cloud_simulation();
    }

    if license.has_feature(ProFeature::AdvancedOptimization) {
        // Enable int8 quantization, pruning, etc.
        enable_advanced_optimization();
    }

    // Check if near expiry
    if license.expires_soon(Duration::from_days(7)) {
        eprintln!("Warning: License expires in {} days", license.days_remaining());
    }

    Ok(())
}
```

## License Development Workflow

During development, you can generate test licenses (debug builds only):

```rust
#[cfg(debug_assertions)]
use edgebot_licensing::generate_dev_license;

#[cfg(debug_assertions)]
fn create_test_license() {
    let license = generate_dev_license(
        "test_customer",
        vec!["cloud_sim", "optimization"],
        "your_secret_key_base64" // Use the dev private key
    ).unwrap();

    println!("Dev license: {}", license.to_string());
    // Set as EDGEBOT_LICENSE_KEY to test
}
```

**IMPORTANT**: Never commit dev licenses or private keys to production.

## Troubleshooting Licensing

| Issue | Solution |
|-------|----------|
| `Pro feature requires valid license` | Set `EDGEBOT_LICENSE_KEY` in your environment |
| `License expired` | Renew subscription at https://edgebot.ai/pricing |
| `Invalid signature` | License key malformed or tampered; contact support |
| `Feature not enabled` | Your license doesn't include that feature; check your plan |
| `No license found` | Pro commands require valid license; free tier cannot use `--cloud` or advanced `optimize` flags |

## Support Resources

- **Documentation**: This book and `cargo doc`
- **License Issues**: Contact support@edgebot.ai
- **Bugs**: https://github.com/edgebot-ai/edgebot-ai/issues
- **Feature Requests**: GitHub Issues with `enhancement` label

## Upgrading/Downgrading Plan

To change your subscription:

1. Log in to https://edgebot.ai/account
2. Select new plan
3. New license key will be emailed (existing key remains valid until expiry)

Downgrades take effect at next billing cycle; upgrades are immediate with prorated charges.

## Refund Policy

30-day money-back guarantee for first-time Pro subscribers. Contact support@edgebot.ai within 30 days of purchase for a full refund.

## Next Steps

- [ROS2 Integration](ros2-integration.md): Use Pro features with ROS2 robots
- [WebAssembly Deployment](wasm-deployment.md): Deploy optimized WASM modules
- [API Reference](reference.md): API documentation
