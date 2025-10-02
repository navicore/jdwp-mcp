// JDWP event handling
//
// Events are sent from the JVM to notify about breakpoints, steps, etc.

use crate::types::*;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Event {
    pub kind: EventKind,
    pub request_id: i32,
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
