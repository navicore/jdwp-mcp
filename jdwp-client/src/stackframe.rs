// StackFrame command implementations
//
// Commands for inspecting stack frame variables

use crate::commands::{command_sets, stack_frame_commands};
use crate::connection::JdwpConnection;
use crate::protocol::{CommandPacket, JdwpResult};
use crate::reader::{read_u64, read_u8};
use crate::types::{FrameId, ThreadId, Value, ValueData};
use bytes::{Buf, BufMut};
use serde::{Deserialize, Serialize};

/// Variable slot information for GetValues
#[derive(Debug, Clone)]
pub struct VariableSlot {
    pub slot: i32,
    pub sig_byte: u8,
}

impl JdwpConnection {
    /// Get values for variable slots in a frame (StackFrame.GetValues command)
    pub async fn get_frame_values(
        &mut self,
        thread_id: ThreadId,
        frame_id: FrameId,
        slots: Vec<VariableSlot>,
    ) -> JdwpResult<Vec<Value>> {
        let id = self.next_id();
        let mut packet = CommandPacket::new(id, command_sets::STACK_FRAME, stack_frame_commands::GET_VALUES);

        // Write thread ID and frame ID
        packet.data.put_u64(thread_id);
        packet.data.put_u64(frame_id);

        // Number of slots to retrieve
        packet.data.put_i32(slots.len() as i32);

        // Write each slot
        for slot in &slots {
            packet.data.put_i32(slot.slot);
            packet.data.put_u8(slot.sig_byte);
        }

        let reply = self.send_command(packet).await?;
        reply.check_error()?;

        let mut data = reply.data();

        // Read number of values (should match slots.len())
        let values_count = crate::reader::read_i32(&mut data)?;
        let mut values = Vec::with_capacity(values_count as usize);

        for _ in 0..values_count {
            let tag = read_u8(&mut data)?;
            let value_data = read_value_by_tag(tag, &mut data)?;

            values.push(Value {
                tag,
                data: value_data,
            });
        }

        Ok(values)
    }
}

/// Read a value based on its type tag
fn read_value_by_tag(tag: u8, buf: &mut &[u8]) -> JdwpResult<ValueData> {
    match tag {
        // 'B' = byte
        66 => Ok(ValueData::Byte(buf.get_i8())),
        // 'C' = char
        67 => Ok(ValueData::Char(buf.get_u16())),
        // 'D' = double
        68 => Ok(ValueData::Double(buf.get_f64())),
        // 'F' = float
        70 => Ok(ValueData::Float(buf.get_f32())),
        // 'I' = int
        73 => Ok(ValueData::Int(buf.get_i32())),
        // 'J' = long
        74 => Ok(ValueData::Long(buf.get_i64())),
        // 'S' = short
        83 => Ok(ValueData::Short(buf.get_i16())),
        // 'Z' = boolean
        90 => Ok(ValueData::Boolean(buf.get_u8() != 0)),
        // 'V' = void
        86 => Ok(ValueData::Void),
        // Object types (L, s, t, g, l, c, [)
        76 | 115 | 116 | 103 | 108 | 99 | 91 => {
            let object_id = read_u64(buf)?;
            Ok(ValueData::Object(object_id))
        }
        _ => Err(crate::protocol::JdwpError::Protocol(format!("Unknown value tag: {}", tag))),
    }
}
