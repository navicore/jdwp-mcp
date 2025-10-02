// JDWP protocol definitions and packet handling
//
// Reference: https://docs.oracle.com/javase/8/docs/platform/jpda/jdwp/jdwp-protocol.html

use bytes::{Buf, BufMut, BytesMut};
use thiserror::Error;

// JDWP uses big-endian (network byte order) for all multi-byte values
// This is architecture-independent (works on Intel, ARM M1/M2/M3, etc.)

pub type JdwpResult<T> = Result<T, JdwpError>;

#[derive(Debug, Error)]
pub enum JdwpError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Protocol error: {0}")]
    Protocol(String),

    #[error("Invalid handshake")]
    InvalidHandshake,

    #[error("JDWP error code {0}: {1}")]
    JdwpErrorCode(u16, String),

    #[error("Connection closed")]
    ConnectionClosed,
}

// JDWP handshake string
pub const JDWP_HANDSHAKE: &[u8] = b"JDWP-Handshake";

// Packet structure:
// length (4 bytes) - includes header
// id (4 bytes)
// flags (1 byte) - 0x00 = command, 0x80 = reply
// [Command packet: command set (1 byte) + command (1 byte)]
// [Reply packet: error code (2 bytes)]
// data (variable)

pub const HEADER_SIZE: usize = 11;
pub const REPLY_FLAG: u8 = 0x80;

#[derive(Debug, Clone)]
pub struct CommandPacket {
    pub id: u32,
    pub command_set: u8,
    pub command: u8,
    pub data: Vec<u8>,
}

#[derive(Debug, Clone)]
pub struct ReplyPacket {
    pub id: u32,
    pub error_code: u16,
    pub data: Vec<u8>,
}

impl CommandPacket {
    pub fn new(id: u32, command_set: u8, command: u8) -> Self {
        Self {
            id,
            command_set,
            command,
            data: Vec::new(),
        }
    }

    pub fn encode(&self) -> Vec<u8> {
        let length = HEADER_SIZE + self.data.len();
        let mut buf = BytesMut::with_capacity(length);

        buf.put_u32(length as u32);
        buf.put_u32(self.id);
        buf.put_u8(0x00); // command flag
        buf.put_u8(self.command_set);
        buf.put_u8(self.command);
        buf.put_slice(&self.data);

        buf.to_vec()
    }
}

impl ReplyPacket {
    pub fn decode(mut buf: &[u8]) -> JdwpResult<Self> {
        if buf.len() < HEADER_SIZE {
            return Err(JdwpError::Protocol("Reply packet too short".to_string()));
        }

        let _length = buf.get_u32();
        let id = buf.get_u32();
        let flags = buf.get_u8();

        if flags != REPLY_FLAG {
            return Err(JdwpError::Protocol(format!("Invalid reply flag: {:#x}", flags)));
        }

        let error_code = buf.get_u16();
        let data = buf.to_vec();

        Ok(Self {
            id,
            error_code,
            data,
        })
    }

    pub fn is_error(&self) -> bool {
        self.error_code != 0
    }

    pub fn check_error(&self) -> JdwpResult<()> {
        if self.is_error() {
            Err(JdwpError::JdwpErrorCode(
                self.error_code,
                self.error_message().to_string(),
            ))
        } else {
            Ok(())
        }
    }

    pub fn data(&self) -> &[u8] {
        &self.data
    }

