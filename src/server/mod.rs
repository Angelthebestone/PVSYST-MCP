//! Minimal, dependency-light Model Context Protocol server over stdio.
//!
//! MCP stdio transport frames each JSON-RPC 2.0 message on its own line
//! (newline-delimited JSON), so we read a line, dispatch, and write one line.
//! Tool calls run on one worker thread so the main loop can keep answering
//! pings and cancellations while a long simulation is in flight.

#[expect(
    clippy::excessive_nesting,
    reason = "deuda medida; pendiente del paso 2 de la limpieza"
)]
mod bucle;
#[expect(
    clippy::excessive_nesting,
    reason = "deuda medida; pendiente del paso 2 de la limpieza"
)]
pub(crate) mod cancelacion;
#[expect(
    clippy::indexing_slicing,
    reason = "deuda medida; pendiente del paso 2 de la limpieza"
)]
mod recetas;
mod resources;
#[expect(
    clippy::indexing_slicing,
    reason = "deuda medida; pendiente del paso 2 de la limpieza"
)]
mod respuesta;
pub(crate) mod salida;
mod tools;

use crate::config::Paths;
use crate::error::Error;
use crate::idioma::{Idioma, Texto};
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
        if method != "tools/call" {
            return;
        }
        // A notification gets no answer, so a failed call would otherwise vanish.
        let resultado = self.ejecutar_herramienta(params, &SeguimientoTarea::inerte());
        if resultado.get("isError") == Some(&Value::Bool(true)) {
            tracing::warn!(method, "tools/call recibido como notificación falló");
        }
    }

    #[expect(
        clippy::too_many_arguments,
        reason = "deuda medida; pendiente del paso 2 de la limpieza"
    )]
    fn dispatch_request(&self, method: &str, params: &Value, id: &Value) -> Option<String> {
        let reply = |result: Value| Some(ok_response(id, &result));
        match method {
            // Notifications (even if sent with an id mistakenly) get no response.
            "notifications/initialized" | "initialized" | "notifications/cancelled" => None,
            "initialize" => reply(self.initialize_result(params)),
            "logging/setLevel" | "completion/complete" | "ping" => reply(json!({})),
            "tools/list" => reply(json!({ "tools": tools::list(&self.paths) })),
            "tools/call" => reply(self.ejecutar_herramienta(params, &SeguimientoTarea::inerte())),
            "resources/list" => reply(resources::list(self.paths.idioma)),
            "resources/read" => Some(self.read_resource(params, id)),
            "resources/templates/list" => reply(json!({ "resourceTemplates": [] })),
            "prompts/list" => reply(recetas::listar_prompts(
                self.paths.aproximaciones,
                self.paths.idioma,
            )),
            "prompts/get" => {
                match recetas::obtener_prompt(params, self.paths.aproximaciones, self.paths.idioma)
                {
                    Ok(value) => reply(value),
                    Err(e) => Some(err_response(id, -32602, &e.to_string())),
                }
            }
            _ => Some(err_response(id, -32601, "Method not found")),
        }
    }

    fn read_resource(&self, params: &Value, id: &Value) -> String {
        let uri = params.get("uri").and_then(Value::as_str).unwrap_or("");
        match resources::read(uri, self.paths.aproximaciones, self.paths.idioma) {
            Ok(value) => ok_response(id, &value),
            Err(e) => {
                tracing::warn!(uri, kind = e.kind(), error = %e, "resource read failed");
                err_response(id, -32602, &e.to_string())
            }
        }
    }

    fn ejecutar_herramienta(&self, params: &Value, seguimiento: &SeguimientoTarea<'_>) -> Value {
        let Some(name) = params.get("name").and_then(Value::as_str) else {
            let e = Error::Invalid(FALTA_NAME.texto_con(self.paths.idioma, &[]));
            return error_de_herramienta(&e, self.formato_resultado(), None);
        };
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
                let vista = tools::vista_de_error(&solicitud, &e);
                error_de_herramienta(&e, formato, vista.as_ref())
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
            "instructions": instrucciones(self.paths.aproximaciones, self.paths.idioma)
        })
    }
}

