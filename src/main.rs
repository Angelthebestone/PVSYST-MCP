//! `pvsyst-mcp` executable: MCP over stdio. stdout carries protocol traffic only;
//! diagnostics go to stderr, filtered by `PVSYST_MCP_LOG` (default `info`).

use anyhow::{Context, Result};
use pvsyst_mcp::{serve, Paths, Server};
use std::io;
use tracing_subscriber::EnvFilter;

fn main() -> Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::try_from_env("PVSYST_MCP_LOG").unwrap_or_else(|_| EnvFilter::new("info")),
        )
        .with_writer(io::stderr)
        .with_ansi(false)
        .init();

    let paths = Paths::resolve();
    tracing::info!(
        version = env!("CARGO_PKG_VERSION"),
        workspace = %paths.workspace.display(),
        "pvsyst-mcp starting"
    );
    let server = Server::new(paths);
    match serve(&server, io::BufReader::new(io::stdin()), io::stdout()) {
        // The client closed our stdout: a normal way to end the session.
        Err(e) if e.kind() == io::ErrorKind::BrokenPipe => Ok(()),
        other => other.context("MCP stdio session failed"),
    }
}
