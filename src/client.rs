use crate::net::{KvmStream, PROTOCOL_VERSION, Packet};
use anyhow::Result;
use rdev::{EventType, simulate};
use tokio::net::TcpStream;

pub async fn run(host: String) -> Result<()> {
    println!("Connecting to {}", host);
    let stream = TcpStream::connect(host).await?;
    let kvm_stream = KvmStream::new(stream);
    let (mut reader, mut writer) = kvm_stream.split();

    // Detect screen info
    let screen_info = detect_screen_info();

    // Send Handshake with screen info
    let handshake = Packet::Handshake {
        version: PROTOCOL_VERSION,
        secret: None,
        screen_info: Some(screen_info),
    };
    writer.send(&handshake).await?;

    // Receive loop
    loop {
        match reader.receive().await {
            Ok(Packet::Event(event)) => {
                let event_type = EventType::from(event);
                if let Err(e) = simulate(&event_type) {
                    println!("Failed to simulate event: {:?}", e);
                }
            }
            Ok(_) => {} // Ignore other packets for now
            Err(e) => {
                println!("Connection lost: {}", e);
                break;
            }
        }
    }
    Ok(())
}

fn detect_screen_info() -> crate::net::ScreenInfo {
    use display_info::DisplayInfo;

    // Get primary display or first available
    match DisplayInfo::all() {
        Ok(displays) if !displays.is_empty() => {
            let display = &displays[0];
            crate::net::ScreenInfo {
                width: display.width,
                height: display.height,
                x: 0,
                y: 0,
                name: hostname::get()
                    .ok()
                    .and_then(|h| h.into_string().ok())
                    .unwrap_or_else(|| "Unknown Client".to_string()),
            }
        }
        _ => {
            // Fallback
            crate::net::ScreenInfo {
                width: 1920,
                height: 1080,
                x: 0,
                y: 0,
                name: hostname::get()
                    .ok()
                    .and_then(|h| h.into_string().ok())
                    .unwrap_or_else(|| "Unknown Client".to_string()),
            }
        }
    }
}
