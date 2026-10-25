//! Tool actions that are entirely MCP calculations, not PVsyst ones.
//!
//! They run only with `PVSYST_MCP_APROXIMACIONES=1`; otherwise they are hidden
//! from `tools/list` and rejected with the PVsyst route that replaces them.

use super::declaracion::Herramienta;
use super::validacion::canonico_de_alias;
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

/// Removes approximated actions, and the aliases that resolve to them, from a tool
/// schema's discriminator enum.
pub(crate) fn ocultar(h: &Herramienta, campo: &str, esquema: &mut Value) {
    let Some(valores) = esquema
        .pointer_mut(&format!("/inputSchema/properties/{campo}/enum"))
        .and_then(Value::as_array_mut)
    else {
        return;
    };
    valores.retain(|v| {
        v.as_str().is_none_or(|a| {
            let canonico = canonico_de_alias(h.sinonimos.valores, a).unwrap_or(a);
            de(h.nombre, canonico).is_none()
        })
    });
}

/// Hides the in-house route from every `via` enum, and from its description,
/// when approximations are off.
pub(crate) fn ocultar_via(esquema: &mut Value) {
    if let Some(valores) = esquema
        .pointer_mut("/inputSchema/properties/via/enum")
        .and_then(Value::as_array_mut)
    {
        valores.retain(|v| v.as_str() != Some(crate::aproximaciones::VIA_APROXIMACION));
    }
    if let Some(descripcion) = esquema.pointer_mut("/inputSchema/properties/via/description") {
        *descripcion = Value::from(DESCRIPCION_VIA_SOLO_PVSYST);
    }
}

const DESCRIPCION_VIA_SOLO_PVSYST: &str = "pvsyst (defecto): la ruta de PVsyst.";

/// Tool descriptions without the in-house actions, for `tools/list` when they are off.
static DESCRIPCIONES_SIN_APROXIMACIONES: &[(&str, &str)] = &[
    (
        "simular",
        "Simulacion: ejecutar (con filas/parametros opcionales), preparar entrada y estado. / EN: Simulation: run (optional rows/parameters), prepare input and status.",
    ),
    (
        "sitio",
        "Sitios y meteo: crear (coordenadas o CSV), convertir, horizonte y TMY de PVGIS. / EN: Sites and weather: create (coordinates or CSV), convert, PVGIS horizon and TMY.",
    ),
    (
        "resultados",
        "Resultados: KPIs, CSV, perdidas, mensual, comparacion, P50/P90, carbono, economia y entregable. / EN: Results: KPIs, CSV, losses, monthly, comparison, P50/P90, carbon, economics and deliverable.",
    ),
];

/// Replaces the tool description by its version without in-house actions, if it has one.
pub(crate) fn ocultar_descripcion(tool: &str, esquema: &mut Value) {
    let Some((_, texto)) = DESCRIPCIONES_SIN_APROXIMACIONES
        .iter()
        .find(|(nombre, _)| *nombre == tool)
    else {
        return;
    };
    if let Some(descripcion) = esquema.get_mut("description") {
        *descripcion = Value::from(*texto);
    }
}

/// Properties read only by approximated actions; removed from the schema when they are off.
static PROPIEDADES_APROXIMADAS: &[(&str, &[&str])] =
    &[("resultados", &["medido_kwh", "medido_csv", "umbral_pct"])];

/// Removes the properties that only approximated actions read, from `properties` and `required`.
pub(crate) fn ocultar_propiedades(tool: &str, esquema: &mut Value) {
    let Some((_, propiedades)) = PROPIEDADES_APROXIMADAS
        .iter()
        .find(|(nombre, _)| *nombre == tool)
    else {
        return;
    };
    for propiedad in propiedades.iter() {
        quitar_propiedad(esquema, propiedad);
    }
}

fn quitar_propiedad(esquema: &mut Value, clave: &str) {
    if let Some(props) = esquema
        .pointer_mut("/inputSchema/properties")
        .and_then(Value::as_object_mut)
    {
        props.remove(clave);
    }
    if let Some(req) = esquema
        .pointer_mut("/inputSchema/required")
        .and_then(Value::as_array_mut)
    {
        req.retain(|v| v.as_str() != Some(clave));
    }
}
