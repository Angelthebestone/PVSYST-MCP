//! Minimal, dependency-light Model Context Protocol server over stdio.
//!
//! MCP stdio transport frames each JSON-RPC 2.0 message on its own line
//! (newline-delimited JSON), so we read a line, dispatch, and write one line.
//! Tool calls run on one worker thread so the main loop can keep answering
//! pings and cancellations while a long simulation is in flight.

mod bucle;
pub(crate) mod cancelacion;
mod recetas;
mod resources;
mod respuesta;
pub(crate) mod salida;
mod tools;

use crate::config::Paths;
use crate::progreso::SeguimientoTarea;
use respuesta::{err_response, error_de_herramienta, ok_response, resultado_de_herramienta};
use serde_json::{json, Value};
use std::sync::OnceLock;

pub(crate) use bucle::{Entrada, TrabajoHerramienta};
use cancelacion::Cancelaciones;
pub(crate) use respuesta::FormatoResultado;

pub(crate) const PROTOCOL_VERSION: &str = "2025-06-18";

/// Supported MCP protocol versions in ascending order (latest is [`PROTOCOL_VERSION`]).
pub(crate) const SUPPORTED_PROTOCOL_VERSIONS: &[&str] = &["2024-11-05", "2025-03-26", "2025-06-18"];

/// A Model Context Protocol server bound to one [`Paths`] configuration.
pub struct Server {
    paths: Paths,
    version: OnceLock<&'static str>,
    pub(crate) cancelaciones: Cancelaciones,
}

impl Server {
    /// Creates a server that operates the PVsyst installation described by `paths`.
    #[must_use]
    pub fn new(paths: Paths) -> Server {
        Server {
            paths,
            version: OnceLock::new(),
            cancelaciones: Cancelaciones::default(),
        }
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
            let _ = self.ejecutar_herramienta(params, &SeguimientoTarea::inerte());
        }
    }

    fn dispatch_request(&self, method: &str, params: &Value, id: &Value) -> Option<String> {
        let reply = |result: Value| Some(ok_response(id, &result));
        match method {
            // Notifications (even if sent with an id mistakenly) get no response.
            "notifications/initialized" | "initialized" | "notifications/cancelled" => None,
            "initialize" => reply(self.initialize_result(params)),
            "logging/setLevel" | "completion/complete" | "ping" => reply(json!({})),
            "tools/list" => reply(json!({ "tools": tools::list(&self.paths) })),
            "tools/call" => reply(self.ejecutar_herramienta(params, &SeguimientoTarea::inerte())),
            "resources/list" => reply(resources::list()),
            "resources/read" => Some(self.read_resource(params, id)),
            "resources/templates/list" => reply(json!({ "resourceTemplates": [] })),
            "prompts/list" => reply(recetas::listar_prompts(self.paths.aproximaciones)),
            "prompts/get" => match recetas::obtener_prompt(params, self.paths.aproximaciones) {
                Ok(value) => reply(value),
                Err(e) => Some(err_response(id, -32602, &e.to_string())),
            },
            _ => Some(err_response(id, -32601, "Method not found")),
        }
    }

    fn read_resource(&self, params: &Value, id: &Value) -> String {
        let uri = params.get("uri").and_then(Value::as_str).unwrap_or("");
        match resources::read(uri, self.paths.aproximaciones) {
            Ok(value) => ok_response(id, &value),
            Err(e) => {
                tracing::warn!(uri, kind = e.kind(), error = %e, "resource read failed");
                err_response(id, -32602, &e.to_string())
            }
        }
    }

    fn ejecutar_herramienta(&self, params: &Value, seguimiento: &SeguimientoTarea<'_>) -> Value {
        let name = params.get("name").and_then(Value::as_str).unwrap_or("");
        // Missing arguments behave like `{}`: `Value::get` on Null yields None.
        let args = params.get("arguments").unwrap_or(&Value::Null);
        let formato = self.formato_resultado();
        let solicitud = tools::Solicitud {
            paths: &self.paths,
            name,
            args,
            seguimiento,
        };
        match tools::call_seguido(&solicitud) {
            Ok(value) => resultado_de_herramienta(value, formato),
            Err(e) => {
                tracing::warn!(tool = name, kind = e.kind(), error = %e, "tool call failed");
                error_de_herramienta(&e, formato)
            }
        }
    }

    fn formato_resultado(&self) -> FormatoResultado {
        match self.version.get() {
            Some(version) if *version == PROTOCOL_VERSION => FormatoResultado::ConEstructura,
            _ => FormatoResultado::SoloTexto,
        }
    }

    /// Negotiate protocol version: echo client's requested version if supported;
    /// otherwise fall back to the server's version. The first handshake wins.
    fn initialize_result(&self, params: &Value) -> Value {
        let requested = params
            .get("protocolVersion")
            .and_then(Value::as_str)
            .and_then(|v| {
                SUPPORTED_PROTOCOL_VERSIONS
                    .iter()
                    .copied()
                    .find(|supported| *supported == v)
            })
            .unwrap_or(PROTOCOL_VERSION);
        let negotiated = *self.version.get_or_init(|| requested);
        json!({
            "protocolVersion": negotiated,
            "capabilities": { "tools": {}, "resources": {}, "prompts": {} },
            "serverInfo": {
                "name": "pvsyst-mcp",
                "version": env!("CARGO_PKG_VERSION"),
            },
            "instructions": "Operate PVsyst 8.1.7 through its data files, its CLI and its GUI. \
        Every PVsyst-native tool is available by default: the domain tools (entorno, leer, \
        editar, listar, proyecto, componente, sitio, simular, resultados, workspace), the raw \
        PVsystCLI passthrough (pvsyst_cli) and the GUI automation tools (gui_observar, \
        gui_actuar). Recipes, coverage, format and CLI reference are in the pvsyst:// \
        resources. Every figure comes from PVsyst; the MCP's own approximations (labelled \
        origen:aproximacion_mcp) need PVSYST_MCP_APROXIMACIONES=1."
        })
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

#[cfg(test)]
#[path = "../../tests/unit/server/tests.rs"]
mod tests;
