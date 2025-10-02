// JDWP command implementations
//
// Command Sets:
// 1 = VirtualMachine
// 2 = ReferenceType
// 6 = Method
// 9 = ObjectReference
// 11 = ThreadReference
// 15 = EventRequest
// 16 = StackFrame

// Command set IDs
pub mod command_sets {
    pub const VIRTUAL_MACHINE: u8 = 1;
    pub const REFERENCE_TYPE: u8 = 2;
    pub const CLASS_TYPE: u8 = 3;
    pub const METHOD: u8 = 6;
    pub const OBJECT_REFERENCE: u8 = 9;
    pub const STRING_REFERENCE: u8 = 10;
    pub const THREAD_REFERENCE: u8 = 11;
    pub const THREAD_GROUP_REFERENCE: u8 = 12;
    pub const ARRAY_REFERENCE: u8 = 13;
    pub const EVENT_REQUEST: u8 = 15;
    pub const STACK_FRAME: u8 = 16;
}

// VirtualMachine commands (set 1)
pub mod vm_commands {
    pub const VERSION: u8 = 1;
    pub const CLASSES_BY_SIGNATURE: u8 = 2;
    pub const ALL_CLASSES: u8 = 3;
    pub const ALL_THREADS: u8 = 4;
    pub const TOP_LEVEL_THREAD_GROUPS: u8 = 5;
    pub const DISPOSE: u8 = 6;
    pub const ID_SIZES: u8 = 7;
    pub const SUSPEND: u8 = 8;
    pub const RESUME: u8 = 9;
    pub const EXIT: u8 = 10;
    pub const CREATE_STRING: u8 = 11;
    pub const CAPABILITIES: u8 = 12;
    pub const CLASS_PATHS: u8 = 13;
    pub const DISPOSE_OBJECTS: u8 = 14;
    pub const HOLD_EVENTS: u8 = 15;
    pub const RELEASE_EVENTS: u8 = 16;
}

// ReferenceType commands (set 2)
pub mod reference_type_commands {
    pub const SIGNATURE: u8 = 1;
    pub const CLASS_LOADER: u8 = 2;
    pub const MODIFIERS: u8 = 3;
    pub const FIELDS: u8 = 4;
    pub const METHODS: u8 = 5;
    pub const GET_VALUES: u8 = 6;
    pub const SOURCE_FILE: u8 = 7;
    pub const NESTED_TYPES: u8 = 8;
    pub const STATUS: u8 = 9;
    pub const INTERFACES: u8 = 10;
    pub const CLASS_OBJECT: u8 = 11;
    pub const SOURCE_DEBUG_EXTENSION: u8 = 12;
    pub const SIGNATURE_WITH_GENERIC: u8 = 13;
    pub const FIELDS_WITH_GENERIC: u8 = 14;
    pub const METHODS_WITH_GENERIC: u8 = 15;
}

// Method commands (set 6)
pub mod method_commands {
    pub const LINE_TABLE: u8 = 1;
    pub const VARIABLE_TABLE: u8 = 2;
    pub const BYTECODES: u8 = 3;
    pub const IS_OBSOLETE: u8 = 4;
    pub const VARIABLE_TABLE_WITH_GENERIC: u8 = 5;
}

// ThreadReference commands (set 11)
pub mod thread_commands {
    pub const NAME: u8 = 1;
    pub const SUSPEND: u8 = 2;
    pub const RESUME: u8 = 3;
    pub const STATUS: u8 = 4;
    pub const THREAD_GROUP: u8 = 5;
    pub const FRAMES: u8 = 6;
    pub const FRAME_COUNT: u8 = 7;
    pub const OWNED_MONITORS: u8 = 8;
    pub const CURRENT_CONTENDED_MONITOR: u8 = 9;
    pub const STOP: u8 = 10;
    pub const INTERRUPT: u8 = 11;
    pub const SUSPEND_COUNT: u8 = 12;
}

// EventRequest commands (set 15)
pub mod event_commands {
    pub const SET: u8 = 1;
    pub const CLEAR: u8 = 2;
    pub const CLEAR_ALL_BREAKPOINTS: u8 = 3;
}

// StackFrame commands (set 16)
pub mod stack_frame_commands {
    pub const GET_VALUES: u8 = 1;
    pub const SET_VALUES: u8 = 2;
    pub const THIS_OBJECT: u8 = 3;
    pub const POP_FRAMES: u8 = 4;
}

// Event kinds for EventRequest.Set
pub mod event_kinds {
    pub const SINGLE_STEP: u8 = 1;
    pub const BREAKPOINT: u8 = 2;
    pub const FRAME_POP: u8 = 3;
    pub const EXCEPTION: u8 = 4;
    pub const USER_DEFINED: u8 = 5;
    pub const THREAD_START: u8 = 6;
    pub const THREAD_DEATH: u8 = 7;
    pub const CLASS_PREPARE: u8 = 8;
    pub const CLASS_UNLOAD: u8 = 9;
    pub const CLASS_LOAD: u8 = 10;
    pub const FIELD_ACCESS: u8 = 20;
    pub const FIELD_MODIFICATION: u8 = 21;
    pub const EXCEPTION_CATCH: u8 = 30;
    pub const METHOD_ENTRY: u8 = 40;
    pub const METHOD_EXIT: u8 = 41;
    pub const METHOD_EXIT_WITH_RETURN_VALUE: u8 = 42;
    pub const MONITOR_CONTENDED_ENTER: u8 = 43;
    pub const MONITOR_CONTENDED_ENTERED: u8 = 44;
    pub const MONITOR_WAIT: u8 = 45;
    pub const MONITOR_WAITED: u8 = 46;
    pub const VM_START: u8 = 90;
    pub const VM_DEATH: u8 = 99;
}

// Step sizes
pub mod step_sizes {
    pub const MIN: i32 = 0;
    pub const LINE: i32 = 1;
}

// Step depths
pub mod step_depths {
    pub const INTO: i32 = 0;
    pub const OVER: i32 = 1;
    pub const OUT: i32 = 2;
}
