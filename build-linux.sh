#!/bin/bash
# Build miniping Linux production binary (static, stripped, minimum size)
set -e

PROFILE="${1:-production}"
TARGET="x86_64-unknown-linux-musl"

echo "=== Building miniping for Linux x86_64 (profile: $PROFILE) ==="
echo ""

case "$PROFILE" in
  release)
    cargo build --release --target "$TARGET"
    cp "target/$TARGET/release/miniping" miniping-linux-x86_64
    ;;
  production)
    cargo build --profile production --target "$TARGET"
    cp "target/$TARGET/production/miniping" miniping-linux-x86_64
    ;;
  *)
    echo "Usage: $0 [release|production]"
    echo "  release     - balanced speed/size"
    echo "  production  - maximum size reduction (default)"
    exit 1
    ;;
esac

echo ""
echo "Binary: miniping-linux-x86_64"
ls -lh miniping-linux-x86_64
file miniping-linux-x86_64