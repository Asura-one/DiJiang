//! JSON-RPC 2.0 protocol types for MCP over stdio.
//!
//! Minimal wire format: MCP uses standard JSON-RPC 2.0 with method names
//! like `resources/list`, `tools/call`, etc.

use serde::{Deserialize, Serialize};
use serde_json::Value;

/// A JSON-RPC 2.0 request or notification received from the client.
#[derive(Debug, Clone, Deserialize)]
pub struct JsonRpcRequest {
    pub jsonrpc: String,
    #[serde(default)]
    pub id: Option<Value>,
    pub method: String,
    #[serde(default)]
    pub params: Option<Value>,
}

/// A JSON-RPC 2.0 response sent to the client.
#[derive(Debug, Clone, Serialize)]
pub struct JsonRpcResponse {
    pub jsonrpc: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub result: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<JsonRpcError>,
}

impl JsonRpcResponse {
    pub fn success(id: Option<Value>, result: Value) -> Self {
        Self {
            jsonrpc: "2.0".to_string(),
            id,
            result: Some(result),
            error: None,
        }
    }

    pub fn error(id: Option<Value>, code: i64, message: String) -> Self {
        Self {
            jsonrpc: "2.0".to_string(),
            id,
            result: None,
            error: Some(JsonRpcError { code, message }),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JsonRpcError {
    pub code: i64,
    pub message: String,
}

/// Union type: either a request (with id) or a notification (without id).
#[derive(Debug, Clone, Deserialize)]
#[serde(untagged)]
pub enum JsonRpcMessage {
    Request(JsonRpcRequest),
    Notification {
        jsonrpc: String,
        method: String,
        #[serde(default)]
        params: Option<Value>,
    },
}

impl JsonRpcMessage {
    pub fn method(&self) -> &str {
        match self {
            JsonRpcMessage::Request(req) => &req.method,
            JsonRpcMessage::Notification { method, .. } => method,
        }
    }

    pub fn id(&self) -> Option<Value> {
        match self {
            JsonRpcMessage::Request(req) => req.id.clone(),
            JsonRpcMessage::Notification { .. } => None,
        }
    }

    pub fn params(&self) -> Option<&Value> {
        match self {
            JsonRpcMessage::Request(req) => req.params.as_ref(),
            JsonRpcMessage::Notification { params, .. } => params.as_ref(),
        }
    }

    pub fn is_notification(&self) -> bool {
        matches!(self, JsonRpcMessage::Notification { .. })
    }
}

// ─── MCP-specific types ─────────────────────────────────────

/// Tool descriptor returned by `tools/list`.
#[derive(Debug, Clone, Serialize)]
pub struct McpTool {
    pub name: String,
    pub description: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub input_schema: Option<Value>,
}

/// Resource descriptor returned by `resources/list`.
#[derive(Debug, Clone, Serialize)]
pub struct McpResource {
    pub uri: String,
    pub name: String,
    pub description: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mime_type: Option<String>,
}

/// Resource content returned by `resources/read`.
#[derive(Debug, Clone, Serialize)]
pub struct McpResourceContent {
    pub uri: String,
    pub mime_type: String,
    pub text: String,
}