const INSTRUCCIONES_BASE: Texto = Texto {
    es: "Opera PVsyst 8.1 (verificado en 8.1.7 y 8.1.8) a través de sus ficheros de datos, su CLI y su GUI. Cada cifra proviene de PVsyst. Hechos que ahorran llamadas: (1) los componentes se direccionan por NOMBRE (componente:\"Nombre\"), nunca por ruta; solo importar toma una ruta (origen). (2) teclas solo acepta nombres de tecla (RETURN, TAB, DOWN); para escribir texto usa escribir con un control (campo:{clase,x,y} con las claves de vista). En tu escritorio se niega a enviar teclas cuando no puede darles el foco; en el escritorio aislado informa foco:false y las envía igual. (3) esperar espera una VENTANA (clase/titulo obligatorios), no es una pausa; reintenta una vez (1,5 s) antes de informar del timeout. (4) campo casa con la etiqueta VISIBLE y rellenar con las claves del esquema; las claves de vista como combobox_100_138 son coordenadas de cliente que puedes pasar a campo/seleccionar. (5) resultados que:resumen devuelve KPIs + mensual + pérdidas principales en una llamada: úsalo en vez de tres lecturas del mismo PDF. (6) dry_run solo lo aceptan las acciones que lo implementan (pvsyst_cli, entorno licencia/exportar_logs, simular ejecutar, sitio crear/convertir_meteo/horizonte_pvgis, proyecto p50_p90, disenar_sistema, workspace restaurar, gui_actuar escena/escena_importar/rellenar); las demás lo rechazan por su nombre en vez de ejecutar. simular, disenar_sistema y proyecto p50_p90 informan costo_estimado (ejecuciones de prueba que gastaría la llamada). sitio crear, sitio convertir_meteo, sitio meteo_pvgis y simular consumen ejecuciones TRIAL (meteo_pvgis es gratis cuando sus coordenadas están en caché). (7) las listas se truncan a 13 elementos por defecto: pasa detalle:normal (50) o detalle:completo (todos). (8) el argumento workspace es un override por llamada: las rutas puras de PVsyst lo RECHAZAN con un error que lo nombra. Si lo rechazan, no repitas sin él: eso apunta al workspace configurado. (9) GUI: lánzala una vez (instancia única). El diálogo de prueba TDEMOForm es modal y bloquea las llamadas de GUI: ciérralo con gui_actuar accion:cerrar ventana:TDEMOForm. Las funciones variante.* y el editor 3D necesitan un proyecto cargado (TProjectForm): gui_actuar accion:funcion funcion:proyecto.cargar, escribe el nombre en el buscador y elige la fila 0 con accion:fila. (10) accion:fila selecciona fila solo en los controles cuya geometría está medida (árbol de proyectos, selector de módulos, tabla de objetos 3D); en los demás se niega y los nombra. (11) los errores listan las alternativas válidas cuando hay una enumeración; una clave listada en ignorados no la leyó la tool, así que no se aplicó. (12) toda respuesta, errores incluidos, lleva vista (estado, resumen, campos, siguiente): cada paso de siguiente es una llamada que puede enviarse tal cual. Recetas y referencias: recursos pvsyst://.",
    en: "Operate PVsyst 8.1 (measured on 8.1.7 and 8.1.8) through its data files, its CLI and its GUI. Every figure comes from PVsyst. Facts that save round-trips: (1) components are addressed by NAME (componente:\"Name\"), never by path; only importar takes a path (origen). (2) teclas takes key names only (RETURN, TAB, DOWN); to type text use escribir with a control (campo:{clase,x,y} with the vista keys). On your own desktop it refuses to send keys when it cannot give them the focus; on the isolated desktop it reports foco:false and sends them anyway. (3) esperar waits for a WINDOW (clase/titulo required); it is not a sleep. It re-checks once (1.5 s) before reporting a timeout. (4) campo matches the VISIBLE label and rellenar the schema keys; vista keys such as combobox_100_138 are client coordinates you can pass to campo/seleccionar. (5) resultados que:resumen returns KPIs + monthly + main losses in one call: prefer it to three reads of the same PDF. (6) dry_run is only accepted by the actions that implement it (pvsyst_cli, entorno licencia/exportar_logs, simular ejecutar, sitio crear/convertir_meteo/horizonte_pvgis, proyecto p50_p90, disenar_sistema, workspace restaurar, gui_actuar escena/escena_importar/rellenar); the other actions refuse it by name instead of running. simular, disenar_sistema and proyecto p50_p90 report costo_estimado (trial executions the call would spend). sitio crear, sitio convertir_meteo, sitio meteo_pvgis and simular spend TRIAL executions (meteo_pvgis is free when its coordinates are cached). (7) lists are truncated to 13 items by default: pass detalle:normal (50) or detalle:completo (all). (8) the workspace argument is a per-call override: pure-PVsyst routes REJECT it with a named error. If rejected, do not just retry without it: that targets the configured workspace. (9) GUI: launch once (single instance). The TDEMOForm trial dialog is modal and blocks GUI calls: close it with gui_actuar accion:cerrar ventana:TDEMOForm. variante.* functions and the 3D editor need a project loaded (TProjectForm): gui_actuar accion:funcion funcion:proyecto.cargar, type the name in the search box and select row 0 with accion:fila. (10) accion:fila selects a row only in the controls whose geometry was measured (project tree, module chooser, 3D object table); elsewhere it refuses and names them. (11) errors list the valid alternatives when there is an enumeration; a key reported in ignorados was not read by the tool, so it was not applied. (12) every answer, errors included, carries vista (estado, resumen, campos, siguiente): each siguiente step is a call you can send as written. Recipes and references: pvsyst:// resources.",
};
const INSTRUCCIONES_APROXIMACIONES: Texto = Texto {
    es: " Las aproximaciones propias del MCP (marcadas con origen:aproximacion_mcp) se activan con PVSYST_MCP_APROXIMACIONES=1.",
    en: " The MCP's own approximations (labelled origen:aproximacion_mcp) are enabled by PVSYST_MCP_APROXIMACIONES=1.",
};

const FALTA_NAME: Texto = Texto {
    es: "Falta 'name' (string) en tools/call",
    en: "Missing 'name' (string) in tools/call",
};

/// Instructions sent at initialize; the approximation note appears only when its gate is on.
fn instrucciones(aproximaciones: bool, idioma: Idioma) -> String {
    if aproximaciones {
        format!(
            "{}{}",
            INSTRUCCIONES_BASE.en_idioma(idioma),
            INSTRUCCIONES_APROXIMACIONES.en_idioma(idioma)
        )
    } else {
        INSTRUCCIONES_BASE.en_idioma(idioma).to_owned()
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
