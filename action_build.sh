#!/bin/bash
set -e

IMAGE_NAME="miniping"

# Get commit hash
COMMIT_HASH="${GITHUB_SHA}"
if [ -z "${COMMIT_HASH}" ]; then
  COMMIT_HASH="local-build"
fi

# Add musl target and build
echo "======================================"
echo "📦 构建项目：${IMAGE_NAME}"
echo "🏷️  提交哈希：${COMMIT_HASH}"
echo "======================================"

# Install musl tools and target
apk add --no-cache musl-tools
rustup target add x86_64-unknown-linux-musl

# Build static binary
cargo build --release --profile production --target x86_64-unknown-linux-musl

# Strip binary
strip target/x86_64-unknown-linux-musl/release/miniping

# Tag and save artifact info
echo "miniping-linux-x86_64" > artifact_name.txt
cp target/x86_64-unknown-linux-musl/release/miniping ./miniping-linux-x86_64

echo "✅ 构建完成！"
ls -lh miniping-linux-x86_64