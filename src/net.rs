use crate::event::KvmEvent;
use anyhow::{Result, anyhow};
use bincode;
use serde::{Deserialize, Serialize};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;

const MAX_FRAME_SIZE: u32 = 1024 * 1024; // 1MB
pub const PROTOCOL_VERSION: u32 = 1;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ScreenInfo {
    pub width: u32,
    pub height: u32,
    pub x: i32,
    pub y: i32,
    pub name: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub enum Packet {
    Handshake {
        version: u32,
        secret: Option<String>,
        screen_info: Option<ScreenInfo>,
    },
    Event(KvmEvent),
    Heartbeat,
}

pub struct KvmStream {
    stream: TcpStream,
}

impl KvmStream {
    pub fn new(stream: TcpStream) -> Self {
        Self { stream }
    }

    pub fn split(self) -> (KvmReader, KvmWriter) {
        let (read, write) = self.stream.into_split();
        (KvmReader { stream: read }, KvmWriter { stream: write })
    }
}

pub struct KvmReader {
    stream: tokio::net::tcp::OwnedReadHalf,
}

impl KvmReader {
    pub async fn receive(&mut self) -> Result<Packet> {
        let len = self.stream.read_u32().await?;
        if len > MAX_FRAME_SIZE {
            return Err(anyhow!("Frame size too large: {}", len));
        }
        let mut buf = vec![0u8; len as usize];
        self.stream.read_exact(&mut buf).await?;
        let packet = bincode::deserialize(&buf)?;
        Ok(packet)
    }
}

pub struct KvmWriter {
    stream: tokio::net::tcp::OwnedWriteHalf,
}

impl KvmWriter {
    pub async fn send(&mut self, packet: &Packet) -> Result<()> {
        let data = bincode::serialize(packet)?;
        let len = data.len() as u32;
        self.stream.write_u32(len).await?;
        self.stream.write_all(&data).await?;
        self.stream.flush().await?;
        Ok(())
    }
}
