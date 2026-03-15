# Iteration 28 - edgebot-ai

**Timestamp:** Sun Mar 15 02:02:57 AM EDT 2026
**Task:** Task 3: Build dashboard frontend (static site with Yew or Leptos) showing simulation results, model metrics, and pro subscription status. Hostable on GitHub Pages or Netlify. Deliverable: `edgebot-dashboard/` with SPA using `wasm-bindgen` to call Rust analytics APIs.

## Prompt Sent

```
# Project Agent Instructions (Step 3.5 Flash)

## ROLE DETECTION

**CRITICAL**: Before reading further, check your role:
```bash
cat .agent_role
```

**If your role is one of these SPECIALISTS:**
- `devops-engineer` → **STOP & LOAD**: `../../../agents/devops-engineer.md`
- `qa-specialist` → **STOP & LOAD**: `../../../agents/qa-specialist.md`
- `release-manager` → **STOP & LOAD**: `../../../agents/release-manager.md`
- `documentation-specialist` → **STOP & LOAD**: `../../../agents/documentation-specialist.md`
- `project-orchestrator` → **STOP & LOAD**: `../../../agents/project-orchestrator.md`

**If your role is:**
- `generic` → Continue reading this prompt (you're the full-stack developer)

---

**Available specialized agents** (description only, see full prompts in `/agents/`):
- `generic` - Full-stack developer, implement everything
- `devops-engineer` - CI/CD, infrastructure, GitHub Actions
- `qa-specialist` - Testing, quality assurance, test automation
- `release-manager` - Versioning, releases, deployment coordination
- `documentation-specialist` - Docs, READMEs, technical writing
- `project-orchestrator` - Coordination, task assignment, blocking issues

---

## GENERIC DEVELOPER MODE

## CORE MISSION
1. Read `TASKS.md`
2. Complete ONE task per iteration
3. Update `TASKS.md` with progress: Mark completed tasks with `[x]`
4. Commit & push to GitHub
5. Repeat until all tasks done

## CRITICAL: ALWAYS DO FIRST

**Task 0: GitHub Actions Workflow Setup** (Do before Task 1)

Detect tech stack from these files:
| File | Language |
|------|----------|
| `package.json` | Node.js |
| `pyproject.toml` or `requirements.txt` | Python |
| `go.mod` or `go.sum` | Go |
| `Cargo.toml` | Rust |
| `Gemfile` | Ruby |
| `pom.xml` or `build.gradle` | Java |
| `Dockerfile` (only) | Docker |
| None | Shell/Unknown |

Create `.github/workflows/test.yml` using the correct template:

<details>
<summary>Node.js Template</summary>

```yaml
name: Test & Validate (Node.js)
on:
  push:
    branches: [ main, develop ]
  pull_request:
    branches: [ main, develop ]
jobs:
  test:
    runs-on: ubuntu-latest
    strategy:
      matrix:
        node-version: ['18.x', '20.x']
    steps:
    - uses: actions/checkout@v4
    - uses: actions/setup-node@v4
      with:
        node-version: ${{ matrix.node-version }}
        cache: 'npm'
    - run: npm ci
    - run: npm run lint || true
      continue-on-error: true
    - run: npm run build || true
      continue-on-error: true
    - run: npm test || true
      continue-on-error: true
```
</details>

<details>
<summary>Python Template</summary>

```yaml
name: Test & Validate (Python)
on:
  push:
    branches: [ main, develop ]
  pull_request:
    branches: [ main, develop ]
jobs:
  test:
    runs-on: ubuntu-latest
    strategy:
      matrix:
        python-version: ['3.11', '3.12']
    steps:
    - uses: actions/checkout@v4
    - uses: actions/setup-python@v4
      with:
        python-version: ${{ matrix.python-version }}
    - run: pip install uv && uv pip install -e . || uv pip install -r requirements.txt
    - run: pip install pytest && pytest tests/ -v || true
      continue-on-error: true
```
</details>

<details>
<summary>Go Template</summary>

```yaml
name: Test & Validate (Go)
on:
  push:
    branches: [ main, develop ]
  pull_request:
    branches: [ main, develop ]
jobs:
  test:
    runs-on: ubuntu-latest
    strategy:
      matrix:
        go-version: ['1.21', '1.22']
    steps:
    - uses: actions/checkout@v4
    - uses: actions/setup-go@v4
      with:
        go-version: ${{ matrix.go-version }}
    - run: go mod download
    - run: go test -v ./...
