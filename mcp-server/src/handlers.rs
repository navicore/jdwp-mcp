// MCP request handlers
//
// Handles initialize, list tools, and debug tool execution

use crate::protocol::*;
use crate::session::SessionManager;
use crate::tools;
use serde_json::json;
use tracing::{debug, info, warn};

pub struct RequestHandler {
    session_manager: SessionManager,
}

impl RequestHandler {
    pub fn new() -> Self {
        Self {
            session_manager: SessionManager::new(),
        }
    }

    pub async fn handle_request(&self, request: JsonRpcRequest) -> JsonRpcResponse {
        let result = match request.method.as_str() {
            "initialize" => self.handle_initialize(request.params),
            "tools/list" => self.handle_list_tools(),
            "tools/call" => self.handle_call_tool(request.params).await,
            _ => Err(JsonRpcError {
                code: METHOD_NOT_FOUND,
                message: format!("Method not found: {}", request.method),
                data: None,
            }),
        };

        match result {
            Ok(value) => JsonRpcResponse {
                jsonrpc: "2.0".to_string(),
                id: request.id,
                result: Some(value),
                error: None,
            },
            Err(error) => JsonRpcResponse {
                jsonrpc: "2.0".to_string(),
                id: request.id,
                result: None,
                error: Some(error),
            },
        }
    }

    pub async fn handle_notification(&self, notification: JsonRpcNotification) {
        match notification.method.as_str() {
            "notifications/initialized" => {
                info!("Client initialized");
            }
            "notifications/cancelled" => {
                debug!("Request cancelled");
            }
            _ => {
                warn!("Unknown notification: {}", notification.method);
            }
        }
    }

    fn handle_initialize(&self, params: Option<serde_json::Value>) -> Result<serde_json::Value, JsonRpcError> {
        let _params: InitializeParams = serde_json::from_value(params.unwrap_or(json!({})))
            .map_err(|e| JsonRpcError {
                code: INVALID_PARAMS,
                message: format!("Invalid initialize params: {}", e),
                data: None,
            })?;

        let result = InitializeResult {
            protocol_version: "2024-11-05".to_string(),
            capabilities: ServerCapabilities {
                tools: ToolsCapability {},
            },
            server_info: ServerInfo {
                name: "jdwp-mcp".to_string(),
                version: env!("CARGO_PKG_VERSION").to_string(),
            },
            instructions: Some(
                "JDWP debugging server for Java applications. \
                Start by using debug.attach to connect to a JVM, \
                then use debug.set_breakpoint, debug.get_stack, etc."
                    .to_string(),
            ),
        };

        Ok(serde_json::to_value(result).unwrap())
    }

    fn handle_list_tools(&self) -> Result<serde_json::Value, JsonRpcError> {
        let result = ListToolsResult {
            tools: tools::get_tools(),
        };

        Ok(serde_json::to_value(result).unwrap())
    }

    async fn handle_call_tool(&self, params: Option<serde_json::Value>) -> Result<serde_json::Value, JsonRpcError> {
        let call_params: CallToolParams = serde_json::from_value(params.unwrap_or(json!({})))
            .map_err(|e| JsonRpcError {
                code: INVALID_PARAMS,
                message: format!("Invalid tool call params: {}", e),
                data: None,
            })?;

        // Route to appropriate handler based on tool name
        let result = match call_params.name.as_str() {
            "debug.attach" => self.handle_attach(call_params.arguments).await,
            "debug.set_breakpoint" => self.handle_set_breakpoint(call_params.arguments).await,
            "debug.list_breakpoints" => self.handle_list_breakpoints(call_params.arguments).await,
            "debug.clear_breakpoint" => self.handle_clear_breakpoint(call_params.arguments).await,
            "debug.continue" => self.handle_continue(call_params.arguments).await,
            "debug.step_over" => self.handle_step_over(call_params.arguments).await,
            "debug.step_into" => self.handle_step_into(call_params.arguments).await,
            "debug.step_out" => self.handle_step_out(call_params.arguments).await,
            "debug.get_stack" => self.handle_get_stack(call_params.arguments).await,
            "debug.evaluate" => self.handle_evaluate(call_params.arguments).await,
            "debug.list_threads" => self.handle_list_threads(call_params.arguments).await,
            "debug.pause" => self.handle_pause(call_params.arguments).await,
            "debug.disconnect" => self.handle_disconnect(call_params.arguments).await,
            _ => Err(format!("Unknown tool: {}", call_params.name)),
        };

        match result {
            Ok(content) => {
                let call_result = CallToolResult {
                    content: vec![ContentBlock::Text { text: content }],
                    is_error: None,
                };
                Ok(serde_json::to_value(call_result).unwrap())
            }
            Err(error) => {
                let call_result = CallToolResult {
                    content: vec![ContentBlock::Text { text: error.clone() }],
                    is_error: Some(true),
                };
                Ok(serde_json::to_value(call_result).unwrap())
            }
        }
    }

