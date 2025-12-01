use crate::net::{KvmStream, PROTOCOL_VERSION, Packet};
use anyhow::Result;
use rdev::{EventType, simulate};
use tokio::net::TcpStream;

pub async fn run(host: String, secret: Option<String>) -> Result<()> {
    println!("Connecting to {}", host);
    let stream = TcpStream::connect(host).await?;
    let kvm_stream = KvmStream::new(stream);
    let (mut reader, mut writer) = kvm_stream.split();

    // Send Handshake
    let handshake = Packet::Handshake {
        version: PROTOCOL_VERSION,
        secret,
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
