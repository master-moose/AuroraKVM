#!/bin/bash
set -e

echo "Building AuroraKVM Release Binaries..."

# Build release binaries
cargo build --release

echo "✓ Build complete!"
echo ""
echo "Binaries located at:"
echo "  Server: ./target/release/aurora_server"
echo "  Client: ./target/release/aurora_client"
