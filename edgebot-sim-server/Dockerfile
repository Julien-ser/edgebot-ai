# Multi-stage build for EdgeBot Simulation Server
FROM rust:1.74-slim AS builder

WORKDIR /app

# Install build dependencies
RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    build-essential \
    curl \
    && rm -rf /var/lib/apt/lists/*

# Copy workspace manifests
COPY Cargo.toml ./
COPY edgebot-core/Cargo.toml ./edgebot-core/
COPY edgebot-sim/Cargo.toml ./edgebot-sim/
COPY edgebot-sim-server/Cargo.toml ./edgebot-sim-server/

# Copy source code
COPY edgebot-core/src ./edgebot-core/src/
COPY edgebot-sim/src ./edgebot-sim/src/
COPY edgebot-sim-server/src ./edgebot-sim-server/src/

# Build the server
RUN cargo build --release --package edgebot-sim-server

# Runtime stage
FROM debian:bullseye-slim AS runtime

# Install runtime dependencies
RUN apt-get update && apt-get install -y \
    ca-certificates \
    curl \
    && rm -rf /var/lib/apt/lists/*

# Optional: Install Webots for full simulation (requires license)
# For headless simulation without Webots, comment this out
# COPY --from=webots-installer /usr/local/webots /usr/local/webots
# ENV WEBOTS_HOME=/usr/local/webots
# ENV PATH=$WEBOTS_HOME:$PATH

WORKDIR /app

# Copy the binary from builder
COPY --from=builder /app/target/release/edgebot-sim-server ./

# Create directories for models and logs
RUN mkdir -p /app/models /app/logs

# Environment variables
ENV HOST=0.0.0.0
ENV PORT=8080
ENV MODELS_DIR=/app/models
ENV WEBOTS_HEADLESS=true
ENV RUST_LOG=info

# Expose the server port
EXPOSE 8080

# Run the server
CMD ["./edgebot-sim-server"]