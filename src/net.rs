use crate::event::KvmEvent;
use anyhow::Result;
use bincode;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;

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
    pub async fn receive(&mut self) -> Result<KvmEvent> {
        let len = self.stream.read_u32().await?;
        let mut buf = vec![0u8; len as usize];
        self.stream.read_exact(&mut buf).await?;
        let event = bincode::deserialize(&buf)?;
        Ok(event)
    }
}

pub struct KvmWriter {
    stream: tokio::net::tcp::OwnedWriteHalf,
}

impl KvmWriter {
    pub async fn send(&mut self, event: &KvmEvent) -> Result<()> {
        let data = bincode::serialize(event)?;
        let len = data.len() as u32;
        self.stream.write_u32(len).await?;
        self.stream.write_all(&data).await?;
        self.stream.flush().await?;
        Ok(())
    }
}
