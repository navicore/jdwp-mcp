// JDWP connection management
//
// Handles TCP connection, handshake, and event loop startup

use crate::eventloop::{spawn_event_loop, EventLoopHandle};
use crate::events::EventSet;
use crate::protocol::*;
use std::sync::atomic::{AtomicU32, Ordering};
use std::sync::Arc;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;
use tracing::{debug, info, warn};

#[derive(Debug)]
pub struct JdwpConnection {
    event_loop: EventLoopHandle,
    next_id: Arc<AtomicU32>,
}

impl JdwpConnection {
    /// Connect to a JVM via JDWP
    pub async fn connect(host: &str, port: u16) -> JdwpResult<Self> {
        info!("Connecting to JDWP at {}:{}", host, port);

        let mut stream = TcpStream::connect((host, port)).await?;

        // Perform JDWP handshake
        Self::handshake(&mut stream).await?;

        // Split stream and spawn event loop
        let (reader, writer) = stream.into_split();
        let event_loop = spawn_event_loop(reader, writer);

        Ok(Self {
            event_loop,
            next_id: Arc::new(AtomicU32::new(1)),
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
        debug!("Sending command packet id={}", packet.id);
        self.event_loop.send_command(packet).await
    }

    /// Try to receive an event (non-blocking)
    pub async fn try_recv_event(&self) -> Option<EventSet> {
        self.event_loop.try_recv_event().await
    }

    /// Wait for the next event (blocking)
    pub async fn recv_event(&self) -> Option<EventSet> {
        self.event_loop.recv_event().await
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
