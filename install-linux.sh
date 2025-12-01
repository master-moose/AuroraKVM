#!/bin/bash
set -e

echo "Installing AuroraKVM Client..."

# Build if not already built
if [ ! -f "target/release/aurora_client" ]; then
    echo "Building release binary..."
    cargo build --release --bin aurora_client
fi

# Install binary
INSTALL_DIR="$HOME/.local/bin"
mkdir -p "$INSTALL_DIR"
cp target/release/aurora_client "$INSTALL_DIR/"
chmod +x "$INSTALL_DIR/aurora_client"
echo "✓ Installed binary to $INSTALL_DIR/aurora_client"

# Install desktop file
DESKTOP_DIR="$HOME/.local/share/applications"
mkdir -p "$DESKTOP_DIR"
cat > "$DESKTOP_DIR/aurora-kvm-client.desktop" << EOF
[Desktop Entry]
Version=1.0
Type=Application
Name=AuroraKVM Client
Comment=KVM client for AuroraKVM
Exec=$INSTALL_DIR/aurora_client --host %u
Icon=preferences-desktop-remote-desktop
Terminal=false
Categories=Network;RemoteAccess;
Keywords=kvm;remote;
EOF
echo "✓ Installed desktop file to $DESKTOP_DIR/aurora-kvm-client.desktop"

# Update desktop database
if command -v update-desktop-database &> /dev/null; then
    update-desktop-database "$DESKTOP_DIR"
    echo "✓ Updated desktop database"
fi

echo ""
echo "Installation complete!"
echo "You can now run 'aurora_client' from the command line"
echo "or launch it from your application menu."
