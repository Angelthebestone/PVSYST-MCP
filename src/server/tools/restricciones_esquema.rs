//! Additional schema constraints and property annotations for tool parameters.

use super::declaracion::Anotador;
use crate::idioma::{Idioma, Texto};
use serde_json::{json, Value};

/// Inclusive integer bounds `(propiedad, mínimo, máximo)`. One table feeds both the
/// schema (`minimum`/`maximum`) and the runtime check in `validacion`.
const LIMITES_ENTEROS: &[(&str, i64, Option<i64>)] = &[
    ("n", 1, None),
    ("lineas", 1, Some(500)),
    ("n_baterias_serie", 1, None),
    ("n_baterias_paralelo", 1, None),
    ("timeout_s", 1, Some(3600)),
];

/// Bounds `(mínimo, máximo)` declared for the integer property `clave`, if any.
pub(crate) fn limites_enteros(clave: &str) -> Option<(i64, Option<i64>)> {
    LIMITES_ENTEROS
        .iter()
        .find(|(nombre, _, _)| *nombre == clave)
        .map(|&(_, minimo, maximo)| (minimo, maximo))
}

const DESC_POTENCIA_KW: Texto = Texto {
    es: "Potencia límite de inyección en kW. Para quitar el límite, usa editar con PLimGrid: null.",
    en: "Injection power limit in kW. To remove the limit, use editar with PLimGrid: null.",
};
const DESC_VIA: Texto = Texto {
    es: "pvsyst (defecto): la ruta de PVsyst. aproximacion: cálculo propio del MCP, oculto salvo con PVSYST_MCP_APROXIMACIONES=1.",
    en: "pvsyst (default): the PVsyst route. aproximacion: the MCP's own calculation, hidden unless PVSYST_MCP_APROXIMACIONES=1.",
};
const DESC_BATERIAS: Texto = Texto {
    es: "Solo aplica a tipo_sistema:aislado.",
    en: "Only applies to tipo_sistema:aislado.",
};
const DESC_TIMEOUT: Texto = Texto {
    es: "Segundos antes de matar el proceso (1–3600). Por defecto 120, y 900 en run-simulation.",
    en: "Seconds before the process is killed (1–3600). Defaults to 120, and 900 for run-simulation.",
};
const DESC_DRY_RUN: Texto = Texto {
    es: "true: ensayo, no ejecuta nada y devuelve lo que haría. null no se acepta: ejecutaría de verdad.",
    en: "true: rehearsal, runs nothing and returns what it would do. null is refused: it would run for real.",
};
const DESC_OPCIONES_CLI: Texto = Texto {
    es: "Argumentos de PVsystCLI. Lista: cada elemento tal cual (`-w:C:/ws`). Objeto: `--nombre:valor`, en orden alfabético; una clave que empieza por `-` se usa tal cual, `true` es un flag sin valor y `false`/`null` omiten la opción.",
    en: "PVsystCLI arguments. List: each item as written (`-w:C:/ws`). Object: `--name:value`, in alphabetical order; a key starting with `-` is used as written, `true` is a flag with no value and `false`/`null` leave the option out.",
};

/// Annotates known domain constraints on specific tool properties in the MCP schema.
/// The GUI-specific annotations only apply to the GUI tools: other tools may reuse
/// the same property names (`texto`) with a different meaning.
pub(crate) fn anotar_restricciones(an: &Anotador, prop_val: &mut Value, clave: &str) {
    if reemplazar_esquema_opciones(an, prop_val, clave) {
        return;
    }
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
    if let Some(texto) = descripcion_de_ejecucion(an, clave) {
        obj.insert("description".to_string(), json!(texto));
    }
    let idioma = an.idioma;
    match clave {
        "potencia_kw" => {
            obj.insert(
                "description".to_string(),
                json!(DESC_POTENCIA_KW.en_idioma(idioma)),
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
            obj.insert("description".to_string(), json!(DESC_VIA.en_idioma(idioma)));
        }
        "n_baterias_serie" | "n_baterias_paralelo" => {
            obj.insert(
                "description".to_string(),
                json!(DESC_BATERIAS.en_idioma(idioma)),
            );
        }
        _ if an.herramienta.nombre.starts_with("gui_") => anotar_gui(obj, clave, idioma),
        _ => {}
    }
}

/// Description of the properties the tools that run PVsystCLI share.
fn descripcion_de_ejecucion(an: &Anotador, clave: &str) -> Option<String> {
    let texto = match clave {
        "timeout_s" => DESC_TIMEOUT,
        "dry_run" => DESC_DRY_RUN,
        _ => return None,
    };
    Some(texto.en_idioma(an.idioma).to_string())
}

/// The passthrough's `opciones` is the only free object of the schema: it declares
/// the scalar values it accepts instead of accepting anything.
fn reemplazar_esquema_opciones(an: &Anotador, prop_val: &mut Value, clave: &str) -> bool {
    if clave != "opciones" || an.herramienta.nombre != "pvsyst_cli" {
        return false;
    }
    *prop_val = json!({
        "anyOf": [
            { "type": "array", "items": { "type": "string" }, "minItems": 1 },
            {
                "type": "object",
                "additionalProperties": { "type": ["string", "number", "boolean", "null"] }
            }
        ],
        "description": DESC_OPCIONES_CLI.en_idioma(an.idioma),
    });
    true
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
        "objetos" => {
            obj.insert("items".to_string(), json!({ "type": "object" }));
            obj.insert("minItems".to_string(), json!(1));
            obj.insert("maxItems".to_string(), json!(crate::escena_3d::MAX_OBJETOS));
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
fn anotar_gui(obj: &mut serde_json::Map<String, Value>, clave: &str, idioma: Idioma) {
    match clave {
        "campo" => {
            obj.insert(
                "description".to_string(),
                json!(Texto {
                    es: "Campo sin rótulo: clase VCL y centro en px a 96 DPI dentro del formulario.",
                    en: "Unlabeled field: VCL class and center in px at 96 DPI inside the form.",
                }
                .en_idioma(idioma)),
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
                json!(Texto {
                    es: "Texto a escribir, o prefijo del ítem del combo a seleccionar.",
                    en: "Text to type, or prefix of the combo item to select.",
                }
                .en_idioma(idioma)),
            );
        }
        "pids" => {
            obj.insert("items".to_string(), json!({ "type": "integer" }));
            obj.insert(
                "description".to_string(),
                json!(Texto {
                    es: "Procesos objetivo; si se omite, se usa PVsyst.exe.",
                    en: "Target processes; if omitted, PVsyst.exe is used.",
                }
                .en_idioma(idioma)),
            );
        }
        _ => {}
    }
}
