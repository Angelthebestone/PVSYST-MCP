//! End-to-end smoke tests: drive the compiled `pvsyst-mcp` server over stdio.
//! Tests needing a real PVsyst install skip (early return) when it is absent.
#![expect(
    clippy::excessive_nesting,
    clippy::indexing_slicing,
    reason = "deuda medida; pendiente del paso 2 de la limpieza"
)]
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

#[path = "mcp_smoke/protocolo_y_recursos.rs"]
mod protocolo_y_recursos;

#[path = "mcp_smoke/proyectos.rs"]
mod proyectos;

#[path = "mcp_smoke/gui.rs"]
mod gui;

#[path = "mcp_smoke/workspace_y_diseno.rs"]
mod workspace_y_diseno;
