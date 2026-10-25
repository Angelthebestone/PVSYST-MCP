#![warn(missing_docs)]

//! PVsyst MCP server.
//!
//! Operates a PVsyst 8.1.7 installation by reading and writing ONLY its data
//! files (.PRJ, .VCi, .PAN, .OND, .MET, .SIT, .CSV, ...). It never links,
//! loads, decompiles or otherwise touches PVsyst.exe beyond the documented
//! black-box invocation of PVsystCLI.exe for batch simulations.
//!
//! The public surface is deliberately tiny: build a [`Server`] from [`Paths`]
//! and drive it with [`serve`] (newline-delimited JSON-RPC over any reader and
//! writer) or one line at a time with [`Server::handle_line`].

#[cfg(not(windows))]
compile_error!("pvsyst-mcp solo funciona en Windows: PVsyst 8.1.7 es una aplicación Windows.");

// Layout: one folder per PVsyst business area (projects, components, sites,
// simulation, workspace, easy_mode). `format` is the pure core every area
// builds on; `adapters` (CLI, GUI) and `server` (MCP stdio) are the edges.
// Dependencies point inward: `format` imports nothing from the crate.

mod adapters;
#[expect(
    clippy::excessive_nesting,
    clippy::indexing_slicing,
    clippy::too_many_arguments,
    reason = "deuda medida; pendiente del paso 2 de la limpieza"
)]
mod admin;
#[expect(
    clippy::indexing_slicing,
    reason = "deuda medida; pendiente del paso 2 de la limpieza"
)]
mod almacenamiento;
#[expect(
    clippy::too_many_arguments,
    reason = "deuda medida; pendiente del paso 2 de la limpieza"
)]
mod aproximaciones;
#[expect(
    clippy::indexing_slicing,
    clippy::too_many_arguments,
    reason = "deuda medida; pendiente del paso 2 de la limpieza"
)]
mod args;
mod bombeo;
mod calendario;
#[expect(
    clippy::excessive_nesting,
    clippy::indexing_slicing,
    clippy::too_many_arguments,
    reason = "deuda medida; pendiente del paso 2 de la limpieza"
)]
mod components;
mod config;
#[expect(
    clippy::indexing_slicing,
    reason = "deuda medida; pendiente del paso 2 de la limpieza"
)]
mod divisas;
#[expect(
    clippy::excessive_nesting,
    clippy::indexing_slicing,
    clippy::too_many_arguments,
    reason = "deuda medida; pendiente del paso 2 de la limpieza"
)]
mod easy_mode;
mod error;
mod escena_3d;
#[expect(
    clippy::excessive_nesting,
    reason = "deuda medida; pendiente del paso 2 de la limpieza"
)]
mod evaluacion_economica;
#[expect(
    clippy::excessive_nesting,
    clippy::indexing_slicing,
    clippy::too_many_arguments,
    reason = "deuda medida; pendiente del paso 2 de la limpieza"
)]
mod format;
mod gui_catalogo;
#[expect(
    clippy::excessive_nesting,
    clippy::indexing_slicing,
    clippy::too_many_arguments,
    reason = "deuda medida; pendiente del paso 2 de la limpieza"
)]
mod history;
#[expect(
    clippy::excessive_nesting,
    clippy::indexing_slicing,
    reason = "deuda medida; pendiente del paso 2 de la limpieza"
)]
mod horizonte_archivo;
#[expect(
    clippy::excessive_nesting,
    reason = "deuda medida; pendiente del paso 2 de la limpieza"
)]
mod inyeccion;
#[expect(
    clippy::excessive_nesting,
    clippy::indexing_slicing,
    reason = "deuda medida; pendiente del paso 2 de la limpieza"
)]
mod medicion;
#[expect(
    clippy::excessive_nesting,
    clippy::indexing_slicing,
    reason = "deuda medida; pendiente del paso 2 de la limpieza"
)]
mod meteo_mef;
#[expect(
    clippy::excessive_nesting,
    clippy::indexing_slicing,
    reason = "deuda medida; pendiente del paso 2 de la limpieza"
)]
mod objects;
#[expect(
    clippy::excessive_nesting,
    reason = "deuda medida; pendiente del paso 2 de la limpieza"
)]
mod perdidas;
#[expect(
    clippy::too_many_arguments,
    reason = "deuda medida; pendiente del paso 2 de la limpieza"
)]
mod plantilla_variables;
#[expect(
    clippy::excessive_nesting,
    clippy::indexing_slicing,
    clippy::too_many_arguments,
    reason = "deuda medida; pendiente del paso 2 de la limpieza"
)]
mod prefs;
mod progreso;
#[expect(
    clippy::excessive_nesting,
    clippy::indexing_slicing,
    clippy::too_many_arguments,
    reason = "deuda medida; pendiente del paso 2 de la limpieza"
)]
mod projects;
mod seguidores;
#[expect(
    clippy::excessive_nesting,
    clippy::indexing_slicing,
    reason = "deuda medida; pendiente del paso 2 de la limpieza"
)]
mod serie_horaria;
mod server;
#[expect(
    clippy::excessive_nesting,
    clippy::indexing_slicing,
    clippy::too_many_arguments,
    reason = "deuda medida; pendiente del paso 2 de la limpieza"
)]
mod simulation;
#[expect(
    clippy::excessive_nesting,
    clippy::indexing_slicing,
    clippy::too_many_arguments,
    reason = "deuda medida; pendiente del paso 2 de la limpieza"
)]
mod sites;
mod state;
mod vista_agente;
#[expect(
    clippy::excessive_nesting,
    clippy::indexing_slicing,
    reason = "deuda medida; pendiente del paso 2 de la limpieza"
)]
mod workspace;

