// ThreadReference command implementations
//
// Commands for working with threads (frames, status, suspend/resume)

use crate::commands::{command_sets, thread_commands};
use crate::connection::JdwpConnection;
use crate::protocol::{CommandPacket, JdwpResult};
use crate::reader::{read_i32, read_u64};
use crate::types::{FrameId, Location, MethodId, ReferenceTypeId, ThreadId};
use bytes::BufMut;
use serde::{Deserialize, Serialize};

/// Stack frame information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Frame {
    pub frame_id: FrameId,
    pub location: Location,
}

impl JdwpConnection {
    /// Get stack frames for a thread (ThreadReference.Frames command)
    pub async fn get_frames(
        &mut self,
        thread_id: ThreadId,
        start_frame: i32,
        length: i32,
    ) -> JdwpResult<Vec<Frame>> {
        let id = self.next_id();
        let mut packet = CommandPacket::new(id, command_sets::THREAD_REFERENCE, thread_commands::FRAMES);

        // Write thread ID
        packet.data.put_u64(thread_id);
        // Start frame (0 = current/top frame)
        packet.data.put_i32(start_frame);
        // Length (-1 = all frames)
        packet.data.put_i32(length);

        let reply = self.send_command(packet).await?;
        reply.check_error()?;

        let mut data = reply.data();

        // Read number of frames
        let frames_count = read_i32(&mut data)?;
        let mut frames = Vec::with_capacity(frames_count as usize);

        for _ in 0..frames_count {
            let frame_id = read_u64(&mut data)?;

            // Read location
            let type_tag = crate::reader::read_u8(&mut data)?;
            let class_id = read_u64(&mut data)?;
            let method_id = read_u64(&mut data)?;
            let index = read_u64(&mut data)?;

            frames.push(Frame {
                frame_id,
                location: Location {
                    type_tag,
                    class_id,
                    method_id,
                    index,
                },
            });
        }

        Ok(frames)
    }

    /// Get all threads (VirtualMachine.AllThreads)
    pub async fn get_all_threads(&mut self) -> JdwpResult<Vec<ThreadId>> {
        let id = self.next_id();
        let packet = CommandPacket::new(id, command_sets::VIRTUAL_MACHINE, crate::commands::vm_commands::ALL_THREADS);

        let reply = self.send_command(packet).await?;
        reply.check_error()?;

        let mut data = reply.data();

        let threads_count = read_i32(&mut data)?;
        let mut threads = Vec::with_capacity(threads_count as usize);

        for _ in 0..threads_count {
            threads.push(read_u64(&mut data)?);
        }

        Ok(threads)
    }

    /// Suspend all threads (VirtualMachine.Suspend)
    pub async fn suspend_all(&mut self) -> JdwpResult<()> {
        let id = self.next_id();
        let packet = CommandPacket::new(id, command_sets::VIRTUAL_MACHINE, crate::commands::vm_commands::SUSPEND);

        let reply = self.send_command(packet).await?;
        reply.check_error()?;

        Ok(())
    }

    /// Resume all threads (VirtualMachine.Resume)
    pub async fn resume_all(&mut self) -> JdwpResult<()> {
        let id = self.next_id();
        let packet = CommandPacket::new(id, command_sets::VIRTUAL_MACHINE, crate::commands::vm_commands::RESUME);

        let reply = self.send_command(packet).await?;
        reply.check_error()?;

        Ok(())
    }
}
