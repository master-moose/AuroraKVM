#!/bin/bash
# AuroraKVM Windows Build Script
# This script cross-compiles for Windows using mingw-w64

set -e

echo "AuroraKVM Windows Build Script"
echo "=============================="
echo ""

# Check if cross-compilation target is installed
if ! rustup target list --installed | grep -q x86_64-pc-windows-gnu; then
    echo "Installing Windows target..."
    rustup target add x86_64-pc-windows-gnu
fi

# Check for mingw-w64
if ! command -v x86_64-w64-mingw32-gcc &> /dev/null; then
    echo "ERROR: mingw-w64 not found!"
    echo "Please install it with:"
    echo "  Ubuntu/Debian: sudo apt install mingw-w64"
    echo "  Arch: sudo pacman -S mingw-w64-gcc"
    echo "  Fedora: sudo dnf install mingw64-gcc"
    exit 1
fi

echo "Building Windows binaries..."
echo ""

# Build server
echo "Building aurora_server.exe..."
cargo build --release --target x86_64-pc-windows-gnu --bin aurora_server

# Build client
echo "Building aurora_client.exe..."
cargo build --release --target x86_64-pc-windows-gnu --bin aurora_client

echo ""
echo "✓ Build complete!"
echo ""
echo "Windows binaries located at:"
echo "  Server: ./target/x86_64-pc-windows-gnu/release/aurora_server.exe"
echo "  Client: ./target/x86_64-pc-windows-gnu/release/aurora_client.exe"
echo ""
echo "Copy these files to your Windows machine to run them."
