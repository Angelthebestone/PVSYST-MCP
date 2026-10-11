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
//!
//! ```
//! use pvsyst_mcp::{serve, Paths, Server};
//!
//! let server = Server::new(Paths::resolve());
//! let mut out = Vec::new();
//! let ping = br#"{"jsonrpc":"2.0","id":1,"method":"ping"}"#;
//! serve(&server, ping.as_slice(), &mut out)?;
//! assert!(String::from_utf8(out)?.contains(r#""result":{}"#));
//! # Ok::<(), Box<dyn std::error::Error>>(())
//! ```

// Layout: one folder per PVsyst business area (projects, components, sites,
// simulation, workspace, easy_mode). `format` is the pure core every area
// builds on; `adapters` (CLI, GUI) and `server` (MCP stdio) are the edges.
// Dependencies point inward: `format` imports nothing from the crate.
#![warn(missing_docs)]

mod adapters;
mod admin;
mod args;
mod components;
mod config;
mod easy_mode;
mod error;
mod format;
mod history;
mod objects;
mod prefs;
mod projects;
mod server;
mod simulation;
mod sites;
mod state;
mod workspace;

use std::io::{self, BufRead, Write};

pub use config::Paths;
pub use server::Server;

/// Serves MCP over newline-delimited JSON: reads one request per line from
/// `input` and writes one response line to `output` (notifications get none).
///
/// Returns when `input` reaches end of file.
///
/// # Errors
///
/// Returns the first I/O error from reading `input` or writing `output`.
pub fn serve<R: BufRead, W: Write>(server: &Server, input: R, mut output: W) -> io::Result<()> {
    for line in input.lines() {
        let line = line?;
        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }
        if let Some(response) = server.handle_line(trimmed) {
            writeln!(output, "{response}")?;
            output.flush()?;
        }
    }
    Ok(())
}
