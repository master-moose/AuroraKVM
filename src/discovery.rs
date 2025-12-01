use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use tokio::net::UdpSocket;
use tokio::time::{Duration, sleep};

const DISCOVERY_PORT: u16 = 8079;
const BROADCAST_INTERVAL_SECS: u64 = 2;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ServerAnnouncement {
    pub name: String,
    pub port: u16,
    pub version: u32,
}

/// Server: Broadcast service announcements on the network
pub async fn broadcast_server(port: u16, name: String, version: u32) -> Result<()> {
    let socket = UdpSocket::bind("0.0.0.0:0").await?;
    socket.set_broadcast(true)?;

    let announcement = ServerAnnouncement {
        name,
        port,
        version,
    };

    let broadcast_addr = SocketAddr::new(
        IpAddr::V4(Ipv4Addr::new(255, 255, 255, 255)),
        DISCOVERY_PORT,
    );

    loop {
        let data = bincode::serialize(&announcement)?;
        let _ = socket.send_to(&data, broadcast_addr).await;
        sleep(Duration::from_secs(BROADCAST_INTERVAL_SECS)).await;
    }
}

/// Client: Discover servers on the network
pub async fn discover_servers(timeout_secs: u64) -> Result<Vec<(ServerAnnouncement, IpAddr)>> {
    let socket = UdpSocket::bind(format!("0.0.0.0:{}", DISCOVERY_PORT)).await?;
    let mut discovered = Vec::new();
    let mut buf = [0u8; 1024];

    let start = std::time::Instant::now();

    while start.elapsed().as_secs() < timeout_secs {
        tokio::select! {
            result = socket.recv_from(&mut buf) => {
                if let Ok((len, addr)) = result {
                    if let Ok(announcement) = bincode::deserialize::<ServerAnnouncement>(&buf[..len]) {
                        // Only accept from local network
                        if is_local_network(addr.ip()) {
                            // Avoid duplicates
                            if !discovered.iter().any(|(_, ip)| *ip == addr.ip()) {
                                discovered.push((announcement, addr.ip()));
                            }
                        }
                    }
                }
            }
            _ = sleep(Duration::from_millis(100)) => {
                // Continue listening
            }
        }
    }

    Ok(discovered)
}

fn is_local_network(ip: IpAddr) -> bool {
    match ip {
        IpAddr::V4(ipv4) => {
            let octets = ipv4.octets();
            // 192.168.x.x
            (octets[0] == 192 && octets[1] == 168) ||
            // 10.x.x.x
            octets[0] == 10 ||
            // 172.16.x.x - 172.31.x.x
            (octets[0] == 172 && octets[1] >= 16 && octets[1] <= 31) ||
            // localhost
            octets[0] == 127
        }
        IpAddr::V6(_) => false, // For now, only support IPv4
    }
}
