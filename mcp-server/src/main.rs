// JDWP MCP Server - Java debugging via Model Context Protocol
//
// Provides LLM-friendly debugging tools for JVM applications via JDWP

use anyhow::Result;
use serde_json::Value;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tracing::{debug, error, info};

mod handlers;
mod protocol;
mod session;
mod tools;

use handlers::RequestHandler;
use protocol::*;

#[tokio::main]
async fn main() -> Result<()> {
    // Tracing to stderr only - stdout is reserved for JSON-RPC protocol
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::from_default_env()
                .add_directive("jdwp_mcp=info".parse().unwrap()),
        )
        .with_writer(std::io::stderr)
        .init();

    info!("Starting JDWP MCP Server...");

    let handler = RequestHandler::new();

    // Stdio transport - no network, no files
    let stdin = tokio::io::stdin();
    let stdout = tokio::io::stdout();
    let mut reader = BufReader::new(stdin);
    let mut stdout = stdout;

    info!("JDWP MCP server ready, waiting for requests...");

    // Single-threaded message loop
    loop {
        let mut line = String::new();
        match reader.read_line(&mut line).await {
            Ok(0) => {
                info!("Client disconnected");
                break;
            }
            Ok(_) => {
                let line = line.trim();
                if line.is_empty() {
                    continue;
                }

                debug!("Received: {}", line);

                // Parse as generic Value first
                match serde_json::from_str::<Value>(line) {
                    Ok(value) => {
                        // Discriminate by id field
                        if value.get("id").is_some() {
                            // It's a request
                            match serde_json::from_value::<JsonRpcRequest>(value) {
                                Ok(request) => {
                                    let response = handler.handle_request(request).await;
                                    let response_str = serde_json::to_string(&response)?;
                                    debug!("Sending: {}", response_str);
                                    stdout.write_all(response_str.as_bytes()).await?;
                                    stdout.write_all(b"\n").await?;
                                    stdout.flush().await?;
                                }
                                Err(e) => {
                                    error!("Invalid request: {}", e);
                                    let error_response = JsonRpcResponse {
                                        jsonrpc: "2.0".to_string(),
                                        id: serde_json::Value::Null,
                                        result: None,
                                        error: Some(JsonRpcError {
                                            code: INVALID_REQUEST,
                                            message: "Invalid request".to_string(),
                                            data: None,
                                        }),
                                    };
                                    let response_str = serde_json::to_string(&error_response)?;
                                    stdout.write_all(response_str.as_bytes()).await?;
                                    stdout.write_all(b"\n").await?;
                                    stdout.flush().await?;
                                }
                            }
                        } else {
                            // It's a notification
                            match serde_json::from_value::<JsonRpcNotification>(value) {
                                Ok(notification) => {
                                    handler.handle_notification(notification).await;
                                }
                                Err(e) => {
                                    error!("Invalid notification: {}", e);
                                }
                            }
                        }
                    }
                    Err(e) => {
                        error!("Parse error: {}", e);
                        let error_response = JsonRpcResponse {
                            jsonrpc: "2.0".to_string(),
                            id: serde_json::Value::Null,
                            result: None,
                            error: Some(JsonRpcError {
                                code: PARSE_ERROR,
                                message: "Parse error".to_string(),
                                data: None,
                            }),
                        };
                        let response_str = serde_json::to_string(&error_response)?;
                        stdout.write_all(response_str.as_bytes()).await?;
                        stdout.write_all(b"\n").await?;
                        stdout.flush().await?;
                    }
                }
            }
            Err(e) => {
                error!("Read error: {}", e);
                break;
            }
        }
    }

    info!("JDWP MCP server shutting down");
    Ok(())
}
