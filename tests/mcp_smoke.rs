//! End-to-end smoke test: drives the compiled `pvsyst-mcp` server over stdio.
// Test code: failing loudly on unexpected state is the point.
#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::needless_pass_by_value,
    unreachable_pub
)]

mod cliente_mcp;

#[path = "mcp_smoke/workspace_y_diseno.rs"]
mod workspace_y_diseno;
