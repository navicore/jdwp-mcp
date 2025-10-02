// JDWP event handling
//
// Events are sent from the JVM to notify about breakpoints, steps, etc.

use crate::commands::event_kinds;
use crate::protocol::{JdwpError, JdwpResult};
use crate::reader::{read_i32, read_u64, read_u8};
use crate::types::*;
use serde::{Deserialize, Serialize};
use tracing::warn;

/// Composite event packet (can contain multiple events)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventSet {
    pub suspend_policy: u8,
    pub events: Vec<Event>,
}

/// Single event within an event set
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Event {
    pub kind: u8,
    pub request_id: i32,
    pub details: EventKind,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum EventKind {
    VMStart {
        thread: ThreadId,
    },
    VMDeath,
    ThreadStart {
        thread: ThreadId,
    },
    ThreadDeath {
        thread: ThreadId,
    },
    ClassPrepare {
        thread: ThreadId,
        ref_type: ReferenceTypeId,
        signature: String,
        status: i32,
    },
    Breakpoint {
        thread: ThreadId,
        location: Location,
    },
    Step {
        thread: ThreadId,
        location: Location,
    },
    Exception {
        thread: ThreadId,
        location: Location,
        exception: ObjectId,
        catch_location: Option<Location>,
    },
    MethodEntry {
        thread: ThreadId,
        location: Location,
    },
    MethodExit {
        thread: ThreadId,
        location: Location,
    },
    Unknown {
        kind: u8,
    },
}

// Event request modifiers
#[derive(Debug, Clone)]
pub enum EventModifier {
    Count(i32),
    ThreadOnly(ThreadId),
    ClassOnly(ReferenceTypeId),
    ClassMatch(String),
    ClassExclude(String),
    LocationOnly(Location),
    ExceptionOnly {
        ref_type: ReferenceTypeId,
        caught: bool,
        uncaught: bool,
    },
    FieldOnly {
        ref_type: ReferenceTypeId,
        field_id: FieldId,
    },
    Step {
        thread: ThreadId,
        size: i32,
        depth: i32,
    },
    InstanceOnly(ObjectId),
}

/// Parse an event packet from JDWP
pub fn parse_event_packet(data: &[u8]) -> JdwpResult<EventSet> {
    let mut buf = data;

    // Read suspend policy
    let suspend_policy = read_u8(&mut buf)?;

    // Read number of events
    let event_count = read_i32(&mut buf)?;

    let mut events = Vec::with_capacity(event_count as usize);

    for _ in 0..event_count {
        let kind = read_u8(&mut buf)?;
        let request_id = read_i32(&mut buf)?;

        let details = match kind {
            event_kinds::BREAKPOINT => {
                let thread = read_u64(&mut buf)?;
                let location = read_location(&mut buf)?;
                EventKind::Breakpoint { thread, location }
            }
            event_kinds::SINGLE_STEP => {
                let thread = read_u64(&mut buf)?;
                let location = read_location(&mut buf)?;
                EventKind::Step { thread, location }
            }
            event_kinds::VM_START => {
                let thread = read_u64(&mut buf)?;
                EventKind::VMStart { thread }
            }
            event_kinds::VM_DEATH => {
                EventKind::VMDeath
            }
            event_kinds::THREAD_START => {
                let thread = read_u64(&mut buf)?;
                EventKind::ThreadStart { thread }
            }
            event_kinds::THREAD_DEATH => {
                let thread = read_u64(&mut buf)?;
                EventKind::ThreadDeath { thread }
            }
            _ => {
                warn!("Unsupported event kind: {}", kind);
                EventKind::Unknown { kind }
            }
        };

        events.push(Event {
            kind,
            request_id,
            details,
        });
    }

    Ok(EventSet {
        suspend_policy,
        events,
    })
}

/// Read a location from the buffer
fn read_location(buf: &mut &[u8]) -> JdwpResult<Location> {
    let type_tag = read_u8(buf)?;
    let class_id = read_u64(buf)?;
    let method_id = read_u64(buf)?;
    let index = read_u64(buf)?;

    Ok(Location {
        type_tag,
        class_id,
        method_id,
        index,
    })
}