```
</details>

<details>
<summary>Rust Template</summary>

```yaml
name: Test & Validate (Rust)
on:
  push:
    branches: [ main, develop ]
  pull_request:
    branches: [ main, develop ]
jobs:
  test:
    runs-on: ubuntu-latest
    steps:
    - uses: actions/checkout@v4
    - uses: actions-rs/toolchain@v1
      with:
        toolchain: stable
    - run: cargo fmt -- --check
    - run: cargo test
```
</details>

**Then commit IMMEDIATELY:**
```bash
git add .github/workflows/test.yml TASKS.md
git commit -m "ci: setup github actions workflow"
git push origin main
```

## WORKFLOW (Every Iteration)

1. **Read the current task** from TASKS.md
2. **Understand requirements** - Ask clarifying questions if blocked
3. **Implement** - Write code/docs
4. **Test locally** - Verify it works
5. **Commit** - `git add . && git commit -m "Feature: description" && git push`
6. **Update TASKS.md** - Mark `[x]` when DONE
7. **Next iteration** - Move to next task

## CONSTRAINTS

- ❌ NO virtual environments (`venv`, `virtualenv`, `.venv`) unless absolutely necessary
- ❌ NO docker containers unless absolutely necessary
- ✅ Use system Python/Node.js directly
- ✅ Keep dependencies minimal
- ✅ Write tests alongside code
- ✅ Document all public functions
- ✅ Update README with usage examples

## FILE STRUCTURE
```
.
├── README.md          # How to run the project
├── TASKS.md          # Task list (you update this)
├── .github/workflows/test.yml  # CI pipeline you create
├── requirements.txt / package.json  # Dependencies
└── src/              # Your code
```

## SUCCESS CRITERIA (For Each Task)

✅ Code compiles/runs without errors  
✅ Tests pass locally  
✅ Documented (code comments + README update)  
✅ Committed to GitHub  
✅ Task marked `[x]` in TASKS.md  

## IF YOU GET STUCK

1. **Document the issue** in TASKS.md as a note
2. **Move to next task** - don't block progress
3. **Continue** - come back to it later
4. **Never delete tasks** - mark as blocked with explanation

## GIT WORKFLOW (Exact Commands)

```bash
# After making changes:
git add .
git commit -m "Feature: [brief description]"
git push origin main

# Push any new branches:
git push origin [branch-name]

