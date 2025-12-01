use crate::config::Config;
use crate::event::KvmEvent;
use crate::net::{KvmStream, PROTOCOL_VERSION, Packet};
use crate::topology::{Focus, Topology};
use anyhow::Result;
use dirs;
use rdev::{EventType, listen};
use serde_json;
use std::sync::{Arc, Mutex};
use tokio::net::TcpListener;

pub async fn run(port: u16) -> Result<()> {
    run_with_state(port, crate::connected::create_connected_clients()).await
}

pub async fn run_with_state(
    port: u16,
    connected_clients: crate::connected::ConnectedClients,
) -> Result<()> {
    // Load config from file
    let config_path = dirs::config_dir()
        .unwrap_or_else(|| std::path::PathBuf::from("."))
        .join("aurora_kvm")
        .join("config.json");

    let config = if config_path.exists() {
        std::fs::read_to_string(&config_path)
            .ok()
            .and_then(|s| serde_json::from_str(&s).ok())
            .unwrap_or_else(|| {
                println!("Warning: Failed to parse config, using default");
                Config::default()
            })
    } else {
        println!(
            "Warning: No config file found at {:?}, using default",
            config_path
        );
        Config::default()
    };

    let topology = Arc::new(Mutex::new(Topology::new(config, connected_clients.clone())));

    // Channel for broadcasting events to clients
    let (tx, _rx) = tokio::sync::broadcast::channel::<KvmEvent>(100);

    // Start Input Capture Thread
    let topology_clone = topology.clone();
    let tx_clone = tx.clone();
    std::thread::spawn(move || {
        if let Err(error) = listen(move |event| {
            let mut topo = topology_clone.lock().unwrap();

            // Check for edge switching if local
            if let EventType::MouseMove { x, y } = event.event_type {
                if *topo.get_focus() == Focus::Local {
                    if let Some(new_focus) = topo.check_edge(x, y) {
                        println!("Switching focus to {:?}", new_focus);
                        topo.set_focus(new_focus);
                    }
                }
            }

            match topo.get_focus() {
                Focus::Local => {} // Pass through (listen doesn't block)
                Focus::Client(_) => {
                    // Forward to clients
                    let kvm_event = KvmEvent::from(event.event_type);
                    let _ = tx_clone.send(kvm_event);
                }
            }
        }) {
            println!("Error: {:?}", error);
        }
    });

    let listener = TcpListener::bind(format!("0.0.0.0:{}", port)).await?;
    println!("Server listening on port {}", port);

    // Start service discovery broadcast
    let discovery_port = port;
    tokio::spawn(async move {
        if let Err(e) = crate::discovery::broadcast_server(
            discovery_port,
            "AuroraKVM Server".to_string(),
            crate::net::PROTOCOL_VERSION,
        )
        .await
        {
            eprintln!("Discovery broadcast error: {}", e);
        }
    });

    loop {
        let (stream, addr) = listener.accept().await?;
        println!("Client connected: {}", addr);
        let mut rx = tx.subscribe();
        let _topology_client = topology.clone();
        let connected_clients_clone = connected_clients.clone();

        tokio::spawn(async move {
            let kvm_stream = KvmStream::new(stream);
            let (mut reader, mut writer) = kvm_stream.split();

            // Handshake
            match reader.receive().await {
                Ok(Packet::Handshake {
                    version,
                    screen_info,
                    ..
                }) => {
                    if version != PROTOCOL_VERSION {
                        println!("Client {} version mismatch: {}", addr, version);
                        return;
                    }

                    // Register connected client
                    if let Some(info) = screen_info {
                        connected_clients_clone.lock().unwrap().insert(
                            addr,
                            crate::connected::ConnectedClient {
                                addr,
                                screen_info: info.clone(),
                            },
                        );
                        println!(
                            "Client {} connected: {} ({}x{})",
                            addr, info.name, info.width, info.height
                        );
                    } else {
                        println!("Client {} connected (no screen info)", addr);
                    }
                }
                Ok(_) => {
                    println!("Client {} sent unexpected packet during handshake", addr);
                    return;
                }
                Err(e) => {
                    println!("Client {} handshake error: {}", addr, e);
                    return;
                }
            }

            loop {
                match rx.recv().await {
                    Ok(event) => {
                        let packet = Packet::Event(event);
                        if let Err(e) = writer.send(&packet).await {
                            println!("Failed to send to client {}: {}", addr, e);
                            break;
                        }
                    }
                    Err(e) => {
                        println!("Broadcast error: {}", e);
                        break;
                    }
                }
            }
        });
    }
}