    pub fn error_message(&self) -> &'static str {
        match self.error_code {
            0 => "NONE",
            10 => "INVALID_THREAD",
            11 => "INVALID_THREAD_GROUP",
            12 => "INVALID_PRIORITY",
            13 => "THREAD_NOT_SUSPENDED",
            14 => "THREAD_SUSPENDED",
            20 => "INVALID_OBJECT",
            21 => "INVALID_CLASS",
            22 => "CLASS_NOT_PREPARED",
            23 => "INVALID_METHODID",
            24 => "INVALID_LOCATION",
            25 => "INVALID_FIELDID",
            30 => "INVALID_FRAMEID",
            31 => "NO_MORE_FRAMES",
            32 => "OPAQUE_FRAME",
            33 => "NOT_CURRENT_FRAME",
            34 => "TYPE_MISMATCH",
            35 => "INVALID_SLOT",
            40 => "DUPLICATE",
            41 => "NOT_FOUND",
            50 => "INVALID_MONITOR",
            51 => "NOT_MONITOR_OWNER",
            52 => "INTERRUPT",
            60 => "INVALID_CLASS_FORMAT",
            61 => "CIRCULAR_CLASS_DEFINITION",
            62 => "FAILS_VERIFICATION",
            63 => "ADD_METHOD_NOT_IMPLEMENTED",
            64 => "SCHEMA_CHANGE_NOT_IMPLEMENTED",
            65 => "INVALID_TYPESTATE",
            66 => "HIERARCHY_CHANGE_NOT_IMPLEMENTED",
            67 => "DELETE_METHOD_NOT_IMPLEMENTED",
            68 => "UNSUPPORTED_VERSION",
            69 => "NAMES_DONT_MATCH",
            70 => "CLASS_MODIFIERS_CHANGE_NOT_IMPLEMENTED",
            71 => "METHOD_MODIFIERS_CHANGE_NOT_IMPLEMENTED",
            99 => "NOT_IMPLEMENTED",
            100 => "NULL_POINTER",
            101 => "ABSENT_INFORMATION",
            102 => "INVALID_EVENT_TYPE",
            103 => "ILLEGAL_ARGUMENT",
            110 => "OUT_OF_MEMORY",
            111 => "ACCESS_DENIED",
            112 => "VM_DEAD",
            113 => "INTERNAL",
            115 => "UNATTACHED_THREAD",
            500 => "INVALID_TAG",
            502 => "ALREADY_INVOKING",
            503 => "INVALID_INDEX",
            504 => "INVALID_LENGTH",
            506 => "INVALID_STRING",
            507 => "INVALID_CLASS_LOADER",
            508 => "INVALID_ARRAY",
            509 => "TRANSPORT_LOAD",
            510 => "TRANSPORT_INIT",
            511 => "NATIVE_METHOD",
            512 => "INVALID_COUNT",
            _ => "UNKNOWN_ERROR",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_command_packet_encode() {
        let packet = CommandPacket::new(1, 1, 1);
        let encoded = packet.encode();

        assert_eq!(encoded.len(), HEADER_SIZE);
        assert_eq!(&encoded[0..4], &[0, 0, 0, 11]); // length (big-endian)
        assert_eq!(&encoded[4..8], &[0, 0, 0, 1]); // id (big-endian)
        assert_eq!(encoded[8], 0x00); // command flag
        assert_eq!(encoded[9], 1); // command set
        assert_eq!(encoded[10], 1); // command
    }

    #[test]
    fn test_big_endian_encoding() {
        // Verify we're using big-endian (network byte order)
        // This test ensures architecture independence (Intel vs ARM M1/M2/M3)
        let packet = CommandPacket::new(0x12345678, 1, 1);
        let encoded = packet.encode();

        // ID should be encoded as big-endian: 0x12345678
        assert_eq!(&encoded[4..8], &[0x12, 0x34, 0x56, 0x78]);

        // NOT little-endian (which would be [0x78, 0x56, 0x34, 0x12])
        assert_ne!(&encoded[4..8], &[0x78, 0x56, 0x34, 0x12]);
    }

    #[test]
    fn test_reply_packet_decode() {
        // Construct a reply packet manually with big-endian values
        let mut reply_data = vec![
            0, 0, 0, 11,  // length = 11 (big-endian)
            0, 0, 0, 1,   // id = 1 (big-endian)
            0x80,         // reply flag
            0, 0,         // error code = 0 (big-endian)
        ];

        let packet = ReplyPacket::decode(&reply_data).unwrap();
        assert_eq!(packet.id, 1);
        assert_eq!(packet.error_code, 0);
        assert!(!packet.is_error());
    }
}
