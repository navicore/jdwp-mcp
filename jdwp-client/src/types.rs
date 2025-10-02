// JDWP type definitions
//
// Common types used across the JDWP protocol

use serde::{Deserialize, Serialize};

// Object IDs are 8 bytes in JDWP
pub type ObjectId = u64;
pub type ThreadId = ObjectId;
pub type ThreadGroupId = ObjectId;
pub type StringId = ObjectId;
pub type ClassLoaderId = ObjectId;
pub type ClassObjectId = ObjectId;
pub type ArrayId = ObjectId;

pub type ReferenceTypeId = u64;
pub type ClassId = ReferenceTypeId;
pub type InterfaceId = ReferenceTypeId;
pub type ArrayTypeId = ReferenceTypeId;

pub type MethodId = u64;
pub type FieldId = u64;
pub type FrameId = u64;

// Location identifies a code position
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Location {
    pub type_tag: u8, // 1=class, 2=interface, 3=array
    pub class_id: ReferenceTypeId,
    pub method_id: MethodId,
    pub index: u64, // bytecode index (PC)
}

// Thread status values
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[repr(u32)]
pub enum ThreadStatus {
    Zombie = 0,
    Running = 1,
    Sleeping = 2,
    Monitor = 3,
    Wait = 4,
}

// Suspend status values
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[repr(u32)]
pub enum SuspendStatus {
    Running = 0,
    Suspended = 1,
}

// Type tags for values
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum TypeTag {
    Array = 91,      // '['
    Byte = 66,       // 'B'
    Char = 67,       // 'C'
    Object = 76,     // 'L'
    Float = 70,      // 'F'
    Double = 68,     // 'D'
    Int = 73,        // 'I'
    Long = 74,       // 'J'
    Short = 83,      // 'S'
    Void = 86,       // 'V'
    Boolean = 90,    // 'Z'
    String = 115,    // 's'
    Thread = 116,    // 't'
    ThreadGroup = 103, // 'g'
    ClassLoader = 108, // 'l'
    ClassObject = 99,  // 'c'
}

// Tagged value
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Value {
    pub tag: u8,
    pub data: ValueData,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum ValueData {
    Byte(i8),
    Char(u16),
    Float(f32),
    Double(f64),
    Int(i32),
    Long(i64),
    Short(i16),
    Boolean(bool),
    Object(ObjectId),
    Void,
}

impl Value {
    /// Format value for display
    pub fn format(&self) -> String {
        match &self.data {
            ValueData::Byte(v) => format!("(byte) {}", v),
            ValueData::Char(v) => format!("(char) '{}'", char::from_u32(*v as u32).unwrap_or('?')),
            ValueData::Float(v) => format!("(float) {}", v),
            ValueData::Double(v) => format!("(double) {}", v),
            ValueData::Int(v) => format!("(int) {}", v),
            ValueData::Long(v) => format!("(long) {}", v),
            ValueData::Short(v) => format!("(short) {}", v),
            ValueData::Boolean(v) => format!("(boolean) {}", v),
            ValueData::Object(id) => {
                if *id == 0 {
                    "(object) null".to_string()
                } else {
                    format!("(object) @{:x}", id)
                }
            }
            ValueData::Void => "(void)".to_string(),
        }
    }
}

// Variable information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Variable {
    pub code_index: u64,
    pub name: String,
    pub signature: String,
    pub length: u32,
    pub slot: u32,
}

// Stack frame information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FrameInfo {
    pub frame_id: FrameId,
    pub location: Location,
}
