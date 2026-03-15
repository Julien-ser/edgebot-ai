# edgebot-ai
**Mission:** Build EdgeBot AI, a platform providing Rust-based toolkits for deploying lightweight AI models on robots and IoT devices. Users simulate, compile to WebAssembly or native binaries, and deploy models for tasks like object detection or pathfinding with zero-copy memory safety.

Target hobbyists, robotics interns (like those using ROS2/Webots), and small labs needing fast, secure edge inference without Python's overhead. Monetize via freemium: free core SDK, $29/month pro for cloud sims and optimizations.

## Why Rust Fits
Rust's ownership model prevents crashes in concurrent robotics code, outperforming C++ in safety while matching speed—key for embedded systems.

Frameworks like Burn enable efficient model training/inference, positioning it for edge AI growth.

Existing successes in blockchain/IoT show Rust scales to production.

## Market Opportunity
AI robotics market hits $20B+ by 2027; edge inference demand surges for low-latency apps.

Niche: AMP Robotics uses Rust for real-time sorting; expand to consumer/hobby bots amid Rust's 2026 adoption push.

Competitors like TensorFlow Lite lack Rust's native safety; your edge: seamless ROS2 integration.

## Phase 1: Planning & Setup
- [x] Task 1: Define Rust project architecture and crate structure for edgebot-ai monorepo using Cargo workspaces, with crates for core inference (`edgebot-core`), simulation (`edgebot-sim`), ROS2 integration (`edgebot-ros2`), and WASM compilation (`edgebot-wasm`). Deliverable: Complete `Cargo.toml` files and workspace configuration.
- [x] Task 2: Set up development environment with Rust toolchain, `rust-analyzer`, and CI pipeline (GitHub Actions) for building, testing (cargo test), linting (clippy), and formatting (rustfmt). Include matrix builds for x86_64-unknown-linux-gnu and wasm32-unknown-unknown targets. Deliverable: `.github/workflows/ci.yml` and rust-toolchain.toml.
- [x] Task 3: Integrate Burn framework (v0.12+) for model inference with backend selection (Autocast, Tch) and establish model format support (ONNX, Burn's `.bin`). Deliverable: `edgebot-core/src/inference/mod.rs` with burn tensor operations and model loading utilities.
- [ ] Task 4: Design zero-copy memory safety interface using Rust's `std::mem::MaybeUninit` and `ndarray`/`burn::tensor` for sharing sensor data (camera images, LiDAR) without copies between ROS2 messages and inference pipelines. Deliverable: `edgebot-core/src/memory/mod.rs` with safe pointer wrappers and documentation.

## Phase 2: Core SDK Development
- [ ] Task 1: Implement model optimizer for edge deployment: quantization (int8, fp16), pruning, and layer fusion using Burn's backend APIs. Deliverable: `edgebot-core/src/optimizer/mod.rs` with CLI subcommand `edgebot-optimize` (using clap) producing optimized `.ebmodel` bundles.
- [ ] Task 2: Build ROS2 bridge using `rclrs` crate for publishing/subscribing to topics (sensor data, inference results) with zero-copy message passing via `ros2_rust`'s loaned API patterns. Deliverable: `edgebot-ros2/src/lib.rs` with example node running YOLO inference on camera topic.
- [ ] Task 3: Create runtime for WebAssembly compilation targeting `wasm32-unknown-unknown` with `wasm-bindgen` for browser-based simulation and `wasm32-wasi` for IoT execution. Deliverable: `edgebot-wasm/src/lib.rs` and build script `build-wasm.sh` generating optimized WASM modules.
- [ ] Task 4: Develop trait-based abstraction `ModelTask` (object detection, pathfinding) with configurable runtime parameters (batch size, confidence threshold). Deliverable: `edgebot-core/src/task/mod.rs` implementing `yolo-v8` and `astar` examples and unit tests.

## Phase 3: Simulation & Compilation
- [ ] Task 1: Integrate Webots robotics simulator via `webots-sys` FFI bindings, providing a headless simulation mode for testing models on virtual robots. Deliverable: `edgebot-sim/src/webots.rs` with Python-like API wrapper for spawn, step, and sensor retrieval.
- [ ] Task 2: Build cloud simulation service using Actix Web API for uploading models, running batch simulations (multi-scene), and returning performance metrics (FPS, memory). Deliverable: `edgebot-sim-server/src/` with endpoints `/simulate`, `/results` and Dockerfile for deployment.
- [ ] Task 3: Implement cross-compilation toolchain for native ARM targets (aarch64-unknown-linux-gnu, thumbv7em-none-eabihf) using `cross` or custom Docker images, with automatic detection of target hardware (Raspberry Pi, STM32). Deliverable: `edgebot-cli/src/compile.rs` subcommand producing statically linked binaries.
- [ ] Task 4: Add profiling and benchmarking suite using `criterion` to measure inference latency, memory footprint, and model size across hardware. Deliverable: `benches/inference.rs` producing JSON reports for pro tier optimization recommendations.

## Phase 4: Deployment & Monetization
- [ ] Task 1: Create EdgeBot CLI (`edgebot-cli`) for end-users: commands `deploy` (flash binary to device via serial/SSH), `simulate` (run local/cloud sim), `optimize` (cloud pro optimization). Deliverable: `edgebot-cli/Cargo.toml` with clap derive macros and subcommands integrating all crates.
- [ ] Task 2: Implement license verification system using Ed25519 signatures for pro features (cloud sims, optimizations) with offline activation tokens. Free core SDK remains open source under MIT/Apache-2.0. Deliverable: `edgebot-licensing/src/` with `verify_pro_access()` function and integration into CLI.
- [ ] Task 3: Build dashboard frontend (static site with Yew or Leptos) showing simulation results, model metrics, and pro subscription status. Hostable on GitHub Pages or Netlify. Deliverable: `edgebot-dashboard/` with SPA using `wasm-bindgen` to call Rust analytics APIs.
- [ ] Task 4: Write comprehensive documentation: book-style guide (`edgebot-book/`) covering quickstart, ROS2 integration, WebAssembly deployment, and pro workflow. Generate API docs with `cargo doc --open`. Deliverable: `docs/book/` with markdown chapters and CI job publishing to GitHub Pages.
