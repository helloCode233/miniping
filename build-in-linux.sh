#!/bin/bash
set -e

echo "Building miniping inside a Linux container..."

# Create a temporary Dockerfile for building
cat > Dockerfile.linux-build << 'EOF'
FROM rust:latest

WORKDIR /build

COPY Cargo.toml Cargo.lock ./
COPY src ./src

# Build for release
RUN cargo build --release

# Copy out the binary
CMD ["cp", "target/release/miniping", "/output/miniping-linux"]
EOF

# Create output directory
mkdir -p linux-output

# Build and extract
docker build -f Dockerfile.linux-build -t miniping-linux-builder .
docker create --name miniping-linux-container miniping-linux-builder
docker cp miniping-linux-container:/output/miniping-linux ./miniping-linux 2>/dev/null || true

# Alternative: run container and copy
if [ ! -f ./miniping-linux ]; then
    docker run --rm -v $(pwd)/linux-output:/output miniping-linux-builder || true
    cp linux-output/miniping-linux ./miniping-linux 2>/dev/null || true
fi

# Cleanup
docker rm miniping-linux-container 2>/dev/null || true
rm -f Dockerfile.linux-build
rm -rf linux-output

if [ -f ./miniping-linux ]; then
    echo "Successfully built: ./miniping-linux"
    echo "File info: $(file ./miniping-linux)"
    echo "Size: $(stat -c%s ./miniping-linux 2>/dev/null || stat -f%z ./miniping-linux) bytes"
else
    echo "Failed to build Linux binary. You can build manually on a Linux system:"
    echo "  cargo build --release"
    echo "Or cross-compile from macOS with:"
    echo "  rustup target add x86_64-unknown-linux-gnu"
    echo "  brew install FiloSottile/musl-cross/musl-cross"
    echo "  Then set up .cargo/config.toml with the correct linker"
fi