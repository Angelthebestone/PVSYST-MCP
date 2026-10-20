//! Read-only MCP resources: playbooks, coverage map and reference dorkspace or the CLI.

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
            { "uri": "pvsyst://cli", "name": "Referencia PVsystCLI", "mimeType": "text/markdown" }
        ]
    })
}

/// Contents of one resource. Unknown URIs are an `Err` (the server maps it to -32602).
pub(crate) fn read(uri: &str) -> Result<Value> {
    let (mime, text) = match uri {
        "pvsyst://recetas" => (
            "application/json",
            super::recetas::como_recurso().to_string(),
        ),
        "pvsyst://cobertura" => ("application/json", cobertura().to_string()),
        "pvsyst://formato" => ("text/markdown", FORMATO.to_string()),
        "pvsyst://cli" => ("text/markdown", CLI.to_string()),
        _ => return Err(Error::Invalid(format!("Recurso desconocido: {uri}"))),
    };
    Ok(json!({ "contents": [{ "uri": uri, "mimeType": mime, "text": text }] }))
}

/// Machine-readable coverage map: PVsyst area -> how the MCP covers it.
fn cobertura() -> Value {
    json!({
        "principle": "PVsyst state lives in text files (PVObject), a documented CLI, and a Win32/VCL desktop app. The MCP exposes generic primitives over ALL THREE, so coverage is bounded by the artifact surface, not by a fixed set of buttons.",
        "coverage_layers": [
            "FILES  - read/write any field of any PVObject file (leer/editar)",
            "CLI    - run any PVsystCLI command (pvsyst_cli, simular, sitio, convertir_meteo)",
            "GUI    - drive any window/control of the desktop app via Win32 automation (gui_observar/gui_actuar)"
        ],
        "gui_automation_note": "GUI-only features (3D shading editor, informative I/V & table tools, import wizards, dialogs) are covered by enumerating the real Win32/VCL windows and posting standard window messages. No PVsyst code is read or disassembled.",
        "areas": [
            { "area": "Easy mode / end-to-end (LLM-friendly)", "tools": ["entorno", "disenar_sistema", "sitio crear", "resultados kpis"], "notes": "Intention-level tools that collapse many steps into one call; the LLM asks 'entorno' then 'disenar_sistema'." },
            { "area": "Workspace", "tools": ["entorno diagnostico", "listar que:archivos", "leer", "escribir_archivo"] },
            { "area": "Projects & variants", "tools": ["listar que:proyectos", "leer", "proyecto crear", "proyecto eliminar", "proyecto renombrar", "proyecto crear_variante", "editar campos", "proyecto dimensionar"] },
            { "area": "Orientation / array sizing", "tools": ["editar campos", "proyecto dimensionar", "editar"] },
            { "area": "Component database (all kinds)", "tools": ["listar que:componentes", "leer", "componente crear", "componente borrar", "listar que:modulos"] },
            { "area": "Site & weather", "tools": ["sitio crear", "sitio convertir_meteo", "editar", "leer"] },
            { "area": "Shadings/horizon (.HOR/.SHD/.BLD/.SHO)", "tools": ["leer", "editar", "escribir_archivo"] },
            { "area": "Losses, IAM, thermal, quality", "tools": ["editar campos", "editar"] },
            { "area": "Models/templates (.LOD/.SPL/.SFI/.RVT)", "tools": ["leer", "editar", "escribir_archivo"] },
            { "area": "Simulation (single, batch, synthetic, param override)", "tools": ["simular ejecutar", "pvsyst_cli"] },
            { "area": "Comparison with measured data (variants CMx)", "tools": ["leer", "editar", "gui_actuar"], "notes": "PVsystCLI cannot simulate CMx variants (verified: 'Simulation cannot be run: the orientation is not defined'); run them from the GUI." },
            { "area": "Batch / parametric runs", "tools": ["simular preparar", "simular ejecutar", "simular barrido"], "notes": "filas -> batch CSV (-bpf)." },
            { "area": "Advanced/hidden parameter override", "tools": ["simular preparar", "simular ejecutar"], "notes": "parametros -> Param_Modif.dat (-ipf)." },
            { "area": "Site & weather readers", "tools": ["leer"] },
            { "area": "Results & reports", "tools": ["resultados csv", "leer"] },
            { "area": "Workspace management (create/export/import)", "tools": ["workspace crear", "workspace exportar", "workspace importar", "listar que:archivos"] },
            { "area": "Component & project import/export", "tools": ["componente importar", "componente exportar", "workspace exportar_proyecto"] },
            { "area": "Licensing / logs / version / help", "tools": ["entorno licencia", "entorno exportar_logs", "pvsyst_cli"] },
            { "area": "GUI-only authoring (3D shading editor, informative I/V & tables, wizards, dialogs)", "tools": ["gui_observar", "gui_actuar"], "notes": "Win32 UI automation; covers invocation/reading/capture of GUI-only functions." },
            { "area": "Any other PVsyst field", "tools": ["leer", "listar que:objetos", "editar"] }
        ]
    })
}
