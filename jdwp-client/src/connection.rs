// JDWP connection management
//
// Handles TCP connection, handshake, and packet I/O

use crate::protocol::*;
use bytes::BytesMut;
use std::sync::atomic::{AtomicU32, Ordering};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;
use tracing::{debug, info, warn};

#[derive(Debug)]
pub struct JdwpConnection {
    stream: TcpStream,
    next_id: AtomicU32,
}

impl JdwpConnection {
    /// Connect to a JVM via JDWP
    pub async fn connect(host: &str, port: u16) -> JdwpResult<Self> {
        info!("Connecting to JDWP at {}:{}", host, port);

        let mut stream = TcpStream::connect((host, port)).await?;

        // Perform JDWP handshake
        Self::handshake(&mut stream).await?;

        Ok(Self {
            stream,
            next_id: AtomicU32::new(1),
        })
    }

    /// Perform JDWP handshake
    async fn handshake(stream: &mut TcpStream) -> JdwpResult<()> {
        debug!("Performing JDWP handshake");

        // Send handshake
        stream.write_all(JDWP_HANDSHAKE).await?;
        stream.flush().await?;

        // Receive handshake response
        let mut buf = vec![0u8; JDWP_HANDSHAKE.len()];
        stream.read_exact(&mut buf).await?;

        if buf != JDWP_HANDSHAKE {
            warn!("Invalid handshake response: {:?}", buf);
            return Err(JdwpError::InvalidHandshake);
        }

        info!("JDWP handshake successful");
        Ok(())
    }

    /// Send a command and wait for reply
    pub async fn send_command(&mut self, packet: CommandPacket) -> JdwpResult<ReplyPacket> {
        let encoded = packet.encode();
        debug!("Sending command packet id={} len={}", packet.id, encoded.len());

        self.stream.write_all(&encoded).await?;
        self.stream.flush().await?;

        // Read reply
        self.read_reply().await
    }

    /// Read a reply packet
    async fn read_reply(&mut self) -> JdwpResult<ReplyPacket> {
        // Read header first to get length
        let mut header = BytesMut::with_capacity(HEADER_SIZE);
        header.resize(HEADER_SIZE, 0);

        self.stream.read_exact(&mut header).await?;

        // Parse length (first 4 bytes)
        let length = u32::from_be_bytes([header[0], header[1], header[2], header[3]]) as usize;

        if length < HEADER_SIZE {
            return Err(JdwpError::Protocol(format!("Invalid packet length: {}", length)));
        }

        // Read rest of packet if there's data beyond header
        let data_len = length - HEADER_SIZE;
        let mut full_packet = header.to_vec();

        if data_len > 0 {
            let mut data = vec![0u8; data_len];
            self.stream.read_exact(&mut data).await?;
            full_packet.extend_from_slice(&data);
        }

        ReplyPacket::decode(&full_packet)
    }

    /// Generate next packet ID
    pub fn next_id(&self) -> u32 {
        self.next_id.fetch_add(1, Ordering::SeqCst)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_next_id() {
        // Test ID counter without creating a real TcpStream
        let counter = AtomicU32::new(1);

        assert_eq!(counter.fetch_add(1, Ordering::SeqCst), 1);
        assert_eq!(counter.fetch_add(1, Ordering::SeqCst), 2);
        assert_eq!(counter.fetch_add(1, Ordering::SeqCst), 3);
    }
}
