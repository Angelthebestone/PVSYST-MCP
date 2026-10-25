//! Read-only MCP resources: playbooks, coverage map, reference format and the CLI.

use crate::error::{Error, Result};
use serde_json::{json, Value};

const FORMATO: &str = include_str!("../../docs/pvsyst_format.md");
const CLI: &str = include_str!("../../PVSYST_CLI_REFERENCE.md");

/// Catalogue returned by `resources/list`.
pub(crate) fn list() -> Value {
    json!({
        "resources": [
            { "uri": "pvsyst://recetas", "name": "Recetas", "mimeType": "application/json" },
            { "uri": "pvsyst://cobertura", "name": "Cobertura", "mimeType": "application/json" },
            { "uri": "pvsyst://formato", "name": "Formato de archivos PVsyst", "mimeType": "text/markdown" },
            { "uri": "pvsyst://cli", "name": "Referencia PVsystCLI", "mimeType": "text/markdown" },
            { "uri": "pvsyst://gui", "name": "Mapa de la GUI de PVsyst", "mimeType": "application/json" }
        ]
    })
}

/// Contents of one resource. Unknown URIs are an `Err` (the server maps it to -32602).
/// With `aproximaciones` off, the coverage map omits the MCP-approximation actions.
pub(crate) fn read(uri: &str, aproximaciones: bool) -> Result<Value> {
    let (mime, text) = match uri {
        "pvsyst://recetas" => (
            "application/json",
            super::recetas::como_recurso(aproximaciones).to_string(),
        ),
        "pvsyst://cobertura" => ("application/json", cobertura(aproximaciones).to_string()),
        "pvsyst://formato" => ("text/markdown", FORMATO.to_string()),
        "pvsyst://cli" => ("text/markdown", CLI.to_string()),
        "pvsyst://gui" => (
            "application/json",
            crate::gui_catalogo::como_recurso().to_string(),
        ),
        _ => return Err(Error::NotFound(format!("Recurso desconocido: {uri}"))),
    };
    Ok(json!({ "contents": [{ "uri": uri, "mimeType": mime, "text": text }] }))
}

/// Machine-readable coverage map: PVsyst area -> how the MCP covers it.
fn cobertura(aproximaciones: bool) -> Value {
    let mut mapa = mapa_de_cobertura();
    if !aproximaciones {
        quitar_aproximaciones(&mut mapa);
    }
    mapa
}

/// Drops `"<tool> <accion>"` entries that are MCP approximations, and areas
/// left with no tool.
fn quitar_aproximaciones(mapa: &mut Value) {
    if let Some(areas) = mapa.get_mut("areas").and_then(Value::as_array_mut) {
        for area in areas.iter_mut() {
            quitar_aproximaciones_area(area);
        }
        areas.retain(|a| a["tools"].as_array().is_some_and(|t| !t.is_empty()));
    }
    if let Some(sinonimos) = mapa.get_mut("sinonimos").and_then(Value::as_object_mut) {
        quitar_aproximaciones_sinonimos(sinonimos);
    }
}

fn quitar_aproximaciones_area(area: &mut Value) {
    let Some(tools) = area.get_mut("tools").and_then(Value::as_array_mut) else {
        return;
    };
    tools.retain(|t| t.as_str().is_none_or(|e| !es_aproximacion(e)));
}

fn quitar_aproximaciones_sinonimos(sinonimos: &mut serde_json::Map<String, Value>) {
    for (tool, data) in sinonimos.iter_mut() {
        let Some(valores) = data.get_mut("valores").and_then(Value::as_object_mut) else {
            continue;
        };
        valores.retain(|_, can| {
            can.as_str()
                .is_none_or(|acc| !super::tools::es_aproximacion(tool, acc))
        });
    }
}

fn es_aproximacion(entrada: &str) -> bool {
    let mut partes = entrada.split_whitespace();
    let (Some(tool), Some(accion)) = (partes.next(), partes.next()) else {
        return false;
    };
    let accion = accion.rsplit(':').next().unwrap_or(accion);
    super::tools::es_aproximacion(tool, accion)
}