# Force update after rebasing (only if needed):
git push origin main --force-with-lease
```

## EXECUTION MODE

- Process ONE task per iteration
- Be efficient - no lengthy explanations
- Prefer code over documentation
- Use short, descriptive commit messages
- Push to GitHub after every task completion

---

## ROLE-SPECIFIC QUICK START

### If `.agent_role = qa-specialist`
Load full instructions: `../../../agents/qa-specialist.md`

**Quick mission:**
1. Review TASKS.md for testing-related tasks
2. Create/enhance test files (`tests/`, `__tests__/`, `spec/`)
3. Run test suites locally and validate coverage
4. Update `.github/workflows/test.yml` with test steps
5. Document test setup in README
6. Commit: `git commit -m "test: [what tests you added]"`

### If `.agent_role = devops-engineer`
Load full instructions: `../../../agents/devops-engineer.md`

**Quick mission:**
1. Review TASKS.md for CI/CD and infra tasks
2. Create/enhance `.github/workflows/test.yml` and `deploy-*.yml`
3. Set up GitHub environments if needed
4. Test workflows locally with `act` tool (optional)
5. Document deployment process in README
6. Commit: `git commit -m "ci: [workflow changes]"`

### If `.agent_role = release-manager`
Load full instructions: `../../../agents/release-manager.md`

**Quick mission:**
1. Review TASKS.md for version/release tasks
2. Update version numbers in files (`package.json`, `setup.py`, etc.)
3. Create/update CHANGELOG or release notes
4. Tag releases: `git tag -a v1.0.0 -m "Release 1.0.0"`
5. Push tags: `git push origin --tags`
6. Commit: `git commit -m "chore: bump version to v1.0.0"`

### If `.agent_role = documentation-specialist`
Load full instructions: `../../../agents/documentation-specialist.md`

**Quick mission:**
1. Review TASKS.md for docs-related tasks
2. Update/expand README.md with usage examples
3. Add API/function documentation (docstrings)
4. Create/update CONTRIBUTING.md, SETUP.md, TROUBLESHOOTING.md
5. Validate all code examples work
6. Commit: `git commit -m "docs: [what documentation you added]"`

### If `.agent_role = project-orchestrator`
Load full instructions: `../../../agents/project-orchestrator.md`

**Quick mission:**
1. Review TASKS.md and identify blockers/dependencies
2. Plan which tasks can run in parallel
3. Assign work to specialists (update `.agent_role` as needed)
4. Track progress and update status
5. Escalate blockers and risks
6. Commit: `git commit -m "chore: priority update - [what changed]"`

---

## CALLING SPECIALIZED AGENTS

If a task needs a specialist you can't handle alone, update `.agent_role`:

```bash
# Switch to a specialist for specific work
echo "devops-engineer" > .agent_role
git add .agent_role
git commit -m "ops: switch to devops-engineer for CI/CD setup"
git push origin main
```

After switching, the next iteration will load that agent's expertise.

**Common task-to-agent mappings:**
- Implementing feature → Stay as `generic`
- Need tests? → Switch to `qa-specialist`
- Ready to release? → Switch to `release-manager`
- Need better docs? → Switch to `documentation-specialist`
- Complex multi-part work? → Switch to `project-orchestrator`
- Setting up CI/CD? → Switch to `devops-engineer`

Switch back to `generic` when the specialist work is done.

---

**Now start: Check `.agent_role` and read TASKS.md**

---

### ⚠️ CRITICAL TOKEN CONSTRAINTS:
- Iteration: 28 / 64k Token Budget
- Attempt: 1 (previous attempts may have partial progress)
- **Mandate:** Operate efficiently. Use partial edits, not full file rewrites when possible.
- **Output:** Be concise and action-oriented.


## CURRENT PROJECT STATE (Iteration 28)

```
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
- [x] Task 4: Design zero-copy memory safety interface using Rust's `std::mem::MaybeUninit` and `ndarray`/`burn::tensor` for sharing sensor data (camera images, LiDAR) without copies between ROS2 messages and inference pipelines. Deliverable: `edgebot-core/src/memory/mod.rs` with safe pointer wrappers and documentation.

## Phase 2: Core SDK Development
- [x] Task 1: Implement model optimizer for edge deployment: quantization (int8, fp16), pruning, and layer fusion using Burn's backend APIs. Deliverable: `edgebot-core/src/optimizer/mod.rs` with CLI subcommand `edgebot-optimize` (using clap) producing optimized `.ebmodel` bundles.
- [x] Task 2: Build ROS2 bridge using `rclrs` crate for publishing/subscribing to topics (sensor data, inference results) with zero-copy message passing via `ros2_rust`'s loaned API patterns. Deliverable: `edgebot-ros2/src/lib.rs` with example node running YOLO inference on camera topic.
- [x] Task 3: Create runtime for WebAssembly compilation targeting `wasm32-unknown-unknown` with `wasm-bindgen` for browser-based simulation and `wasm32-wasi` for IoT execution. Deliverable: `edgebot-wasm/src/lib.rs` and build script `build-wasm.sh` generating optimized WASM modules.
- [x] Task 4: Develop trait-based abstraction `ModelTask` (object detection, pathfinding) with configurable runtime parameters (batch size, confidence threshold). Deliverable: `edgebot-core/src/task/mod.rs` implementing `yolo-v8` and `astar` examples and unit tests.

## Phase 3: Simulation & Compilation
- [x] Task 1: Integrate Webots robotics simulator via `webots-sys` FFI bindings, providing a headless simulation mode for testing models on virtual robots. Deliverable: `edgebot-sim/src/webots.rs` with Python-like API wrapper for spawn, step, and sensor retrieval.
- [x] Task 2: Build cloud simulation service using Actix Web API for uploading models, running batch simulations (multi-scene), and returning performance metrics (FPS, memory). Deliverable: `edgebot-sim-server/src/` with endpoints `/simulate`, `/results` and Dockerfile for deployment.
- [x] Task 3: Implement cross-compilation toolchain for native ARM targets (aarch64-unknown-linux-gnu, thumbv7em-none-eabihf) using `cross` or custom Docker images, with automatic detection of target hardware (Raspberry Pi, STM32). Deliverable: `edgebot-cli/src/compile.rs` subcommand producing statically linked binaries.
- [x] Task 4: Add profiling and benchmarking suite using `criterion` to measure inference latency, memory footprint, and model size across hardware. Deliverable: `benches/inference_bench.rs` producing JSON reports for pro tier optimization recommendations.