    // Tool implementations (stubs for now)
    async fn handle_attach(&self, args: serde_json::Value) -> Result<String, String> {
        let host = args.get("host").and_then(|v| v.as_str()).unwrap_or("localhost");
        let port = args.get("port").and_then(|v| v.as_u64()).unwrap_or(5005) as u16;

        match jdwp_client::JdwpConnection::connect(host, port).await {
            Ok(connection) => {
                let session_id = self.session_manager.create_session(connection).await;
                Ok(format!("Connected to JVM at {}:{} (session: {})", host, port, session_id))
            }
            Err(e) => Err(format!("Failed to connect: {}", e)),
        }
    }

    async fn handle_set_breakpoint(&self, args: serde_json::Value) -> Result<String, String> {
        let class_pattern = args.get("class_pattern")
            .and_then(|v| v.as_str())
            .ok_or_else(|| "Missing 'class_pattern' parameter".to_string())?;

        let line = args.get("line")
            .and_then(|v| v.as_i64())
            .ok_or_else(|| "Missing 'line' parameter".to_string())? as i32;

        let method_hint = args.get("method").and_then(|v| v.as_str());

        // Get current session
        let session_guard = self.session_manager.get_current_session().await
            .ok_or_else(|| "No active debug session. Use debug.attach first.".to_string())?;

        let mut session = session_guard.lock().await;

        // Convert class name to JVM signature format
        // e.g., "com.example.MyClass" -> "Lcom/example/MyClass;"
        let signature = if class_pattern.starts_with('L') && class_pattern.ends_with(';') {
            class_pattern.to_string()
        } else {
            format!("L{};", class_pattern.replace('.', "/"))
        };

        // Find the class
        let classes = session.connection.classes_by_signature(&signature).await
            .map_err(|e| format!("Failed to find class: {}", e))?;

        if classes.is_empty() {
            return Err(format!("Class not found: {}", class_pattern));
        }

        let class = &classes[0];

        // Get methods
        let methods = session.connection.get_methods(class.type_id).await
            .map_err(|e| format!("Failed to get methods: {}", e))?;

        // Find the right method (use hint if provided, otherwise find first method containing the line)
        let mut target_method = None;

        for method in &methods {
            if let Some(hint) = method_hint {
                if method.name == hint {
                    target_method = Some(method);
                    break;
                }
            }

            // Check if this method contains the line
            if let Ok(line_table) = session.connection.get_line_table(class.type_id, method.method_id).await {
                if line_table.lines.iter().any(|e| e.line_number == line) {
                    target_method = Some(method);
                    break;
                }
            }
        }

        let method = target_method.ok_or_else(|| {
            format!("No method found containing line {} in class {}", line, class_pattern)
        })?;

        // Get line table and find bytecode index for the line
        let line_table = session.connection.get_line_table(class.type_id, method.method_id).await
            .map_err(|e| format!("Failed to get line table: {}", e))?;

        let line_entry = line_table.lines.iter()
            .find(|e| e.line_number == line)
            .ok_or_else(|| format!("Line {} not found in method {}", line, method.name))?;

        // Set the breakpoint!
        let request_id = session.connection.set_breakpoint(
            class.type_id,
            method.method_id,
            line_entry.line_code_index,
            jdwp_client::SuspendPolicy::All,
        ).await.map_err(|e| format!("Failed to set breakpoint: {}", e))?;

        // Track the breakpoint in session
        let bp_id = format!("bp_{}", request_id);
        session.breakpoints.insert(bp_id.clone(), crate::session::BreakpointInfo {
            id: bp_id.clone(),
            request_id,
            class_pattern: class_pattern.to_string(),
            line: line as u32,
            method: Some(method.name.clone()),
            enabled: true,
            hit_count: 0,
        });

        Ok(format!(
            "✅ Breakpoint set at {}:{}\n   Method: {}\n   Breakpoint ID: {}\n   JDWP Request ID: {}",
            class_pattern, line, method.name, bp_id, request_id
        ))
    }

