// VirtualMachine command implementations
//
// These are the fundamental commands for interacting with the JVM

use crate::commands::{command_sets, vm_commands};
use crate::connection::JdwpConnection;
use crate::protocol::{CommandPacket, JdwpResult};
use crate::reader::{read_i32, read_string, read_u32, read_u8};
use crate::types::ReferenceTypeId;
use bytes::BufMut;
use serde::{Deserialize, Serialize};

/// JVM version information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VmVersion {
    pub description: String,
    pub jdwp_major: i32,
    pub jdwp_minor: i32,
    pub vm_version: String,
    pub vm_name: String,
}

/// ID sizes used by the JVM
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VmIdSizes {
    pub field_id_size: i32,
    pub method_id_size: i32,
    pub object_id_size: i32,
    pub reference_type_id_size: i32,
    pub frame_id_size: i32,
}

/// Class information from ClassesBySignature
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClassInfo {
    pub ref_type_tag: u8,  // 1=class, 2=interface, 3=array
    pub type_id: ReferenceTypeId,
    pub signature: String,
    pub status: i32,
}

impl JdwpConnection {
    /// Get JVM version information (VirtualMachine.Version command)
    pub async fn get_version(&mut self) -> JdwpResult<VmVersion> {
        let id = self.next_id();
        let packet = CommandPacket::new(id, command_sets::VIRTUAL_MACHINE, vm_commands::VERSION);

        let reply = self.send_command(packet).await?;
        reply.check_error()?;

        let mut data = reply.data();

        let description = read_string(&mut data)?;
        let jdwp_major = read_i32(&mut data)?;
        let jdwp_minor = read_i32(&mut data)?;
        let vm_version = read_string(&mut data)?;
        let vm_name = read_string(&mut data)?;

        Ok(VmVersion {
            description,
            jdwp_major,
            jdwp_minor,
            vm_version,
            vm_name,
        })
    }

    /// Get ID sizes (VirtualMachine.IDSizes command)
    /// This tells us how many bytes are used for various ID types
    pub async fn get_id_sizes(&mut self) -> JdwpResult<VmIdSizes> {
        let id = self.next_id();
        let packet = CommandPacket::new(id, command_sets::VIRTUAL_MACHINE, vm_commands::ID_SIZES);

        let reply = self.send_command(packet).await?;
        reply.check_error()?;

        let mut data = reply.data();

        let field_id_size = read_i32(&mut data)?;
        let method_id_size = read_i32(&mut data)?;
        let object_id_size = read_i32(&mut data)?;
        let reference_type_id_size = read_i32(&mut data)?;
        let frame_id_size = read_i32(&mut data)?;

        Ok(VmIdSizes {
            field_id_size,
            method_id_size,
            object_id_size,
            reference_type_id_size,
            frame_id_size,
        })
    }

    /// Find classes by signature (VirtualMachine.ClassesBySignature command)
    /// Signature format: "Lcom/example/MyClass;" for classes
    pub async fn classes_by_signature(&mut self, signature: &str) -> JdwpResult<Vec<ClassInfo>> {
        let id = self.next_id();
        let mut packet = CommandPacket::new(id, command_sets::VIRTUAL_MACHINE, vm_commands::CLASSES_BY_SIGNATURE);

        // Write signature as JDWP string (4-byte length + UTF-8 bytes)
        let sig_bytes = signature.as_bytes();
        packet.data.put_u32(sig_bytes.len() as u32);
        packet.data.extend_from_slice(sig_bytes);

        let reply = self.send_command(packet).await?;
        reply.check_error()?;

        let mut data = reply.data();

        // Read number of classes
        let classes_count = read_i32(&mut data)?;
        let mut classes = Vec::with_capacity(classes_count as usize);

        for _ in 0..classes_count {
            let ref_type_tag = read_u8(&mut data)?;
            let type_id = crate::reader::read_u64(&mut data)?;
            let status = read_i32(&mut data)?;

            classes.push(ClassInfo {
                ref_type_tag,
                type_id,
                signature: signature.to_string(),
                status,
            });
        }

        Ok(classes)
    }
}