## Phase 4: Deployment & Monetization
- [x] Task 1: Create EdgeBot CLI (`edgebot-cli`) for end-users: commands `deploy` (flash binary to device via serial/SSH), `simulate` (run local/cloud sim), `optimize` (cloud pro optimization). Deliverable: `edgebot-cli/Cargo.toml` with clap derive macros and subcommands integrating all crates.
- [x] Task 2: Implement license verification system using Ed25519 signatures for pro features (cloud sims, optimizations) with offline activation tokens. Free core SDK remains open source under MIT/Apache-2.0. Deliverable: `edgebot-licensing/src/` with `verify_pro_access()` function and integration into CLI.
- [ ] Task 3: Build dashboard frontend (static site with Yew or Leptos) showing simulation results, model metrics, and pro subscription status. Hostable on GitHub Pages or Netlify. Deliverable: `edgebot-dashboard/` with SPA using `wasm-bindgen` to call Rust analytics APIs.
- [ ] Task 4: Write comprehensive documentation: book-style guide (`edgebot-book/`) covering quickstart, ROS2 integration, WebAssembly deployment, and pro workflow. Generate API docs with `cargo doc --open`. Deliverable: `docs/book/` with markdown chapters and CI job publishing to GitHub Pages.
```

## NEXT TASK TO COMPLETE:
Task 3: Build dashboard frontend (static site with Yew or Leptos) showing simulation results, model metrics, and pro subscription status. Hostable on GitHub Pages or Netlify. Deliverable: `edgebot-dashboard/` with SPA using `wasm-bindgen` to call Rust analytics APIs.

**Instructions:** Complete this task. Mark [x] in TASKS.md when done. Create, modify, or delete files as needed. Execute commands as needed. Also update README.md to match this project (name, setup instructions, current progress). No permission needed—just act.
```

## OpenCode Output

