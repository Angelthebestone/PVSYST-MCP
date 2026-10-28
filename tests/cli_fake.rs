//! End-to-end tests of the CLI runner against `fake_pvsystcli`, which replays
//! recorded PVsystCLI stdout. No PVsyst licence or trial execution is used.

// Test code: failing loudly on unexpected state is the point.
#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::too_many_lines,
    clippy::needless_pass_by_value,
    unreachable_pub
)]

mod cliente_mcp;

#[path = "cli_fake/ejecucion.rs"]
mod ejecucion;

#[path = "cli_fake/encadenado.rs"]
mod encadenado;

#[path = "cli_fake/resultados.rs"]
mod resultados;

#[path = "cli_fake/entorno.rs"]
mod entorno;
