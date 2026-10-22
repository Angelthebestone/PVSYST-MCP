//! Tool actions that are entirely MCP calculations, not PVsyst ones.
//!
//! They run only with `PVSYST_MCP_APROXIMACIONES=1`; otherwise they are hidden
//! from `tools/list` and rejected with the PVsyst route that replaces them.

use crate::aproximaciones::{self, Aproximacion};
use crate::config::Paths;
use crate::error::Result;
use serde_json::Value;

/// `(tool, action, approximation)`. O(n) lookup over a handful of rows.
static ACCIONES: &[(&str, &str, Aproximacion)] = &[
    (
        "simular",
        "preestimar",
        Aproximacion {
            funcion: "simular preestimar",
            alternativa: "simular accion:ejecutar sobre el proyecto (PVsystCLI run-simulation)",
        },
    ),
    (
        "simular",
        "barrido",
        Aproximacion {
            funcion: "simular barrido",
            alternativa: "simular accion:ejecutar con 'filas' (modo lote -bpf: Plane tilt / Plane azim por fila)",
        },
    ),
    (
        "sitio",
        "geometria_solar",
        Aproximacion {
            funcion: "sitio geometria_solar",
            alternativa: "simular accion:ejecutar con un SFI que pida HSol, AzSol, AngInc, GlobInc y FTransp (input_sfi_file)",
        },
    ),
    (
        "resultados",
        "medido_vs_simulado",
        Aproximacion {
            funcion: "resultados medido_vs_simulado",
            alternativa: "la herramienta de datos medidos de la GUI de PVsyst (Herramientas → Datos medidos)",
        },
    ),
];

/// The approximation behind `tool`/`accion`, if that action is one.
pub(crate) fn de(tool: &str, accion: &str) -> Option<&'static Aproximacion> {
    ACCIONES
        .iter()
        .find(|(t, a, _)| *t == tool && *a == accion)
        .map(|(_, _, ap)| ap)
}

/// Rejects an approximated action unless approximations are enabled.
pub(crate) fn exigir(paths: &Paths, tool: &str, accion: &str) -> Result<()> {
    de(tool, accion).map_or(Ok(()), |ap| aproximaciones::exigir(paths, ap))
}

/// Labels the result of an approximated action.
pub(crate) fn marcar(tool: &str, accion: &str, resultado: &mut Value) {
    if let Some(ap) = de(tool, accion) {
        aproximaciones::marcar(resultado, ap);
    }
}

/// Removes approximated actions from a tool schema's discriminator enum.
pub(crate) fn ocultar(tool: &str, campo: &str, esquema: &mut Value) {
    let Some(valores) = esquema
        .pointer_mut(&format!("/inputSchema/properties/{campo}/enum"))
        .and_then(Value::as_array_mut)
    else {
        return;
    };
    valores.retain(|v| v.as_str().is_none_or(|a| de(tool, a).is_none()));
}

/// Hides the in-house route from every `via` enum when approximations are off.
pub(crate) fn ocultar_via(esquema: &mut Value) {
    let Some(valores) = esquema
        .pointer_mut("/inputSchema/properties/via/enum")
        .and_then(Value::as_array_mut)
    else {
        return;
    };
    valores.retain(|v| v.as_str() != Some(crate::aproximaciones::VIA_APROXIMACION));
}
