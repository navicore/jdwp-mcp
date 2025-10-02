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

    async fn handle_set_breakpoint(&self, _args: serde_json::Value) -> Result<String, String> {
        // TODO: Implement breakpoint setting
        Ok("Breakpoint setting not yet implemented".to_string())
    }

    async fn handle_list_breakpoints(&self, _args: serde_json::Value) -> Result<String, String> {
        // TODO: Implement breakpoint listing
        Ok("Breakpoint listing not yet implemented".to_string())
    }

    async fn handle_clear_breakpoint(&self, _args: serde_json::Value) -> Result<String, String> {
        // TODO: Implement breakpoint clearing
        Ok("Breakpoint clearing not yet implemented".to_string())
    }

    async fn handle_continue(&self, _args: serde_json::Value) -> Result<String, String> {
        // TODO: Implement continue
        Ok("Continue not yet implemented".to_string())
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

    async fn handle_get_stack(&self, _args: serde_json::Value) -> Result<String, String> {
        // TODO: Implement stack inspection
        Ok("Stack inspection not yet implemented".to_string())
    }

    async fn handle_evaluate(&self, _args: serde_json::Value) -> Result<String, String> {
        // TODO: Implement expression evaluation
        Ok("Expression evaluation not yet implemented".to_string())
    }

    async fn handle_list_threads(&self, _args: serde_json::Value) -> Result<String, String> {
        // TODO: Implement thread listing
        Ok("Thread listing not yet implemented".to_string())
    }

    async fn handle_pause(&self, _args: serde_json::Value) -> Result<String, String> {
        // TODO: Implement pause
        Ok("Pause not yet implemented".to_string())
    }

    async fn handle_disconnect(&self, _args: serde_json::Value) -> Result<String, String> {
        // TODO: Implement disconnect
        Ok("Disconnect not yet implemented".to_string())
    }
}
