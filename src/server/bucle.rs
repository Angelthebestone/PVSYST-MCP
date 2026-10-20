//! Line classification and the single tool worker thread.

use super::respuesta::ok_response;
use super::salida::Salida;
use super::Server;
use crate::progreso::{Informe, SeguimientoTarea};
use serde_json::{json, Value};
use std::io::Write;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::Receiver;
use std::sync::Arc;

const TOTAL_PROGRESO: f64 = 100.0;

/// One queued `tools/call`, with the flag that can cancel it.
pub(crate) struct TrabajoHerramienta {
    pub(crate) id: Option<Value>,
    pub(crate) params: Value,
    pub(crate) bandera: Arc<AtomicBool>,
    pub(crate) clave: String,
}

/// Classification of one input line for the serving loop.
pub(crate) enum Entrada {
    Herramienta { id: Option<Value>, params: Value },
    Cancelacion { id_peticion: Value },
    Inmediata(Option<String>),
}

impl Server {
    /// Classifies one line: tool calls go to the worker, cancellations mark their
    /// flag, and everything else is answered immediately.
    pub(crate) fn clasificar(&self, linea: &str) -> Entrada {
        let Ok(msg) = serde_json::from_str::<Value>(linea) else {
            return Entrada::Inmediata(self.handle_line(linea));
        };
        if super::validate_jsonrpc_shape(&msg).is_err() {
            return Entrada::Inmediata(self.handle_line(linea));
        }
        let method = msg.get("method").and_then(Value::as_str).unwrap_or("");
        let params = msg.get("params").cloned().unwrap_or(Value::Null);
        match method {
            "tools/call" => Entrada::Herramienta {
                id: msg.get("id").cloned(),
                params,
            },
            "notifications/cancelled" => Entrada::Cancelacion {
                id_peticion: params.get("requestId").cloned().unwrap_or(Value::Null),
            },
            _ => Entrada::Inmediata(self.handle_line(linea)),
        }
    }

    /// Drains queued tool calls in order until the sender is dropped.
    pub(crate) fn atender_trabajos<W: Write + Send>(
        &self,
        trabajos: Receiver<TrabajoHerramienta>,
        salida: &Salida<W>,
    ) {
        for trabajo in trabajos {
            self.atender_trabajo(&trabajo, salida);
        }
    }

    fn atender_trabajo<W: Write + Send>(&self, trabajo: &TrabajoHerramienta, salida: &Salida<W>) {
        let cancelado = || trabajo.bandera.load(Ordering::Relaxed);
        if cancelado() {
            self.cancelaciones.quitar(&trabajo.clave, &trabajo.bandera);
            return;
        }
        let informar = self.informe_de_progreso(&trabajo.params, salida);
        let seguimiento = SeguimientoTarea::nuevo(Arc::clone(&trabajo.bandera), informar);
        let respuesta = self.ejecutar_herramienta(&trabajo.params, &seguimiento);
        if let Some(id) = trabajo.id.as_ref().filter(|_| !cancelado()) {
            if let Err(e) = salida.escribir_linea(&ok_response(id, &respuesta)) {
                tracing::warn!(error = %e, "no se pudo escribir la respuesta de la tool");
            }
        }
        self.cancelaciones.quitar(&trabajo.clave, &trabajo.bandera);
    }

    fn informe_de_progreso<'a, W: Write + Send>(
        &self,
        params: &Value,
        salida: &'a Salida<W>,
    ) -> Option<Informe<'a>> {
        let token = params
            .get("_meta")?
            .get("progressToken")
            .filter(|v| v.is_string() || v.is_number())?
            .clone();
        Some(Box::new(move |progreso, mensaje| {
            let linea = json!({
                "jsonrpc": "2.0",
                "method": "notifications/progress",
                "params": {
                    "progressToken": token,
                    "progress": progreso,
                    "total": TOTAL_PROGRESO,
                    "message": mensaje,
                },
            })
            .to_string();
            if let Err(e) = salida.escribir_linea(&linea) {
                tracing::warn!(error = %e, "no se pudo escribir el progreso");
            }
        }))
    }
}