use serde_json::Value;
use server::salida::Salida;
use server::{Entrada, TrabajoHerramienta};
use std::io::{self, BufRead, Write};
use std::sync::mpsc;

pub use config::Paths;
pub use server::Server;

/// Makes the process Per-Monitor-DPI-aware (v2). Call once at start-up, before
/// the server touches any PVsyst window: GUI coordinates (clicks, captures,
/// layout positions measured at 96 DPI) are only right in physical pixels.
pub fn declarar_dpi_por_monitor() {
    adapters::gui::ffi::declarar_dpi_por_monitor();
}

/// Reads `PVSYST_MCP_ESCRITORIO` and, for `aislado`, creates a private Win32
/// desktop where PVsyst runs and the GUI tools act without touching the user's
/// focus, cursor or keyboard. Call once at start-up, before [`serve`].
///
/// # Errors
///
/// Fails when the variable has an unknown value (so a typo never drives the
/// user's desktop) or when the desktop cannot be created.
pub fn preparar_escritorio() -> io::Result<()> {
    adapters::gui::escritorio::preparar()
        .map(|_| ())
        .map_err(|e| io::Error::new(io::ErrorKind::InvalidInput, e.to_string()))
}

/// Serves MCP over newline-delimited JSON: reads one request per line from
/// `input` and writes one response line to `output` (notifications get none).
///
/// Tool calls are queued to one worker thread, so pings and cancellations are
/// answered while a long simulation runs. Returns when `input` reaches EOF and
/// the worker has drained its queue.
///
/// # Errors
///
/// Returns the first I/O error from reading `input` or writing `output`.
#[expect(
    clippy::excessive_nesting,
    reason = "deuda medida; pendiente del paso 2 de la limpieza"
)]
pub fn serve<R: BufRead, W: Write + Send>(server: &Server, input: R, output: W) -> io::Result<()> {
    let salida = Salida::nueva(output);
    std::thread::scope(|scope| -> io::Result<()> {
        let (emisor, receptor) = mpsc::channel();
        let salida_compartida = &salida;
        let trabajador = scope.spawn(move || -> io::Result<()> {
            adapters::gui::escritorio::adjuntar_hilo_actual()
                .map_err(|e| io::Error::other(e.to_string()))?;
            server.atender_trabajos(receptor, salida_compartida);
            Ok(())
        });

        for line in input.lines() {
            let line = line?;
            let trimmed = line.trim();
            if trimmed.is_empty() {
                continue;
            }
            match server.clasificar(trimmed) {
                Entrada::Herramienta { id, params } => {
                    let clave = id
                        .as_ref()
                        .map_or_else(|| "null".to_string(), Value::to_string);
                    let bandera = server.cancelaciones.registrar(clave.clone());
                    emisor
                        .send(TrabajoHerramienta {
                            id,
                            params,
                            bandera,
                            clave,
                        })
                        .map_err(|_| io::Error::other("hilo trabajador detenido"))?;
                }
                Entrada::Cancelacion { id_peticion } => {
                    server.cancelaciones.cancelar(&id_peticion.to_string());
                }
                Entrada::Inmediata(Some(respuesta)) => salida.escribir_linea(&respuesta)?,
                Entrada::Inmediata(None) => {}
            }
        }

        drop(emisor);
        trabajador
            .join()
            .map_err(|_| io::Error::other("hilo trabajador terminó con pánico"))??;
        Ok(())
    })
}
