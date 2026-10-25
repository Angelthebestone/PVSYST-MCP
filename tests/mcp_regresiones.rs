//! Regresiones de comportamiento sobre la superficie pública del MCP (stdio):
//! entradas inválidas, resolución dentro del `workspace` pedido y límites de las
//! salidas. Las que necesitan red o la CLI de PVsyst van marcadas `#[ignore]`.
#![expect(
    clippy::indexing_slicing,
    reason = "deuda medida; pendiente del paso 2 de la limpieza"
)]
// Test code: failing loudly on unexpected state is the point.
#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

mod cliente_mcp;

#[path = "mcp_regresiones/argumentos.rs"]
mod argumentos;

#[path = "mcp_regresiones/diseno.rs"]
mod diseno;

#[path = "mcp_regresiones/workspace.rs"]
mod workspace;