    async fn handle_list_breakpoints(&self, _args: serde_json::Value) -> Result<String, String> {
        let session_guard = self.session_manager.get_current_session().await
            .ok_or_else(|| "No active debug session".to_string())?;

        let session = session_guard.lock().await;

        if session.breakpoints.is_empty() {
            return Ok("No breakpoints set".to_string());
        }

        let mut output = format!("📍 {} breakpoint(s):\n\n", session.breakpoints.len());

        for (_, bp) in session.breakpoints.iter() {
            output.push_str(&format!(
                "  {} [{}] {}:{}\n",
                if bp.enabled { "✓" } else { "✗" },
                bp.id,
                bp.class_pattern,
                bp.line
            ));
            if let Some(method) = &bp.method {
                output.push_str(&format!("     Method: {}\n", method));
            }
            if bp.hit_count > 0 {
                output.push_str(&format!("     Hits: {}\n", bp.hit_count));
            }
        }

        Ok(output)
    }

    async fn handle_clear_breakpoint(&self, args: serde_json::Value) -> Result<String, String> {
        let bp_id = args.get("breakpoint_id")
            .and_then(|v| v.as_str())
            .ok_or_else(|| "Missing 'breakpoint_id' parameter".to_string())?;

        let session_guard = self.session_manager.get_current_session().await
            .ok_or_else(|| "No active debug session".to_string())?;

        let mut session = session_guard.lock().await;

        // Find the breakpoint
        let bp_info = session.breakpoints.get(bp_id)
            .ok_or_else(|| format!("Breakpoint not found: {}", bp_id))?
            .clone();

        // Clear the breakpoint in the JVM
        session.connection.clear_breakpoint(bp_info.request_id).await
            .map_err(|e| format!("Failed to clear breakpoint: {}", e))?;

        // Remove from session
        session.breakpoints.remove(bp_id);

        Ok(format!(
            "✅ Breakpoint cleared: {} at {}:{}\n   JDWP Request ID: {}",
            bp_id, bp_info.class_pattern, bp_info.line, bp_info.request_id
        ))
    }

    async fn handle_continue(&self, _args: serde_json::Value) -> Result<String, String> {
        let session_guard = self.session_manager.get_current_session().await
            .ok_or_else(|| "No active debug session".to_string())?;

        let mut session = session_guard.lock().await;

        session.connection.resume_all().await
            .map_err(|e| format!("Failed to resume: {}", e))?;

        Ok("▶️  Execution resumed".to_string())
    }

    async fn handle_step_over(&self, _args: serde_json::Value) -> Result<String, String> {
        // TODO: Implement step over
        Ok("Step over not yet implemented".to_string())
    }

    async fn handle_step_into(&self, _args: serde_json::Value) -> Result<String, String> {
        // TODO: Implement step into
        Ok("Step into not yet implemented".to_string())
    }

    async fn handle_step_out(&self, _args: serde_json::Value) -> Result<String, String> {
        // TODO: Implement step out
        Ok("Step out not yet implemented".to_string())
    }

