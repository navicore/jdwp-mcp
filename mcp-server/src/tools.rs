// Debug tools schema definitions
//
// MCP tools for JDWP debugging operations

use crate::protocol::Tool;
use serde_json::json;

pub fn get_tools() -> Vec<Tool> {
    vec![
        Tool {
            name: "debug.attach".to_string(),
            description: "Connect to a JVM via JDWP protocol".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "host": {
                        "type": "string",
                        "description": "JVM host (e.g., 'localhost')",
                        "default": "localhost"
                    },
                    "port": {
                        "type": "integer",
                        "description": "JDWP port (e.g., 5005)",
                        "default": 5005
                    },
                    "timeout_ms": {
                        "type": "integer",
                        "description": "Connection timeout in milliseconds",
                        "default": 5000
                    }
                },
                "required": ["host", "port"]
            }),
        },
        Tool {
            name: "debug.set_breakpoint".to_string(),
            description: "Set a breakpoint at a specific location".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "class_pattern": {
                        "type": "string",
                        "description": "Class name pattern (e.g., 'com.example.MyClass')"
                    },
                    "line": {
                        "type": "integer",
                        "description": "Line number"
                    },
                    "method": {
                        "type": "string",
                        "description": "Method name (optional, helps resolve ambiguity)"
                    }
                },
                "required": ["class_pattern", "line"]
            }),
        },
        Tool {
            name: "debug.list_breakpoints".to_string(),
            description: "List all active breakpoints".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {}
            }),
        },
        Tool {
            name: "debug.clear_breakpoint".to_string(),
            description: "Clear a specific breakpoint".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "breakpoint_id": {
                        "type": "string",
                        "description": "Breakpoint ID from list_breakpoints"
                    }
                },
                "required": ["breakpoint_id"]
            }),
        },
        Tool {
            name: "debug.continue".to_string(),
            description: "Resume execution (all threads or specific thread)".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "thread_id": {
                        "type": "string",
                        "description": "Thread ID to resume (optional, resumes all if omitted)"
                    }
                }
            }),
        },
        Tool {
            name: "debug.step_over".to_string(),
            description: "Step over current line".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "thread_id": {
                        "type": "string",
                        "description": "Thread ID to step"
                    }
                },
                "required": ["thread_id"]
            }),
        },
        Tool {
            name: "debug.step_into".to_string(),
            description: "Step into method call".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "thread_id": {
                        "type": "string",
                        "description": "Thread ID to step"
                    }
                },
                "required": ["thread_id"]
            }),
        },
        Tool {
            name: "debug.step_out".to_string(),
            description: "Step out of current method".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "thread_id": {
                        "type": "string",
                        "description": "Thread ID to step"
                    }
                },
                "required": ["thread_id"]
            }),
        },
        Tool {
            name: "debug.get_stack".to_string(),
            description: "Get stack frames with summarized variables".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "thread_id": {
                        "type": "string",
                        "description": "Thread ID"
                    },
                    "max_frames": {
                        "type": "integer",
                        "description": "Maximum number of frames to return",
                        "default": 20
                    },
                    "include_variables": {
                        "type": "boolean",
                        "description": "Include local variables in frames",
                        "default": true
                    },
                    "max_variable_depth": {
                        "type": "integer",
                        "description": "How deep to traverse object graphs (1-3)",
                        "default": 2
                    }
                },
                "required": ["thread_id"]
            }),
        },
        Tool {
            name: "debug.evaluate".to_string(),
            description: "Evaluate expression in frame context".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "thread_id": {
                        "type": "string",
                        "description": "Thread ID"
                    },
                    "frame_index": {
                        "type": "integer",
                        "description": "Stack frame index (0 = current frame)",
                        "default": 0
                    },
                    "expression": {
                        "type": "string",
                        "description": "Java expression to evaluate"
                    },
                    "max_result_length": {
                        "type": "integer",
                        "description": "Maximum length of result string",
                        "default": 500
                    }
                },
                "required": ["thread_id", "expression"]
            }),
        },
        Tool {
            name: "debug.list_threads".to_string(),
            description: "List all threads with status".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {}
            }),
        },
        Tool {
            name: "debug.pause".to_string(),
            description: "Pause execution (all threads or specific thread)".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "thread_id": {
                        "type": "string",
                        "description": "Thread ID to pause (optional, pauses all if omitted)"
                    }
                }
            }),
        },
        Tool {
            name: "debug.disconnect".to_string(),
            description: "Disconnect from JVM debug session".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {}
            }),
        },
    ]
}
