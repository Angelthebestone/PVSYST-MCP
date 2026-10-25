//! Additional schema constraints and property annotations for tool parameters.

use serde_json::{json, Value};

/// Inclusive integer bounds `(propiedad, mínimo, máximo)`. One table feeds both the
/// schema (`minimum`/`maximum`) and the runtime check in `validacion`.
const LIMITES_ENTEROS: &[(&str, i64, Option<i64>)] = &[
    ("n", 1, None),
    ("lineas", 1, Some(500)),
    ("n_baterias_serie", 1, None),
    ("n_baterias_paralelo", 1, None),
];

/// Bounds `(mínimo, máximo)` declared for the integer property `clave`, if any.
pub(crate) fn limites_enteros(clave: &str) -> Option<(i64, Option<i64>)> {
    LIMITES_ENTEROS
        .iter()
        .find(|(nombre, _, _)| *nombre == clave)
        .map(|&(_, minimo, maximo)| (minimo, maximo))
}

/// Annotates known domain constraints on specific tool properties in the MCP schema.
/// The GUI-specific annotations only apply to the GUI tools: other tools may reuse
/// the same property names (`texto`) with a different meaning.
pub(crate) fn anotar_restricciones(prop_val: &mut Value, herramienta: &str, clave: &str) {
    let Some(obj) = prop_val.as_object_mut() else {
        return;
    };
    if let Some((minimo, maximo)) = limites_enteros(clave) {
        obj.insert("minimum".to_string(), json!(minimo));
        if let Some(maximo) = maximo {
            obj.insert("maximum".to_string(), json!(maximo));
        }
    }
    if anotar_estructuras(obj, clave) {
        return;
    }
    match clave {
        "potencia_kw" => {
            obj.insert(
                "description".to_string(),
                json!("Potencia límite de inyección en kW. Para quitar el límite, usa editar con PLimGrid: null."),
            );
        }
        "estrategia" => {
            obj.insert("enum".to_string(), json!(["autoconsumo", "recorte_picos"]));
        }
        "tipo_sistema" => {
            obj.insert("enum".to_string(), json!(["red", "aislado"]));
        }
        "via" => {
            obj.insert("enum".to_string(), json!(["pvsyst", "aproximacion"]));
            obj.insert(
                "description".to_string(),
                json!("pvsyst (defecto): la ruta de PVsyst. aproximacion: cálculo propio del MCP, oculto salvo con PVSYST_MCP_APROXIMACIONES=1."),
            );
        }
        "n_baterias_serie" | "n_baterias_paralelo" => {
            obj.insert(
                "description".to_string(),
                json!("Solo aplica a tipo_sistema:aislado."),
            );
        }
        _ if herramienta.starts_with("gui_") => anotar_gui(obj, clave),
        _ => {}
    }
}

fn anotar_estructuras(obj: &mut serde_json::Map<String, Value>, clave: &str) -> bool {
    match clave {
        "medido_kwh" => {
            obj.insert("items".to_string(), json!({ "type": "number" }));
            obj.insert("minItems".to_string(), json!(12));
            obj.insert("maxItems".to_string(), json!(12));
            true
        }
        "filas" | "ediciones" | "puntos" | "pasos" => {
            obj.insert("items".to_string(), json!({ "type": "object" }));
            true
        }
        "valores" => {
            obj.insert(
                "anyOf".to_string(),
                json!([
                    {
                        "type": "array",
                        "items": {
                            "type": "array",
                            "items": { "type": "number" }
                        }
                    },
                    { "type": "object" }
                ]),
            );
            obj.remove("type");
            true
        }
        _ => false,
    }
}

/// Schema details of the GUI-automation arguments.
fn anotar_gui(obj: &mut serde_json::Map<String, Value>, clave: &str) {
    match clave {
        "campo" => {
            obj.insert(
                "description".to_string(),
                json!("Campo sin rótulo: clase VCL y centro en px a 96 DPI dentro del formulario."),
            );
            obj.insert(
                "properties".to_string(),
                json!({
                    "clase": {
                        "type": "string",
                        "enum": ["TEdit", "TComboBox", "TRadioButton", "TCheckBox"],
                        "default": "TEdit"
                    },
                    "x": { "type": "integer" },
                    "y": { "type": "integer" }
                }),
            );
            obj.insert("required".to_string(), json!(["x", "y"]));
        }
        "texto" => {
            obj.insert(
                "description".to_string(),
                json!("Texto a escribir, o prefijo del ítem del combo a seleccionar."),
            );
        }
        "pids" => {
            obj.insert("items".to_string(), json!({ "type": "integer" }));
            obj.insert(
                "description".to_string(),
                json!("Procesos objetivo; si se omite, se usa PVsyst.exe."),
            );
        }
        _ => {}
    }
}
