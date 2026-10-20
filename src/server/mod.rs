//! Minimal, dependency-light Model Context Protocol server over stdio.
//!
//! MCP stdio transport frames each JSON-RPC 2.0 message on its own line
//! (newline-delimited JSON), so we read a line, dispatch, and write one line.

use crate::config::Paths;
mod resources;
mod tools;

use serde_json::{json, Value};

pub(crate) const PROTOCOL_VERSION: &str = "2025-03-26";

/// Supported MCP protocol versions in ascending order (latest is [`PROTOCOL_VERSION`]).
pub(crate) const SUPPORTED_PROTOCOL_VERSIONS: &[&str] = &["2024-11-05", "2025-03-26"];

/// A Model Context Protocol server bound to one [`Paths`] configuration.
pub struct Server {
    paths: Paths,
}

impl Server {
    /// Creates a server that operates the PVsyst installation described by `paths`.
    #[must_use]
    pub fn new(paths: Paths) -> Server {
        Server { paths }
    }

    /// Handles one JSON-RPC line. Returns the response line to write back, or
    /// `None` for notifications, which never get one.
    ///
    /// ```
    /// use pvsyst_mcp::{Paths, Server};
    ///
    /// let server = Server::new(Paths::resolve());
    /// let reply = server
    ///     .handle_line(r#"{"jsonrpc":"2.0","id":7,"method":"ping"}"#)
    ///     .expect("requests get a response");
    /// assert!(reply.contains(r#""id":7"#));
    /// assert!(server
    ///     .handle_line(r#"{"jsonrpc":"2.0","method":"notifications/initialized"}"#)
    ///     .is_none());
    /// ```
    pub fn handle_line(&self, line: &str) -> Option<String> {
        let Ok(msg) = serde_json::from_str::<Value>(line) else {
            tracing::warn!("received a line that is not valid JSON");
            return Some(err_response(&Value::Null, -32700, "Parse error"));
        };

        let id = msg.get("id");
        let (method, params) = match validate_jsonrpc_shape(&msg) {
            Ok(pair) => pair,
            Err((code, err_msg)) => {
                return Some(err_response(id.unwrap_or(&Value::Null), code, err_msg))
            }
        };

        tracing::debug!(method, "request");

        // Centralized notification rule: messages without an `id` NEVER receive a response.
        let Some(req_id) = id else {
            self.process_notification(method, params);
            return None;
        };

        self.dispatch_request(method, params, req_id)
    }

    fn process_notification(&self, method: &str, params: &Value) {
        if method == "tools/call" {
            let _ = self.call_tool(params);
        }
    }

    fn dispatch_request(&self, method: &str, params: &Value, id: &Value) -> Option<String> {
        let reply = |result: Value| Some(ok_response(id, &result));
        match method {
            // Notifications (even if sent with an id mistakenly) get no response.
            "notifications/initialized" | "initialized" | "notifications/cancelled" => None,
            "initialize" => reply(initialize_result(params)),
            "logging/setLevel" | "completion/complete" | "ping" => reply(json!({})),
            "tools/list" => reply(json!({ "tools": tools::list(&self.paths) })),
            "tools/call" => reply(self.call_tool(params)),
            "resources/list" => reply(resources::list()),
            "resources/read" => Some(self.read_resource(params, id)),
            "resources/templates/list" => reply(json!({ "resourceTemplates": [] })),
            "prompts/list" => reply(json!({ "prompts": [] })),
            _ => Some(err_response(id, -32601, "Method not found")),
        }
    }

    fn read_resource(&self, params: &Value, id: &Value) -> String {
        let uri = params.get("uri").and_then(Value::as_str).unwrap_or("");
        match resources::read(uri) {
            Ok(value) => ok_response(id, &value),
            Err(e) => {
                tracing::warn!(uri, kind = e.kind(), error = %e, "resource read failed");
                err_response(id, -32602, &e.to_string())
            }
        }
    }

    fn call_tool(&self, params: &Value) -> Value {
        let name = params.get("name").and_then(Value::as_str).unwrap_or("");
        // Missing arguments behave like `{}`: `Value::get` on Null yields None.
        let args = params.get("arguments").unwrap_or(&Value::Null);
        match tools::call(&self.paths, name, args) {
            Ok(value) => resultado_de_herramienta(value),
            Err(e) => {
                tracing::warn!(tool = name, kind = e.kind(), error = %e, "tool call failed");
                json!({
                    "content": [{ "type": "text", "text": e.to_string() }],
                    "isError": true,
                })
            }
        }
    }
}

/// Validates that `msg` conforms to the JSON-RPC 2.0 request envelope.
fn validate_jsonrpc_shape(msg: &Value) -> std::result::Result<(&str, &Value), (i64, &'static str)> {
    if !msg.is_object() {
        return Err((-32600, "Invalid Request"));
    }
    if msg.get("jsonrpc").and_then(Value::as_str) != Some("2.0") {
        return Err((-32600, "Invalid Request: jsonrpc must be '2.0'"));
    }
    let Some(method) = msg.get("method").and_then(Value::as_str) else {
        return Err((-32600, "Invalid Request: method must be a string"));
    };
    if let Some(params) = msg.get("params") {
        if !params.is_object() {
            return Err((-32602, "Invalid params: must be an object"));
        }
    }
    let params = msg.get("params").unwrap_or(&Value::Null);
    Ok((method, params))
}

/// Negotiate protocol version: echo client's requested version if supported;
/// otherwise fall back to the server's version.
fn initialize_result(params: &Value) -> Value {
    let client_version = params.get("protocolVersion").and_then(Value::as_str);
    let negotiated = match client_version {
        Some(v) if SUPPORTED_PROTOCOL_VERSIONS.contains(&v) => v,
        _ => PROTOCOL_VERSION,
    };
    json!({
        "protocolVersion": negotiated,
        "capabilities": { "tools": {}, "resources": {} },
        "serverInfo": {
            "name": "pvsyst-mcp",
            "version": env!("CARGO_PKG_VERSION"),
        },
        "instructions": "Operate PVsyst 8.1.7 through its data files only. \
    Use the domain tools (entorno, leer, editar, listar, proyecto, componente, \
    sitio, simular, resultados, workspace). Recipes, coverage, format and CLI \
    reference are in the pvsyst:// resources. Expert tools need PVSYST_MCP_EXPERTO=1."
    })
}

fn ok_response(id: &Value, result: &Value) -> String {
    json!({ "jsonrpc": "2.0", "id": id, "result": result }).to_string()
}

fn err_response(id: &Value, code: i64, message: &str) -> String {
    json!({
        "jsonrpc": "2.0",
        "id": id,
        "error": { "code": code, "message": message }
    })
    .to_string()
}

/// Builds the MCP tool result. A `_imagen_png_base64` key moves out of the JSON
/// text into its own image content item.
fn resultado_de_herramienta(mut value: Value) -> Value {
    let imagen = value
        .as_object_mut()
        .and_then(|o| o.remove("_imagen_png_base64"));
    let mut content = vec![json!({ "type": "text", "text": value.to_string() })];
    if let Some(data) = imagen {
        content.push(json!({ "type": "image", "data": data, "mimeType": "image/png" }));
    }
    json!({ "content": content, "isError": false })
}

#[cfg(test)]
#[path = "../../tests/unit/server/tests.rs"]
mod tests;
