#!/bin/bash
# Build script for EdgeBot Dashboard WebAssembly target
# Usage: ./build-dashboard-wasm.sh [--release] [--target <target>]

set -e

# Default to wasm32-unknown-unknown
TARGET="wasm32-unknown-unknown"
PROFILE="debug"

# Parse arguments
while [[ $# -gt 0 ]]; do
    case $1 in
        --release)
            PROFILE="release"
            shift
            ;;
        --target)
            TARGET="$2"
            shift 2
            ;;
        *)
            echo "Unknown argument: $1"
            exit 1
            ;;
    esac
done

echo "Building EdgeBot Dashboard for target: $TARGET (profile: $PROFILE)"

# Check if target is installed
if ! rustup target list | grep -q "$TARGET"; then
    echo "Target $TARGET not installed. Installing..."
    rustup target add "$TARGET"
fi

# Build the dashboard
cargo build --target "$TARGET" --profile "$PROFILE" -p edgebot-dashboard

# Output location
OUT_DIR="target/$TARGET/$PROFILE"
WASM_FILE="$OUT_DIR/edgebot_dashboard.wasm"
JS_FILE="$OUT_DIR/edgebot_dashboard.js"

if [ -f "$WASM_FILE" ]; then
    echo "✓ WASM built: $WASM_FILE"
    echo "  Size: $(du -h $WASM_FILE | cut -f1)"
    
    # Copy assets to a dist folder
    DIST_DIR="dist/$TARGET/$PROFILE"
    mkdir -p "$DIST_DIR"
    
    # Copy WASM and generate JS bindings
    cp "$WASM_FILE" "$DIST_DIR/"
    
    # Generate JS wrapper with wasm-bindgen if not already done
    if [ ! -f "$JS_FILE" ]; then
        echo "Generating JavaScript bindings with wasm-bindgen..."
        wasm-bindgen "$WASM_FILE" --out-dir "$DIST_DIR" --target web --no-typescript
    else
        cp "$JS_FILE" "$DIST_DIR/"
    fi
    
    # Copy index.html and styles.css
    mkdir -p "$DIST_DIR/assets"
    cp public/index.html "$DIST_DIR/"
    cp public/styles.css "$DIST_DIR/assets/"
    
    echo "✓ Dashboard built successfully in: $DIST_DIR"
    echo ""
    echo "To serve locally:"
    echo "  cd $DIST_DIR && python3 -m http.server 8000"
    echo "  Then open: http://localhost:8000"
    echo ""
    echo "To deploy to GitHub Pages:"
    echo "  cp -r $DIST_DIR/* docs/"
    echo "  git add docs/ && git commit -m 'chore: update dashboard' && git push"
else
    echo "✗ Build failed: WASM file not found"
    exit 1
fi