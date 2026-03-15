#!/bin/bash
#
# EdgeBot WASM Build Script
# Builds WebAssembly modules for browser and WASI targets
#
# Usage:
#   ./build-wasm.sh [target] [options]
#
# Targets:
#   browser    - wasm32-unknown-unknown (default)
#   wasi       - wasm32-wasi
#   all        - Build both targets
#
# Options:
#   --release  - Build in release mode (default)
#   --debug    - Build in debug mode
#   --optimize - Additional size optimizations
#   --help     - Show this help

set -e  # Exit on error

# Default values
TARGET=${1:-browser}
MODE="--release"
OPTIMIZE=false
OUTPUT_DIR="target/wasm"

# Parse arguments
shift || true
while [[ $# -gt 0 ]]; do
    case $1 in
        --release)
            MODE="--release"
            shift
            ;;
        --debug)
            MODE=""
            shift
            ;;
        --optimize)
            OPTIMIZE=true
            shift
            ;;
        --help)
            echo "Usage: $0 [target] [options]"
            echo ""
            echo "Targets:"
            echo "  browser    Build wasm32-unknown-unknown (default)"
            echo "  wasi       Build wasm32-wasi"
            echo "  all        Build both targets"
            echo ""
            echo "Options:"
            echo "  --release  Build in release mode (default)"
            echo "  --debug    Build in debug mode"
            echo "  --optimize Enable additional size optimizations"
            echo "  --help     Show this help"
            exit 0
            ;;
        *)
            echo "Unknown option: $1"
            exit 1
            ;;
    esac
done

# Color codes for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

log_info() {
    echo -e "${GREEN}[INFO]${NC} $1"
}

log_warn() {
    echo -e "${YELLOW}[WARN]${NC} $1"
}

log_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

# Ensure output directory exists
mkdir -p "$OUTPUT_DIR"

# Build function
build_target() {
    local target_triple=$1
    local target_name=$2
    local output_name=$3

    log_info "Building for $target_name ($target_triple)..."

    # Set RUSTFLAGS for WASM optimization
    local rustflags=""
    if [ "$OPTIMIZE" = true ]; then
        rustflags="-C link-arg=-z -C link-arg=stack-size=65536"
        log_warn "Enabling additional size optimizations"
    fi

    # Build the WASM module
    if [ -n "$MODE" ]; then
        cargo build $MODE --target "$target_triple" --package edgebot-wasm
    else
        cargo build --target "$target_triple" --package edgebot-wasm
    fi

    # Copy output to named file
    local input_path="target/$target_triple/$output_name/edgebot_wasm.core.wasm"
    local output_path="$OUTPUT_DIR/edgebot-wasm-${target_name}.wasm"

    if [ -f "$input_path" ]; then
        cp "$input_path" "$output_path"
        log_info "WASM module created: $output_path"

        # Show file size
        local size=$(stat -f%z "$output_path" 2>/dev/null || stat -c%s "$output_path")
        log_info "File size: $size bytes"
    else
        # Try alternative WASM file name patterns
        local alt_path="target/$target_triple/$output_name/edgebot_wasm.wasm"
        if [ -f "$alt_path" ]; then
            cp "$alt_path" "$output_path"
            log_info "WASM module created: $output_path"
        else
            log_error "WASM file not found at expected location"
            log_info "Searched in:"
            log_info "  - $input_path"
            log_info "  - $alt_path"
            exit 1
        fi
    fi
}

# Build based on target
case "$TARGET" in
    browser)
        build_target "wasm32-unknown-unknown" "browser" "wasm32-unknown-unknown"
        ;;
    wasi)
        build_target "wasm32-wasi" "wasi" "wasm32-wasi"
        ;;
    all)
        build_target "wasm32-unknown-unknown" "browser" "wasm32-unknown-unknown"
        build_target "wasm32-wasi" "wasi" "wasm32-wasi"
        log_info "All targets built successfully"
        ;;
    *)
        log_error "Unknown target: $TARGET"
        log_info "Valid targets: browser, wasi, all"
        exit 1
        ;;
esac

log_info "WASM build complete!"
