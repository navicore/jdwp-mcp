// JDWP Event Loop
//
// Handles concurrent reading of events and replies from JDWP socket

use crate::events::{parse_event_packet, EventSet};
use crate::protocol::{CommandPacket, JdwpError, JdwpResult, ReplyPacket, HEADER_SIZE, REPLY_FLAG};
use bytes::BytesMut;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::tcp::{OwnedReadHalf, OwnedWriteHalf};
use tokio::sync::{mpsc, oneshot};
use tracing::{debug, error, info, warn};

/// Maximum allowed JDWP packet size (10MB)
/// This prevents memory exhaustion from malicious or buggy JVMs
const MAX_PACKET_SIZE: usize = 10 * 1024 * 1024;

/// Maximum time to wait for a command reply before considering it lost
const REPLY_TIMEOUT: std::time::Duration = std::time::Duration::from_secs(30);

/// Request to send a command and get reply
pub struct CommandRequest {
    pub packet: CommandPacket,
    pub reply_tx: oneshot::Sender<JdwpResult<ReplyPacket>>,
}

/// Handle to the event loop for sending commands and receiving events.
///
/// This handle can be cloned to send commands from multiple tasks, but only ONE clone
/// should call `recv_event()` or `try_recv_event()` at a time. The event receiver is
/// wrapped in an Arc<Mutex<Receiver>> which allows sharing, but concurrent event
/// consumption from multiple tasks will lead to unpredictable behavior (events distributed
/// round-robin across consumers).
///
/// # Thread Safety
/// - Commands can be sent concurrently from multiple clones
/// - Events should be consumed from a single task/clone
///
/// # Example
/// ```no_run
/// // Good: Single event consumer
/// let handle1 = event_loop.clone();
/// let handle2 = event_loop.clone();
///
/// // Both can send commands
/// handle1.send_command(cmd1);
/// handle2.send_command(cmd2);
///
/// // Only one should consume events
/// while let Some(event) = handle1.recv_event().await {
///     // Process event
/// }
/// ```
#[derive(Clone, Debug)]
pub struct EventLoopHandle {
    command_tx: mpsc::Sender<CommandRequest>,
    event_rx: Arc<tokio::sync::Mutex<mpsc::Receiver<EventSet>>>,
}

impl EventLoopHandle {
    /// Send a command and wait for reply
    pub async fn send_command(&self, packet: CommandPacket) -> JdwpResult<ReplyPacket> {
        let (reply_tx, reply_rx) = oneshot::channel();

        let request = CommandRequest { packet, reply_tx };

        self.command_tx
            .send(request)
            .await
            .map_err(|_| JdwpError::Protocol("Event loop shut down".to_string()))?;

        reply_rx
            .await
            .map_err(|_| JdwpError::Protocol("Reply channel closed".to_string()))?
    }

    /// Try to receive an event (non-blocking)
    pub async fn try_recv_event(&self) -> Option<EventSet> {
        let mut rx = self.event_rx.lock().await;
        rx.try_recv().ok()
    }

    /// Wait for the next event (blocking)
    pub async fn recv_event(&self) -> Option<EventSet> {
        let mut rx = self.event_rx.lock().await;
        rx.recv().await
    }
}

/// Start the event loop task
pub fn spawn_event_loop(reader: OwnedReadHalf, writer: OwnedWriteHalf) -> EventLoopHandle {
    let (command_tx, command_rx) = mpsc::channel(32);
    // Use larger buffer for events to avoid loss under load
    // Events are critical (breakpoints, exceptions) and shouldn't be dropped
    let (event_tx, event_rx) = mpsc::channel(256);

    tokio::spawn(event_loop_task(reader, writer, command_rx, event_tx));

    EventLoopHandle {
        command_tx,
        event_rx: Arc::new(tokio::sync::Mutex::new(event_rx)),
    }
}

/// Pending reply with timestamp for timeout tracking
struct PendingReply {
    sender: oneshot::Sender<JdwpResult<ReplyPacket>>,
    sent_at: tokio::time::Instant,
}

