# AuroraKVM Rust KVM Project

This file contains a full pseudocode scaffold, project outline, and LLM prompt for developing AuroraKVM, a self-hosted cross-platform Rust-based keyboard/mouse sharing system.

---

## System Prompt for LLM Agent

```
You are a senior software engineer and CTO, a master of your craft, and a savant with an IQ of 160. Your duty is to inspect the following software project in detail, identify all technical issues, code smells, architectural weaknesses, security vulnerabilities, performance bottlenecks, and inconsistencies. Propose and implement solutions where possible, ensuring the project is fully production-ready, maintainable, and scalable. Provide clear explanations for each change or recommendation you make, prioritizing best practices, efficiency, and reliability. Assume full authority to refactor, optimize, and improve the project code and structure.
```

---

## Project: AuroraKVM

### Goals
1. Share mouse and keyboard across multiple machines over LAN.
2. Fully self-hosted; server runs on main PC, clients run on other machines.
3. Cross-platform: Windows & Linux (macOS optional).
4. Minimal external dependencies, mostly Rust crates.
5. Secure, maintainable, and production-ready.
6. Hotkey or mouse-edge switching between machines.

### High-Level Architecture

```
         ┌─────────────┐
         │  Main PC    │
         │  AuroraKVM  │
         │  Server     │
         └─────┬───────┘
               │ TCP/Encrypted LAN
     ┌─────────┴───────────┐
     │                     │
┌────┴─────┐           ┌───┴─────┐
│ Client 1 │           │ Client 2│
│ AuroraKVM│           │ AuroraKVM│
└──────────┘           └──────────┘
```

### Core Components

#### Server
- Listens for TCP connections from clients.
- Maintains a table of connected clients and focus state.
- Receives `KvmEvent` objects (keyboard/mouse events) from clients.
- Injects events locally using `rdev`.
- Handles machine focus switching.
- Crates: `tokio`, `rdev`, `serde + bincode`, optional `tokio-rustls`.

#### Client
- Captures local keyboard/mouse events using `rdev`.
- Sends events to server over TCP.
- Requests focus via hotkey or mouse-edge detection.
- Crates: `rdev`, `tokio`, `serde + bincode`.

#### Serialization

```rust
#[derive(Serialize, Deserialize)]
pub enum KvmEvent {
    MouseMove { x: i32, y: i32 },
    MouseButton { button: u8, pressed: bool },
    Key { code: u32, pressed: bool },
}
```

#### Focus Management
- Server tracks which client has focus.
- Client sends focus requests via hotkey/mouse-edge detection.

#### Security
- Optional TLS using `tokio-rustls`.
- Optional pre-shared key authentication.

### Project Structure

```
aurora_kvm/
├─ Cargo.toml
├─ src/
│  ├─ main.rs         # Entry point, parses CLI args for server/client
│  ├─ server.rs       # TCP server + focus management
│  ├─ client.rs       # Captures & sends input events
│  ├─ events.rs       # KvmEvent struct + serialization
│  ├─ config.rs       # Config parser (ports, hotkeys, client mapping)
│  ├─ network.rs      # TCP + optional TLS abstraction
│  └─ utils.rs        # Logging, error handling
└─ README.md
```

### Future / Advanced Features
- Auto-discovery via LAN (mDNS / UDP broadcast)
- Multi-monitor support
- Clipboard sharing
- Customizable hotkeys
- Optional input event compression

### Pseudocode Scaffold

#### main.rs
```
if arg == "server":
    start_server()
else if arg == "client":
    start_client(server_addr)
```

#### server.rs
```
function start_server():
    listen TCP on port
    loop:
        accept client connection
        spawn handle_client(client)

function handle_client(client):
    loop:
        event = receive KvmEvent
        if client_has_focus:
            inject_event(event)
        else:
            ignore_event
```

#### client.rs
```
function start_client(server_addr):
    connect TCP to server
    listen for local events using rdev
    loop:
        event = capture_local_event()
        send event to server
```

#### events.rs
```
enum KvmEvent:
    MouseMove(x, y)
    MouseButton(button, pressed)
    Key(code, pressed)
```

#### config.rs
```
parse config file:
    port, hotkeys, client mapping
```

#### network.rs
```
abstract TCP + optional TLS
send_event(event)
receive_event()
```

#### utils.rs
```
logging
error handling
```

### Task for LLM Agent
- Audit the pseudocode and outline.
- Identify technical issues, architecture weaknesses, security flaws, and performance bottlenecks.
- Refactor and optimize into a production-ready, maintainable, scalable Rust project.
- Implement missing details for TCP streaming, focus switching, hotkeys, optional TLS, and cross-platform input injection.
- Produce a working Rust scaffold ready to compile and run on Windows and Linux without admin permissions.
```

---

This `.md` file is fully self-contained and can be given directly to your LLM agent for code generation, auditing, and optimization. It includes goals, architecture, module pseudocode, and a complete system prompt. 

