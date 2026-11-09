//! Regresiones de comportamiento sobre la superficie pública del MCP (stdio):
//! entradas inválidas, resolución dentro del `workspace` pedido y límites de las
//! salidas. Las que necesitan red o la CLI de PVsyst van marcadas `#[ignore]`.
// Test code: failing loudly on unexpected state is the point.
#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

mod cliente_mcp;

#[path = "mcp_regresiones/argumentos.rs"]
mod argumentos;

#[path = "mcp_regresiones/diseno.rs"]
mod diseno;

#[path = "mcp_regresiones/workspace.rs"]
mod workspace;

#[path = "mcp_regresiones/archivos.rs"]
mod archivos;

#[path = "mcp_regresiones/ruta_pvsyst.rs"]
mod ruta_pvsyst;

#[path = "mcp_regresiones/proyectos.rs"]
mod proyectos;

#[path = "mcp_regresiones/plantillas.rs"]
mod plantillas;

#[path = "mcp_regresiones/exportar.rs"]
mod exportar;

#[path = "mcp_regresiones/escritura.rs"]
mod escritura;
