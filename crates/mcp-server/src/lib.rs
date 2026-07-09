//! DiJiang MCP Server — exposes workflow state, patterns, and tactics
//! as Model Context Protocol resources and tools over stdio.
//!
//! Protocol: JSON-RPC 2.0 over stdin/stdout
//! Spec: https://modelcontextprotocol.io/

pub mod handlers;
pub mod protocol;

use anyhow::Result;
use handlers::DiJiangMcpHandler;
use protocol::{JsonRpcMessage, JsonRpcResponse};
use std::io::{BufRead, Write};

/// Run the MCP server loop: read JSON-RPC from stdin, dispatch, write to stdout.
pub fn run_server(handler: &DiJiangMcpHandler) -> Result<()> {
    let stdin = std::io::stdin();
    let stdout = std::io::stdout();

    for line in stdin.lock().lines() {
        let line = line?;
        if line.trim().is_empty() {
            continue;
        }

        let request: JsonRpcMessage = match serde_json::from_str(&line) {
            Ok(req) => req,
            Err(e) => {
                let err_resp = JsonRpcResponse::error(None, -32700, format!("Parse error: {}", e));
                let mut out = stdout.lock();
                writeln!(out, "{}", serde_json::to_string(&err_resp)?)?;
                out.flush()?;
                continue;
            }
        };

        let response = handler.handle_request(&request)?;
        if let Some(resp) = response {
            let mut out = stdout.lock();
            writeln!(out, "{}", serde_json::to_string(&resp)?)?;
            out.flush()?;
        }
        // Notifications have no response
    }

    Ok(())
}
