#!/bin/bash
set -e

PROFILE="debug"
while [[ $# -gt 0 ]]; do
    case $1 in
        --release)
            PROFILE="release"
            shift
            ;;
        *)
            echo "Unknown argument: $1"
            exit 1
            ;;
    esac
done

echo "Building EdgeBot Dashboard (profile: $PROFILE)"

# Ensure wasm target is installed
if ! rustup target list | grep -q "wasm32-unknown-unknown (installed)"; then
    echo "Installing wasm32-unknown-unknown target..."
    rustup target add wasm32-unknown-unknown
fi

# Install trunk if missing
if ! command -v trunk &> /dev/null; then
    echo "Installing trunk..."
    cargo install trunk
fi

# Build with trunk
if [ "$PROFILE" = "release" ]; then
    trunk build --release
else
    trunk build
fi

OUT_DIR="dist"
if [ -d "$OUT_DIR" ]; then
    echo "✓ Dashboard built in $OUT_DIR/"
    echo ""
    echo "To serve locally:"
    echo "  cd $OUT_DIR && python3 -m http.server 8000"
    echo "  Then open: http://localhost:8000"
    echo ""
    echo "To deploy to GitHub Pages:"
    echo "  cp -r $OUT_DIR/* ../docs/"
    echo "  git add docs/ && git commit -m 'chore: update dashboard' && git push"
    echo ""
    echo "Or use the GitHub Actions workflow for automatic deployment."
else
    echo "✗ Build failed: $OUT_DIR not found"
    exit 1
fi