    async fn handle_get_stack(&self, args: serde_json::Value) -> Result<String, String> {
        let session_guard = self.session_manager.get_current_session().await
            .ok_or_else(|| "No active debug session".to_string())?;

        let mut session = session_guard.lock().await;

        let thread_id = args.get("thread_id")
            .and_then(|v| v.as_str())
            .and_then(|s| u64::from_str_radix(s.trim_start_matches("0x"), 16).ok());

        let max_frames = args.get("max_frames")
            .and_then(|v| v.as_i64())
            .unwrap_or(20) as i32;

        let include_variables = args.get("include_variables")
            .and_then(|v| v.as_bool())
            .unwrap_or(true);

        // If no thread specified, get all threads and use the first suspended one
        let target_thread = if let Some(tid) = thread_id {
            tid
        } else {
            let threads = session.connection.get_all_threads().await
                .map_err(|e| format!("Failed to get threads: {}", e))?;

            *threads.first().ok_or_else(|| "No threads found".to_string())?
        };

        // Get frames
        let frames = session.connection.get_frames(target_thread, 0, max_frames).await
            .map_err(|e| format!("Failed to get frames: {}", e))?;

        if frames.is_empty() {
            return Ok(format!("Thread {:x} has no stack frames", target_thread));
        }

        let mut output = format!("🔍 Stack for thread {:x} ({} frames):\n\n", target_thread, frames.len());

        for (idx, frame) in frames.iter().enumerate() {
            output.push_str(&format!("Frame {}:\n", idx));
            output.push_str(&format!("  Location: class={:x}, method={:x}, index={}\n",
                frame.location.class_id, frame.location.method_id, frame.location.index));

            // Try to get method name
            if let Ok(methods) = session.connection.get_methods(frame.location.class_id).await {
                if let Some(method) = methods.iter().find(|m| m.method_id == frame.location.method_id) {
                    output.push_str(&format!("  Method: {}\n", method.name));

                    // Get variables if requested
                    if include_variables {
                        match session.connection.get_variable_table(frame.location.class_id, frame.location.method_id).await {
                            Ok(var_table) => {
                                let current_index = frame.location.index;
                                let active_vars: Vec<_> = var_table.iter()
                                    .filter(|v| current_index >= v.code_index && current_index < v.code_index + v.length as u64)
                                    .collect();

                                if !active_vars.is_empty() {
                                    output.push_str(&format!("  Variables ({}):\n", active_vars.len()));

                                    let slots: Vec<jdwp_client::stackframe::VariableSlot> = active_vars.iter()
                                        .map(|v| jdwp_client::stackframe::VariableSlot {
                                            slot: v.slot as i32,
                                            sig_byte: v.signature.as_bytes()[0],
                                        })
                                        .collect();

                                    if let Ok(values) = session.connection.get_frame_values(target_thread, frame.frame_id, slots).await {
                                        for (var, value) in active_vars.iter().zip(values.iter()) {
                                            output.push_str(&format!("    {} = {}\n", var.name, value.format()));
                                        }
                                    }
                                }
                            }
                            Err(_) => {}
                        }
                    }
                }
            }

            output.push_str("\n");
        }

        Ok(output)
    }

    async fn handle_evaluate(&self, _args: serde_json::Value) -> Result<String, String> {
        // TODO: Implement expression evaluation
        Ok("Expression evaluation not yet implemented".to_string())
    }

    async fn handle_list_threads(&self, _args: serde_json::Value) -> Result<String, String> {
        let session_guard = self.session_manager.get_current_session().await
            .ok_or_else(|| "No active debug session".to_string())?;

        let mut session = session_guard.lock().await;

        let threads = session.connection.get_all_threads().await
            .map_err(|e| format!("Failed to get threads: {}", e))?;

        let mut output = format!("🧵 {} thread(s):\n\n", threads.len());

        for (idx, thread_id) in threads.iter().enumerate() {
            output.push_str(&format!("  Thread {} (ID: 0x{:x})\n", idx + 1, thread_id));

            // Try to get frame count
            match session.connection.get_frames(*thread_id, 0, 1).await {
                Ok(frames) if !frames.is_empty() => {
                    output.push_str("     Status: Has frames (possibly suspended)\n");
                }
                Ok(_) => {
                    output.push_str("     Status: Running (no frames)\n");
                }
                Err(_) => {
                    output.push_str("     Status: Cannot inspect\n");
                }
            }
        }

        Ok(output)
    }

    async fn handle_pause(&self, _args: serde_json::Value) -> Result<String, String> {
        let session_guard = self.session_manager.get_current_session().await
            .ok_or_else(|| "No active debug session".to_string())?;

        let mut session = session_guard.lock().await;

        session.connection.suspend_all().await
            .map_err(|e| format!("Failed to suspend: {}", e))?;

        Ok("⏸️  Execution paused (all threads suspended)".to_string())
    }

    async fn handle_disconnect(&self, _args: serde_json::Value) -> Result<String, String> {
        let current_session_id = self.session_manager.get_current_session_id().await;

        if let Some(session_id) = current_session_id {
            // Remove the session (this will also clear current session)
            self.session_manager.remove_session(&session_id).await;
            Ok(format!("✅ Disconnected from debug session: {}", session_id))
        } else {
            Err("No active debug session to disconnect".to_string())
        }
    }
}
