// ReferenceType command implementations
//
// Commands for working with classes, interfaces, and arrays

use crate::commands::{command_sets, reference_type_commands};
use crate::connection::JdwpConnection;
use crate::protocol::{CommandPacket, JdwpResult};
use crate::reader::{read_i32, read_string, read_u64};
use crate::types::{MethodId, ReferenceTypeId};
use bytes::BufMut;
use serde::{Deserialize, Serialize};

/// Method information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MethodInfo {
    pub method_id: MethodId,
    pub name: String,
    pub signature: String,
    pub mod_bits: i32,
}

impl JdwpConnection {
    /// Get methods for a reference type (ReferenceType.Methods command)
    pub async fn get_methods(&mut self, ref_type_id: ReferenceTypeId) -> JdwpResult<Vec<MethodInfo>> {
        let id = self.next_id();
        let mut packet = CommandPacket::new(id, command_sets::REFERENCE_TYPE, reference_type_commands::METHODS);

        // Write reference type ID (8 bytes)
        packet.data.put_u64(ref_type_id);

        let reply = self.send_command(packet).await?;
        reply.check_error()?;

        let mut data = reply.data();

        // Read number of methods
        let methods_count = read_i32(&mut data)?;
        let mut methods = Vec::with_capacity(methods_count as usize);

        for _ in 0..methods_count {
            let method_id = read_u64(&mut data)?;
            let name = read_string(&mut data)?;
            let signature = read_string(&mut data)?;
            let mod_bits = read_i32(&mut data)?;

            methods.push(MethodInfo {
                method_id,
                name,
                signature,
                mod_bits,
            });
        }

        Ok(methods)
    }
}
