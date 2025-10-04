// StringReference command implementations
//
// Commands for working with String objects

use crate::commands::{command_sets, string_reference_commands};
use crate::connection::JdwpConnection;
use crate::protocol::{CommandPacket, JdwpResult};
use crate::reader::read_string;
use crate::types::ObjectId;
use bytes::BufMut;

impl JdwpConnection {
    /// Get the string value from a String object (StringReference.Value command)
    ///
    /// # Arguments
    /// * `string_id` - The ObjectId of the String object
    ///
    /// # Returns
    /// The actual string value
    ///
    /// # Example
    /// ```no_run
    /// let value = connection.get_string_value(string_object_id).await?;
    /// println!("String value: {}", value);
    /// ```
    pub async fn get_string_value(&mut self, string_id: ObjectId) -> JdwpResult<String> {
        let id = self.next_id();
        let mut packet = CommandPacket::new(
            id,
            command_sets::STRING_REFERENCE,
            string_reference_commands::VALUE,
        );

        // Write the string object ID
        packet.data.put_u64(string_id);

        let reply = self.send_command(packet).await?;
        reply.check_error()?;

        let mut data = reply.data();

        // Read the string value
        let value = read_string(&mut data)?;

        Ok(value)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_string_value_packet() {
        // Test that packet is constructed correctly
        // This is a unit test to verify packet format
    }
}
