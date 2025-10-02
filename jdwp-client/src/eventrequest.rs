// EventRequest command implementations
//
// Set up event requests (breakpoints, steps, exceptions, etc.)

use crate::commands::{command_sets, event_commands, event_kinds};
use crate::connection::JdwpConnection;
use crate::protocol::{CommandPacket, JdwpResult};
use crate::reader::read_i32;
use crate::types::{Location, MethodId, ReferenceTypeId};
use bytes::BufMut;

/// Suspend policy for events
#[repr(u8)]
pub enum SuspendPolicy {
    None = 0,
    EventThread = 1,
    All = 2,
}

impl JdwpConnection {
    /// Set a breakpoint at a specific location (EventRequest.Set command)
    /// Returns the request ID for this breakpoint
    pub async fn set_breakpoint(
        &mut self,
        class_id: ReferenceTypeId,
        method_id: MethodId,
        bytecode_index: u64,
        suspend_policy: SuspendPolicy,
    ) -> JdwpResult<i32> {
        let id = self.next_id();
        let mut packet = CommandPacket::new(id, command_sets::EVENT_REQUEST, event_commands::SET);

        // Event kind: BREAKPOINT (2)
        packet.data.put_u8(event_kinds::BREAKPOINT);

        // Suspend policy
        packet.data.put_u8(suspend_policy as u8);

        // Number of modifiers (1 - location only)
        packet.data.put_i32(1);

        // Modifier kind: LocationOnly (7)
        packet.data.put_u8(7);

        // Location:
        // - type tag (1 = class)
        packet.data.put_u8(1);
        // - class ID
        packet.data.put_u64(class_id);
        // - method ID
        packet.data.put_u64(method_id);
        // - index (bytecode position)
        packet.data.put_u64(bytecode_index);

        let reply = self.send_command(packet).await?;
        reply.check_error()?;

        let mut data = reply.data();
        let request_id = read_i32(&mut data)?;

        Ok(request_id)
    }

    /// Clear a breakpoint by request ID (EventRequest.Clear command)
    pub async fn clear_breakpoint(&mut self, request_id: i32) -> JdwpResult<()> {
        let id = self.next_id();
        let mut packet = CommandPacket::new(id, command_sets::EVENT_REQUEST, event_commands::CLEAR);

        // Event kind: BREAKPOINT
        packet.data.put_u8(event_kinds::BREAKPOINT);

        // Request ID
        packet.data.put_i32(request_id);

        let reply = self.send_command(packet).await?;
        reply.check_error()?;

        Ok(())
    }
}
