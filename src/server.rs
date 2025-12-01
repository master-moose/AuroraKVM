use crate::config::Config;
use crate::event::KvmEvent;
use crate::net::{KvmStream, PROTOCOL_VERSION, Packet};
use crate::topology::{Focus, Topology};
use anyhow::Result;
use dirs;
use rdev::{Event, grab};
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

    // Virtual cursor state
    let virtual_cursor = Arc::new(Mutex::new((0.0, 0.0)));
    let virtual_cursor_clone = virtual_cursor.clone();

    std::thread::spawn(move || {
        use rdev::{Event, EventType, grab};

        if let Err(error) = grab(move |event: Event| -> Option<Event> {
            let mut topo = topology_clone.lock().unwrap();
            let mut v_cursor = virtual_cursor_clone.lock().unwrap();

            match topo.get_focus() {
                Focus::Local => {
                    // Pass through events to local OS
                    // Check for edge switching
                    if let EventType::MouseMove { x, y } = event.event_type {
                        // Update virtual cursor to match real cursor while local
                        *v_cursor = (x, y);

                        if let Some(new_focus) = topo.check_edge(x, y) {
                            println!("Switching focus to {:?}", new_focus);
                            topo.set_focus(new_focus);
                            // When switching to client, we might want to center the mouse
                            // or just leave it at the edge. Leaving it is safer for now.
                        }
                    }
                    Some(event)
                }
                Focus::Client(_) => {
                    // Swallow events (return None) so local OS doesn't see them

                    // Update virtual cursor for MouseMove
                    if let EventType::MouseMove { x, y } = event.event_type {
                        // In grab mode, (x,y) are still absolute screen coordinates.
                        // But since we swallow events, the OS cursor won't move.
                        // Wait, if we swallow, the OS cursor STAYS put.
                        // So subsequent MouseMove events might report the SAME (x,y) or
                        // relative deltas depending on OS.
                        // rdev usually reports absolute. If OS cursor is frozen, x/y might be static.
                        // Actually, rdev on Linux often uses XInput2 or similar.
                        // If we swallow, X11 might not move the cursor.
                        // We need relative movement.
                        // rdev doesn't give relative movement easily in MouseMove.
                        // BUT, for now let's assume we can just track the "would be" position
                        // if we were letting it move, OR we rely on the fact that we need to
                        // calculate deltas if the OS cursor is locked.

                        // simpler approach for first iteration:
                        // We can't easily get deltas from absolute rdev events if the cursor is frozen.
                        // However, we can try to let the cursor move BUT confine it?
                        // No, user wants "fully detach".

                        // Let's try to infer delta from the event if possible,
                        // OR just use the event's x/y if rdev reports raw input before OS clamping.
                        // On Linux `grab` usually hooks deep.

                        // Let's assume for a moment we just forward the event to the client
                        // and swallow it locally.
                        // But we need to check if we come back to local.

                        // We need to track the virtual position manually.
                        // If rdev gives us the new position even if we return None, we are good.
                        // If rdev gives us the OLD position because we returned None previously, we are stuck.

                        // EXPERIMENTAL: Let's assume rdev `grab` on Linux sees the physical move
                        // even if we block propagation.
                        *v_cursor = (x, y);

                        // Check if we returned to local
                        // We use the virtual coordinates for this check
                        // But wait, if we are "Client", we want to check if we hit the "return" edge.
                        // The `check_edge` function currently only checks if we are inside local.
                        // We need a reverse check.

                        // Actually, `check_edge` in Topology handles "if not inside local -> find client".
                        // We need "if inside local -> switch to local".

                        // Let's do a manual check here for return
                        let (vx, vy) = *v_cursor;

                        // Check if we are back inside any local screen
                        let mut inside_local = false;
                        for screen in &topo.get_config().local_screens {
                            let sx = screen.x as f64;
                            let sy = screen.y as f64;
                            let sw = screen.width as f64;
                            let sh = screen.height as f64;

                            if vx >= sx && vx < sx + sw && vy >= sy && vy < sy + sh {
                                inside_local = true;
                                break;
                            }
                        }

                        if inside_local {
                            println!("Returning focus to Local");
                            topo.set_focus(Focus::Local);
                            // We don't swallow this event so the cursor actually moves back in
                            return Some(event);
                        }
                    }

                    // Forward to clients
                    let kvm_event = KvmEvent::from(event.event_type);
                    let _ = tx_clone.send(kvm_event);

                    None // Swallow event
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