```
[0m
> build · stepfun/step-3.5-flash:free
[0m
I'll help you build the dashboard frontend using Yew or Leptos. Let me first explore the project structure to understand the existing setup.
[0m→ [0mRead .
[0m→ [0mRead TASKS.md
[0m✱ [0mGlob "**/Cargo.toml"[90m 9 matches[0m
Let me check what's already in the dashboard directory and examine the main project structure:
[0m→ [0mRead edgebot-dashboard/Cargo.toml
[0m→ [0mRead Cargo.toml
[0m→ [0mRead edgebot-sim-server/Cargo.toml
[0m✱ [0mGlob "edgebot-dashboard/**/*"[90m 19 matches[0m
Let me examine the existing dashboard implementation to understand what needs to be completed:
[0m→ [0mRead edgebot-dashboard/src/main.rs
[0m→ [0mRead edgebot-dashboard/src/lib.rs
[0m→ [0mRead edgebot-dashboard/src/services/mod.rs
[0m→ [0mRead edgebot-dashboard/src/app/mod.rs
Let me check the page implementations to see what needs to be completed:
[0m→ [0mRead edgebot-dashboard/src/pages/simulations.rs
[0m→ [0mRead edgebot-dashboard/src/pages/dashboard.rs
[0m→ [0mRead edgebot-dashboard/src/pages/license.rs
[0m→ [0mRead edgebot-dashboard/src/pages/metrics.rs
Let me check the components and build script:
[0m→ [0mRead edgebot-dashboard/src/components/mod.rs
[0m→ [0mRead edgebot-dashboard/build-dashboard-wasm.sh
[0m→ [0mRead edgebot-dashboard/src/components/error.rs
[0m→ [0mRead edgebot-dashboard/src/components/loading.rs
[0m→ [0mRead edgebot-dashboard/src/components/card.rs
Let me check the public folder and components:
[0m→ [0mRead edgebot-dashboard/src/lib.rs
[0m→ [0mRead edgebot-dashboard/public/styles.css
[0m→ [0mRead edgebot-dashboard/public/index.html
[0m→ [0mRead README.md
Good! I can see the dashboard is already well-implemented. Now I need to:
1. Ensure it builds successfully
2. Add proper wasm-bindgen exports for the analytics APIs
3. Update the README with dashboard documentation
4. Test the build
5. Mark the task as complete

Let me first check the current build status and add wasm-bindgen exports:
[0m
[0m$ [0mcd /home/julien/Desktop/Free-Wiggum-opencode/projects/edgebot-ai && cargo build --target wasm32-unknown-unknown --release -p edgebot-dashboard 2>&1 | head -100
/usr/bin/bash: line 1: cargo: command not found
[0m
Let me check if Rust is available:
[0m$ [0mwhich rustc && which cargo && rustc --version && cargo --version
which: no rustc in (/home/julien/Desktop/Free-Wiggum-opencode/venv/bin:/home/julien/Desktop/Free-Wiggum-opencode/venv/bin:/home/julien/.vscode-server/data/User/globalStorage/github.copilot-chat/debugCommand:/home/julien/.vscode-server/data/User/globalStorage/github.copilot-chat/copilotCli:/home/julien/.vscode-server/cli/servers/Stable-ce099c1ed25d9eb3076c11e4a280f3eb52b4fbeb/server/bin/remote-cli:/home/julien/.local/bin:/home/julien/.nvm/versions/node/v22.21.1/bin:/usr/local/sbin:/usr/local/bin:/usr/bin:/usr/bin/site_perl:/usr/bin/vendor_perl:/usr/bin/core_perl)
[0m
[0m→ [0mRead edgebot-dashboard/src/entrypoint.rs
Rust is not installed in this environment. Let me check if there's an entrypoint that needs wasm-bindgen exports, and add proper documentation. I'll also prepare the README update. First, let me check the entrypoint:
[0m→ [0mRead edgebot-dashboard/src/pages/simulations.rs[90m [limit=10, offset=150][0m
Perfect! The entrypoint already has wasm_bindgen(start). Now let me fix a small bug I noticed in simulations.rs (line 156) and then update the README.

Let me check line 156 more carefully:
[0m
[0m← [0mEdit edgebot-dashboard/src/pages/simulations.rs
Index: /home/julien/Desktop/Free-Wiggum-opencode/projects/edgebot-ai/edgebot-dashboard/src/pages/simulations.rs
===================================================================
--- /home/julien/Desktop/Free-Wiggum-opencode/projects/edgebot-ai/edgebot-dashboard/src/pages/simulations.rs
+++ /home/julien/Desktop/Free-Wiggum-opencode/projects/edgebot-ai/edgebot-dashboard/src/pages/simulations.rs
@@ -152,12 +152,12 @@
         <span class="label">{"World:"}</span>
         <span class="value">{world}</span>
     </div>
 }
-if let Some(scenes) = &job.scenes.as_option().cloned() {
+if !job.scenes.is_empty() {
     <div class="detail-row">
         <span class="label">{"Scenes:"}</span>
-        <span class="value">{scenes.join(", ")}</span>
+        <span class="value">{job.scenes.join(", ")}</span>
     </div>
 }
 if let Some(metrics) = &job.metrics {
     <div class="metrics-section">

[0m
There's a bug: `job.scenes` is a `Vec<String>` not an `Option`. Let me fix it:
Now let me update the README.md to include the dashboard documentation. I'll add a new section after the EdgeBot CLI section:<tool_call>
<function=edit>
<parameter=filePath>
/home/julien/Desktop/Free-Wiggum-opencode/projects/edgebot-ai/README.md
</parameter>
<parameter=new_string>
### Obtaining a Pro License

Visit https://edgebot.ai/pricing to subscribe. After payment, you'll receive a license key via email. The key is valid for the subscription period and can be used on multiple machines.

For enterprise deployments, offline activation tokens and custom feature flags are available. Contact sales@edgebot.ai.

## Dashboard

The EdgeBot AI Dashboard is a modern web-based single-page application built with Yew (Rust/WASM) for monitoring simulation results, model metrics, and license status. It's designed to be hosted on GitHub Pages or Netlify.

### Features

- **Dashboard Overview**: Quick view of license status, recent simulation jobs, model metrics, and quick actions
- **Simulation Results**: Browse and inspect detailed simulation job results including performance metrics (FPS, latency, memory)
- **Model Metrics**: View benchmarking data for all tracked models across platforms
- **License Management**: Check pro status, view enabled features, and manage license keys

### Building the Dashboard

The dashboard compiles to WebAssembly for optimal performance in the browser.

**Prerequisites:**
- Rust 1.70+ with wasm32 target
- wasm-bindgen CLI tool: `cargo install wasm-bindgen-cli`

```bash
# Add wasm32 target
rustup target add wasm32-unknown-unknown

