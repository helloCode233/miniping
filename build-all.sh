#!/bin/bash
set -e

echo "Building miniping for multiple platforms..."

# Build for host (macOS)
echo "=== Building for macOS (host) ==="
cargo build --release
cp target/release/miniping miniping-macos
echo "Built: miniping-macos"

# Build for Linux x86_64 (musl static)
echo "=== Building for Linux x86_64 (musl) ==="
# Check if musl target is installed
if ! rustup target list | grep -q "x86_64-unknown-linux-musl (installed)"; then
    echo "Installing x86_64-unknown-linux-musl target..."
    rustup target add x86_64-unknown-linux-musl
fi

# Try to build with musl
if cargo build --release --target x86_64-unknown-linux-musl 2>/dev/null; then
    cp target/x86_64-unknown-linux-musl/release/miniping miniping-linux-x86_64
    echo "Built: miniping-linux-x86_64 (static musl)"
else
    echo "Musl build failed. Trying with Docker..."
    # Fallback to Docker build
    if command -v docker &> /dev/null; then
        docker build -t miniping-builder . 2>/dev/null || true
        if docker create --name miniping-container miniping-builder 2>/dev/null; then
            docker cp miniping-container:/miniping miniping-linux-x86_64 2>/dev/null || true
            docker rm miniping-container 2>/dev/null || true
            if [ -f miniping-linux-x86_64 ]; then
                echo "Built: miniping-linux-x86_64 (via Docker)"
            else
                echo "Warning: Could not build Linux binary"
            fi
        fi
    fi
fi

echo ""
echo "Build summary:"
ls -lh miniping-* 2>/dev/null || echo "No binaries built"
echo ""
echo "To build manually:"
echo "  macOS: cargo build --release"
echo "  Linux: cargo build --release --target x86_64-unknown-linux-musl"
echo "  Or use: docker build -t miniping ."