/// Main event loop task
async fn event_loop_task(
    mut reader: OwnedReadHalf,
    mut writer: OwnedWriteHalf,
    mut command_rx: mpsc::Receiver<CommandRequest>,
    event_tx: mpsc::Sender<EventSet>,
) {
    info!("Event loop started");

    let mut pending_replies: HashMap<u32, PendingReply> = HashMap::new();
    let mut cleanup_interval = tokio::time::interval(tokio::time::Duration::from_secs(10));

    loop {
        tokio::select! {
            // Handle outgoing commands
            Some(cmd) = command_rx.recv() => {
                let packet_id = cmd.packet.id;
                debug!("Sending command id={}", packet_id);

                let encoded = cmd.packet.encode();
                if let Err(e) = writer.write_all(&encoded).await {
                    error!("Failed to write command: {}", e);
                    cmd.reply_tx.send(Err(JdwpError::Io(e))).ok();
                    continue;
                }

                if let Err(e) = writer.flush().await {
                    error!("Failed to flush command: {}", e);
                    cmd.reply_tx.send(Err(JdwpError::Io(e))).ok();
                    continue;
                }

                pending_replies.insert(packet_id, PendingReply {
                    sender: cmd.reply_tx,
                    sent_at: tokio::time::Instant::now(),
                });
            }

            // Periodic cleanup of timed-out pending replies
            _ = cleanup_interval.tick() => {
                let now = tokio::time::Instant::now();
                let before_count = pending_replies.len();

                pending_replies.retain(|packet_id, pending| {
                    let elapsed = now.duration_since(pending.sent_at);
                    if elapsed > REPLY_TIMEOUT {
                        warn!("Command {} timed out after {:?}, removing from pending replies", packet_id, elapsed);
                        // Note: sender is dropped here, which will notify the waiting command
                        false
                    } else {
                        true
                    }
                });

                let removed = before_count - pending_replies.len();
                if removed > 0 {
                    warn!("Cleaned up {} timed-out pending replies", removed);
                }
            }

            // Handle incoming packets
            result = read_packet(&mut reader) => {
                match result {
                    Ok((is_reply, packet_id, data)) => {
                        if is_reply {
                            // It's a reply - route to waiting command
                            debug!("Received reply id={}", packet_id);

                            if let Some(pending) = pending_replies.remove(&packet_id) {
                                match ReplyPacket::decode(&data) {
                                    Ok(reply) => {
                                        pending.sender.send(Ok(reply)).ok();
                                    }
                                    Err(e) => {
                                        warn!("Failed to decode reply: {}", e);
                                        pending.sender.send(Err(e)).ok();
                                    }
                                }
                            } else {
                                warn!("Received reply for unknown command id={} (may have timed out)", packet_id);
                            }
                        } else {
                            // It's an event - parse and broadcast
                            debug!("Received event packet, len={}", data.len());

                            // Event packets have command_set and command in header
                            // Data starts after 11-byte header
                            let event_data = &data[HEADER_SIZE..];

                            match parse_event_packet(event_data) {
                                Ok(event_set) => {
                                    info!("Parsed event set: {} events, suspend_policy={}",
                                          event_set.events.len(), event_set.suspend_policy);

                                    // Send event, blocking if channel is full
                                    // This ensures critical events (breakpoints, exceptions) are never lost
                                    // The JVM is already suspended when events occur, so blocking here is acceptable
                                    if (event_tx.send(event_set).await).is_err() {
                                        info!("Event receiver dropped, shutting down event loop");
                                        break;
                                    }
                                }
                                Err(e) => {
                                    warn!("Failed to parse event: {}", e);
                                }
                            }
                        }
                    }
                    Err(e) => {
                        error!("Failed to read packet: {}", e);
                        break;
                    }
                }
            }
        }
    }

    info!("Event loop shutting down");
}

/// Read a packet from the socket and determine if it's a reply or event
async fn read_packet(reader: &mut OwnedReadHalf) -> JdwpResult<(bool, u32, Vec<u8>)> {
    // Read header
    let mut header = BytesMut::with_capacity(HEADER_SIZE);
    header.resize(HEADER_SIZE, 0);

    reader
        .read_exact(&mut header)
        .await
        .map_err(JdwpError::Io)?;

    // Parse header
    let length = u32::from_be_bytes([header[0], header[1], header[2], header[3]]) as usize;
    let packet_id = u32::from_be_bytes([header[4], header[5], header[6], header[7]]);
    let flags = header[8];

    if length < HEADER_SIZE {
        return Err(JdwpError::Protocol(format!(
            "Invalid packet length: {}",
            length
        )));
    }

    if length > MAX_PACKET_SIZE {
        return Err(JdwpError::Protocol(format!(
            "Packet too large: {} bytes (max: {} bytes)",
            length, MAX_PACKET_SIZE
        )));
    }

    // Read rest of packet
    let data_len = length - HEADER_SIZE;
    let mut full_packet = header.to_vec();

    if data_len > 0 {
        let mut data = vec![0u8; data_len];
        reader.read_exact(&mut data).await.map_err(JdwpError::Io)?;
        full_packet.extend_from_slice(&data);
    }

    let is_reply = flags == REPLY_FLAG;

    Ok((is_reply, packet_id, full_packet))
}