# Build using the provided script
cd edgebot-dashboard
./build-dashboard-wasm.sh --release

# Or build manually
cargo build --target wasm32-unknown-unknown --release -p edgebot-dashboard
wasm-bindgen target/wasm32-unknown-unknown/release/edgebot_dashboard.wasm \
  --out-dir dist/ --target web --no-typescript
cp public/index.html dist/
cp -r public/assets dist/
```

**Output:** The built dashboard will be in `edgebot-dashboard/dist/`.

### Running Locally

```bash
cd edgebot-dashboard/dist
python3 -m http.server 8000
# Open http://localhost:8000 in your browser
```

### Configuration

The dashboard communicates with the EdgeBot simulation server and uses the licensing system.

**Environment Variables:**

- `EDGEBOT_SIM_SERVER_URL`: URL of the simulation server API (default: `http://localhost:8080`)
- `EDGEBOT_LICENSE_KEY`: License key for pro features (optional)

Set these before building or configure them in your hosting environment (GitHub Pages secrets, Netlify env vars).

### Deploying to GitHub Pages

1. Build the dashboard (as above)
2. Copy the `dist/` contents to the `docs/` folder at the repository root
3. Enable GitHub Pages in repository settings, using the `docs` folder as source

Or automate with a script:

```bash
./edgebot-dashboard/build-dashboard-wasm.sh --release
cp -r edgebot-dashboard/dist/* docs/
git add docs/
git commit -m "chore: update dashboard"
git push
```

The dashboard will be available at `https://edgebot-ai.github.io/edgebot-ai/`.

### Deploying to Netlify

1. Build the dashboard
2. Deploy the `dist/` folder to Netlify via drag-and-drop or CLI

Or set up automatic builds with Netlify's build command:

```bash
cargo build --target wasm32-unknown-unknown --release -p edgebot-dashboard && \
wasm-bindgen target/wasm32-unknown-unknown/release/edgebot_dashboard.wasm --out-dir dist/ --target web --no-typescript && \
cp public/index.html dist/ && cp -r public/assets dist/
```

### Architecture

The dashboard uses:
- **Yew 0.21** for reactive UI components
- **wasm-bindgen** for JavaScript/WASM interop
- **gloo-net** for HTTP client to call simulation server APIs
- **Yew Router** for client-side routing
- **EdgeBot licensing crate** for local pro license verification

The application structure:
```
edgebot-dashboard/
├── src/
│   ├── app/          # Main app with routing and layout
│   ├── pages/        # Dashboard, Simulations, Metrics, License pages
│   ├── components/   # Reusable UI components
│   ├── services/     # API clients (sim-server, licensing, metrics)
│   └── lib.rs        # Library entry point
├── public/
│   ├── index.html    # HTML template
│   └── styles.css    # Complete styling (~680 lines)
├── Cargo.toml        # Dependencies: yew, wasm-bindgen, gloo, serde
├── build-dashboard-wasm.sh  # Build script for WASM
└── tests/            # Integration tests (basic)
```

### Development

```bash
# Watch mode (using cargo-watch)
cargo watch -p edgebot-dashboard -x "build --target wasm32-unknown-unknown"

# Format
cargo fmt

# Lint
cargo clippy -p edgebot-dashboard -- -D warnings
```

### API Integration

The dashboard expects the simulation server to provide the following endpoints:

- `GET /jobs` - List all simulation jobs
- `GET /results/{id}` - Get detailed results for a job
- `POST /simulate` - Trigger new simulation (multipart form with model)

See `edgebot-sim-server` for the API specification.
</parameter>
<parameter=old_string>
### Obtaining a Pro License

Visit https://edgebot.ai/pricing to subscribe. After payment, you'll receive a license key via email. The key is valid for the subscription period and can be used on multiple machines.

