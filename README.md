# AuroraKVM

A lightweight, secure KVM (Keyboard, Video, Mouse) solution for seamlessly controlling multiple computers with a single set of peripherals.

## Features

- 🔒 **Secure**: Handshake protocol with secret-based authentication
- 🖥️ **Multi-Monitor Support**: Full support for complex multi-monitor setups
- 🎨 **Visual Configuration**: Drag-and-drop GUI for easy topology setup
- ⚡ **Low Latency**: Direct input event forwarding with minimal overhead
- 🌐 **Cross-Platform**: Linux and Windows support

## Architecture

- **Server**: Runs on the main machine, captures input and forwards to clients
- **Client**: Runs on secondary machines, receives and simulates input events
- **Protocol**: Binary protocol with versioning and authentication

## Installation

### Linux

#### Quick Install (Client Only)
```bash
./install-linux.sh
```

This will:
- Build the client binary
- Install to `~/.local/bin/`
- Create a desktop entry for application menu

#### Manual Build
```bash
# Build all binaries
./build.sh

# Binaries will be in target/release/
```

### Windows

#### Cross-Compile from Linux
```bash
./build-windows.sh
```

This requires `mingw-w64` to be installed:
```bash
# Ubuntu/Debian
sudo apt install mingw-w64

# Arch
sudo pacman -S mingw-w64-gcc

# Fedora
sudo dnf install mingw64-gcc
```

Binaries will be in `target/x86_64-pc-windows-gnu/release/`

## Configuration

### Server Setup

1. **Run Configuration GUI**:
   ```bash
   aurora_server --configure
   ```

2. **Configure Your Topology**:
   - Add local screens (your monitors)
   - Add client machines with their IP addresses
   - Drag clients to position them relative to your screens
   - Save configuration

3. **Example Configuration** (`~/.config/aurora_kvm/config.json`):
   ```json
   {
     "port": 8080,
     "secret": "my_secret_key",
     "local_screens": [
       {
         "x": 0,
         "y": 0,
         "width": 1920,
         "height": 1080
       },
       {
         "x": 1920,
         "y": 0,
         "width": 1920,
         "height": 1080
       }
     ],
     "clients": [
       {
         "name": "Laptop",
         "ip": "192.168.1.100:8080",
         "x": 3840,
         "y": 0,
         "width": 1920,
         "height": 1080
       }
     ]
   }
   ```

### Running

#### Server
```bash
aurora_server --port 8080
```

#### Client
```bash
aurora_client --host 192.168.1.10:8080 --secret my_secret_key
```

## Usage

1. Start the server on your main machine
2. Start clients on secondary machines
3. Move your mouse to the edge of your screen to switch focus
4. Your keyboard and mouse will now control the focused machine

## Development

### Project Structure
```
src/
├── bin/
│   ├── server.rs      # Server binary entry point
│   └── client.rs      # Client binary entry point
├── config.rs          # Configuration structures
├── event.rs           # Event type definitions
├── gui.rs             # Configuration GUI
├── net.rs             # Network protocol
├── server.rs          # Server logic
├── client.rs          # Client logic
└── topology.rs        # Focus and edge detection
```

### Building from Source
```bash
cargo build --release
```

### Testing
```bash
cargo test
```

## Security Considerations

- **Authentication**: Uses shared secret for client authentication
- **Frame Size Limits**: 1MB max frame size to prevent DoS
- **Protocol Version**: Handshake includes version negotiation
- **Local Network**: Designed for trusted local networks

> ⚠️ **Note**: This is a local network tool. Do not expose to the internet without additional security measures (VPN, firewall, etc.)

## Troubleshooting

### Linux: Input not working
Ensure you have permissions for `/dev/input/`:
```bash
sudo usermod -a -G input $USER
# Log out and back in
```

### Windows: Events not simulating
Run as administrator to ensure input injection works properly.

### Connection refused
- Check firewall settings
- Verify server is running
- Confirm correct IP address and port

## Contributing

Contributions welcome! Please:
1. Fork the repository
2. Create a feature branch
3. Make your changes
4. Submit a pull request

## License

MIT License - See LICENSE file for details

## Acknowledgments

Built with:
- [rdev](https://github.com/Narsil/rdev) - Input capture and simulation
- [egui](https://github.com/emilk/egui) - GUI framework
- [tokio](https://tokio.rs/) - Async runtime
