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
mod admin;
mod almacenamiento;
mod args;
mod bombeo;
mod calendario;
mod components;
mod config;
mod easy_mode;
mod error;
mod evaluacion_economica;
mod format;
mod history;
mod inyeccion;
mod objects;
mod perdidas;
mod prefs;
mod progreso;
mod projects;
mod seguidores;
mod server;
mod simulation;
mod sites;
mod state;
mod workspace;

use serde_json::Value;
use server::salida::Salida;
use server::{Entrada, TrabajoHerramienta};
use std::io::{self, BufRead, Write};
use std::sync::mpsc;

pub use config::Paths;
pub use server::Server;

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
pub fn serve<R: BufRead, W: Write + Send>(server: &Server, input: R, output: W) -> io::Result<()> {
    let salida = Salida::nueva(output);
    std::thread::scope(|scope| -> io::Result<()> {
        let (emisor, receptor) = mpsc::channel();
        let salida_compartida = &salida;
        let trabajador = scope.spawn(move || server.atender_trabajos(receptor, salida_compartida));

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
            .map_err(|_| io::Error::other("hilo trabajador terminó con pánico"))?;
        Ok(())
    })
}