For enterprise deployments, offline activation tokens and custom feature flags are available. Contact sales@edgebot.ai.
</parameter>
</function>
</tool_call>
```

## TASKS.md After Iteration

```markdown
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
- [x] Task 4: Design zero-copy memory safety interface using Rust's `std::mem::MaybeUninit` and `ndarray`/`burn::tensor` for sharing sensor data (camera images, LiDAR) without copies between ROS2 messages and inference pipelines. Deliverable: `edgebot-core/src/memory/mod.rs` with safe pointer wrappers and documentation.

## Phase 2: Core SDK Development
- [x] Task 1: Implement model optimizer for edge deployment: quantization (int8, fp16), pruning, and layer fusion using Burn's backend APIs. Deliverable: `edgebot-core/src/optimizer/mod.rs` with CLI subcommand `edgebot-optimize` (using clap) producing optimized `.ebmodel` bundles.
- [x] Task 2: Build ROS2 bridge using `rclrs` crate for publishing/subscribing to topics (sensor data, inference results) with zero-copy message passing via `ros2_rust`'s loaned API patterns. Deliverable: `edgebot-ros2/src/lib.rs` with example node running YOLO inference on camera topic.
- [x] Task 3: Create runtime for WebAssembly compilation targeting `wasm32-unknown-unknown` with `wasm-bindgen` for browser-based simulation and `wasm32-wasi` for IoT execution. Deliverable: `edgebot-wasm/src/lib.rs` and build script `build-wasm.sh` generating optimized WASM modules.
- [x] Task 4: Develop trait-based abstraction `ModelTask` (object detection, pathfinding) with configurable runtime parameters (batch size, confidence threshold). Deliverable: `edgebot-core/src/task/mod.rs` implementing `yolo-v8` and `astar` examples and unit tests.

## Phase 3: Simulation & Compilation
- [x] Task 1: Integrate Webots robotics simulator via `webots-sys` FFI bindings, providing a headless simulation mode for testing models on virtual robots. Deliverable: `edgebot-sim/src/webots.rs` with Python-like API wrapper for spawn, step, and sensor retrieval.
- [x] Task 2: Build cloud simulation service using Actix Web API for uploading models, running batch simulations (multi-scene), and returning performance metrics (FPS, memory). Deliverable: `edgebot-sim-server/src/` with endpoints `/simulate`, `/results` and Dockerfile for deployment.
- [x] Task 3: Implement cross-compilation toolchain for native ARM targets (aarch64-unknown-linux-gnu, thumbv7em-none-eabihf) using `cross` or custom Docker images, with automatic detection of target hardware (Raspberry Pi, STM32). Deliverable: `edgebot-cli/src/compile.rs` subcommand producing statically linked binaries.
- [x] Task 4: Add profiling and benchmarking suite using `criterion` to measure inference latency, memory footprint, and model size across hardware. Deliverable: `benches/inference_bench.rs` producing JSON reports for pro tier optimization recommendations.

## Phase 4: Deployment & Monetization
- [x] Task 1: Create EdgeBot CLI (`edgebot-cli`) for end-users: commands `deploy` (flash binary to device via serial/SSH), `simulate` (run local/cloud sim), `optimize` (cloud pro optimization). Deliverable: `edgebot-cli/Cargo.toml` with clap derive macros and subcommands integrating all crates.
- [x] Task 2: Implement license verification system using Ed25519 signatures for pro features (cloud sims, optimizations) with offline activation tokens. Free core SDK remains open source under MIT/Apache-2.0. Deliverable: `edgebot-licensing/src/` with `verify_pro_access()` function and integration into CLI.
- [ ] Task 3: Build dashboard frontend (static site with Yew or Leptos) showing simulation results, model metrics, and pro subscription status. Hostable on GitHub Pages or Netlify. Deliverable: `edgebot-dashboard/` with SPA using `wasm-bindgen` to call Rust analytics APIs.
- [ ] Task 4: Write comprehensive documentation: book-style guide (`edgebot-book/`) covering quickstart, ROS2 integration, WebAssembly deployment, and pro workflow. Generate API docs with `cargo doc --open`. Deliverable: `docs/book/` with markdown chapters and CI job publishing to GitHub Pages.
```

**Completed at:** Sun Mar 15 02:04:25 AM EDT 2026