fn mapa_de_cobertura() -> Value {
    json!({
        "sinonimos": super::tools::tabla_alias(),
        "principle": "PVsyst state lives in text files (PVObject), a documented CLI, and a Win32/VCL desktop app. The MCP exposes generic primitives over ALL THREE, so coverage is bounded by the artifact surface, not by a fixed set of buttons.",
        "coverage_layers": [
            "FILES  - read/write any field of any PVObject file (leer/editar)",
            "CLI    - run any PVsystCLI command (pvsyst_cli, simular, sitio, convertir_meteo)",
            "GUI    - drive any window/control of the desktop app via Win32 automation (gui_observar/gui_actuar)"
        ],
        "gui_automation_note": "GUI-only features (3D shading editor, informative I/V & table tools, import wizards, dialogs) are covered by enumerating the real Win32/VCL windows, posting standard window messages, typing into measured 96-DPI fields, choosing combo items and switching tab pages. No PVsyst code is read or disassembled.",
        "areas": [
            { "area": "Easy mode / end-to-end (LLM-friendly)", "tools": ["entorno", "disenar_sistema", "sitio crear", "resultados kpis"], "notes": "Intention-level tools that collapse many steps into one call; tipo_sistema:red|aislado selects the template and variant." },
            { "area": "Workspace", "tools": ["entorno diagnostico", "listar que:archivos", "leer", "escribir_archivo"] },
            { "area": "Projects & variants", "tools": ["listar que:proyectos", "leer", "proyecto crear", "proyecto eliminar", "proyecto renombrar", "proyecto crear_variante", "editar campos", "proyecto dimensionar", "proyecto seguidor", "proyecto limite_inyeccion", "proyecto almacenamiento", "proyecto perdidas", "proyecto bombeo", "proyecto economia"] },
            { "area": "Orientation / array sizing", "tools": ["editar campos", "proyecto dimensionar", "editar"], "notes": "dimensionar with n_mod_serie + n_strings writes them as given; choosing strings from kwp is an MCP approximation (PVSYST_MCP_APROXIMACIONES=1). PVsyst validates the electrical window when simulating (avisos_pvsyst)." },
            { "area": "Component database (all kinds)", "tools": ["listar que:componentes", "leer", "componente crear", "componente borrar", "listar que:modulos"] },
            { "area": "Site & weather", "tools": ["sitio crear", "sitio convertir_meteo", "editar", "leer"] },
            { "area": "Shadings/horizon (.HOR/.SHD/.BLD/.SHO)", "tools": ["leer", "editar", "escribir_archivo"] },
            { "area": "Losses, IAM, thermal, quality", "tools": ["editar campos", "editar"] },
            { "area": "Models/templates (.LOD/.SPL/.SFI/.RVT)", "tools": ["leer", "editar", "escribir_archivo"] },
            { "area": "Simulation (single, batch, synthetic, param override)", "tools": ["simular ejecutar", "pvsyst_cli"] },
            { "area": "Comparison with measured data (variants CMx)", "tools": ["leer", "editar", "gui_actuar"], "notes": "PVsystCLI cannot simulate CMx variants (verified: 'Simulation cannot be run: the orientation is not defined'); run them from the GUI." },
            { "area": "Batch / parametric runs", "tools": ["simular preparar", "simular ejecutar"], "notes": "filas -> batch CSV (-bpf); every row is one PVsyst run." },
            { "area": "Advanced/hidden parameter override", "tools": ["simular preparar", "simular ejecutar"], "notes": "parametros -> Param_Modif.dat (-ipf)." },
            { "area": "Site & weather readers", "tools": ["leer"] },
            { "area": "Results & reports", "tools": ["resultados csv", "leer"] },
            { "area": "Workspace management (create/export/import)", "tools": ["workspace crear", "workspace exportar", "workspace importar", "listar que:archivos"] },
            { "area": "Component & project import/export", "tools": ["componente importar", "componente exportar", "workspace exportar_proyecto"] },
            { "area": "Licensing / logs / version / help", "tools": ["entorno licencia", "entorno exportar_logs", "pvsyst_cli"] },
            { "area": "Trackers (one-axis)", "tools": ["proyecto seguidor"], "notes": "Replaces pvOrient/pvShading from _DEMO_Tracking and preserves NoOrient; refuses 3D scenes and bifacial sheds." },
            { "area": "Injection limit", "tools": ["proyecto limite_inyeccion"], "notes": "PLimGrid, LimGrSeparLoss and TanPhi in pvSystem." },
            { "area": "Battery storage / self-consumption", "tools": ["proyecto almacenamiento", "proyecto limite_inyeccion"], "notes": "pvGridStorage with self-consumption or peak-shaving strategies." },
            { "area": "Variant losses", "tools": ["proyecto perdidas"], "notes": "Soiling, thermal, LID and mismatch on pvMainArray/pvSubArray." },
            { "area": "Pumping", "tools": ["proyecto bombeo"], "notes": "Pump file, static depth and the WaterUse section." },
            { "area": "Economic evaluation (EconomicEval)", "tools": ["proyecto economia", "resultados economia"], "notes": "proyecto economia edits EconomicEval; resultados economia reads investment, yearly costs, payback and LCOE from PVsyst's report." },
            { "area": "P50/P90 evaluation", "tools": ["proyecto p50_p90", "resultados p50_p90"], "notes": "proyecto p50_p90 configures PVsyst's P50-P90 page in its own window; resultados p50_p90 reads the saved report." },
            { "area": "Measured vs simulated monthly energy", "tools": ["resultados medido_vs_simulado"], "notes": "MCP approximation (PVSYST_MCP_APROXIMACIONES=1): compares E_Grid from the PDF with a measured series; PVsyst's own tool is the GUI measured-data analysis." },
            { "area": "Stand-alone systems", "tools": ["disenar_sistema"], "notes": "tipo_sistema:aislado uses _DEMO_StandAlone.VC3 and optional battery counts." },
            { "area": "GUI-only authoring (3D shading editor, informative I/V & tables, wizards, dialogs)", "tools": ["gui_observar", "gui_actuar"], "notes": "Win32/UIA automation: enumerate, read, capture, click, type into fields (escribir), select combo items (seleccionar), switch tab pages (pagina), send real keys and read PVsyst's log messages." },
            { "area": "Any other PVsyst field", "tools": ["leer", "listar que:objetos", "editar"] }
        ]
    })
}
