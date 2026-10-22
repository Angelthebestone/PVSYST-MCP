//! MCP tool declarations: schemas, property types and discriminator rules.
//! Boundary validation lives in [`validacion`].

use crate::config::Paths;
use crate::error::Result;
use crate::progreso::SeguimientoTarea;
use serde_json::{json, Map, Value};

/// JSON Schema types supported in tool property declarations.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum TipoDato {
    String,
    Number,
    Integer,
    Boolean,
    Object,
    Array,
    IntegerOrString,
    NumberOrString,
    ArrayOrObject,
}

impl TipoDato {
    /// Validates whether a JSON value matches the expected type.
    pub(crate) fn es_compatible(&self, v: &Value) -> bool {
        match self {
            Self::String => v.is_string(),
            Self::Number => v.is_number(),
            Self::Integer => v.is_i64() || v.is_u64(),
            Self::Boolean => v.is_boolean(),
            Self::Object => v.is_object(),
            Self::Array => v.is_array(),
            Self::IntegerOrString => (v.is_i64() || v.is_u64()) || v.is_string(),
            Self::NumberOrString => v.is_number() || v.is_string(),
            Self::ArrayOrObject => v.is_array() || v.is_object(),
        }
    }

    /// Readable name of the expected type for error reporting.
    pub(crate) fn nombre_esperado(&self) -> &'static str {
        match self {
            Self::String => "string",
            Self::Number => "number",
            Self::Integer => "integer",
            Self::Boolean => "boolean",
            Self::Object => "object",
            Self::Array => "array",
            Self::IntegerOrString => "integer|string",
            Self::NumberOrString => "number|string",
            Self::ArrayOrObject => "array|object",
        }
    }

    /// JSON schema fragment representing this type.
    pub(crate) fn esquema_json(&self) -> Value {
        match self {
            Self::String => json!({ "type": "string" }),
            Self::Number => json!({ "type": "number" }),
            Self::Integer => json!({ "type": "integer" }),
            Self::Boolean => json!({ "type": "boolean" }),
            Self::Object => json!({ "type": "object" }),
            Self::Array => json!({ "type": "array" }),
            Self::IntegerOrString => json!({ "type": ["integer", "string"] }),
            Self::NumberOrString => json!({ "type": ["number", "string"] }),
            Self::ArrayOrObject => json!({ "type": ["array", "object"] }),
        }
    }
}

/// Identifies the runtime JSON type of `v` for validation diagnostics.
pub(crate) fn nombre_tipo_recibido(v: &Value) -> &'static str {
    match v {
        Value::Null => "null",
        Value::Bool(_) => "boolean",
        Value::Number(n) if n.is_i64() || n.is_u64() => "integer",
        Value::Number(_) => "number",
        Value::String(_) => "string",
        Value::Array(_) => "array",
        Value::Object(_) => "object",
    }
}

/// Discriminator configuration for multi-action tools (e.g. `accion` or `que`).
pub(crate) struct Discriminador {
    pub(crate) campo: &'static str,
    pub(crate) defecto: Option<&'static str>,
    pub(crate) valores: &'static [&'static str],
}

/// How a tool runs: plain, or receiving progress and cancellation.
#[derive(Clone, Copy)]
pub(crate) enum Ejecutor {
    Simple(fn(&Paths, &Value) -> Result<Value>),
    Seguido(fn(&Paths, &Value, &SeguimientoTarea<'_>) -> Result<Value>),
}

/// MCP behaviour hints and display title for one tool.
pub(crate) struct Anotaciones {
    pub(crate) titulo: &'static str,
    pub(crate) es_solo_lectura: bool,
    pub(crate) es_destructiva: bool,
    pub(crate) es_idempotente: bool,
    pub(crate) usa_red: bool,
}

/// A consolidated MCP tool declaration coupling schema, validation, and handler.
pub(crate) struct Herramienta {
    pub(crate) nombre: &'static str,
    pub(crate) descripcion: &'static str,
    pub(crate) anotaciones: Anotaciones,
    pub(crate) propiedades: &'static [(&'static str, TipoDato)],
    pub(crate) requeridos: &'static [&'static str],
    pub(crate) discriminador: Option<Discriminador>,
    pub(crate) requeridos_por_accion: &'static [(&'static str, &'static [&'static str])],
    pub(crate) ejecutar: Ejecutor,
}

impl Herramienta {
    /// Builds the MCP tool schema exposed to clients in `tools/list`.
    pub(crate) fn esquema(&self) -> Value {
        // No `outputSchema`: one tool returns different shapes per action.
        let mut props = Map::new();
        let mut req: Vec<&str> = self.requeridos.to_vec();

        for (clave, tipo) in self.propiedades {
            let mut prop_val = tipo.esquema_json();
            if *clave == "workspace" {
                prop_val["description"] = json!(
                    "Ruta del workspace que reemplaza la raíz de confinamiento para esta llamada. Requiere cliente MCP de confianza."
                );
            }
            anotar_restricciones(&mut prop_val, clave);
            props.insert((*clave).to_string(), prop_val);
        }

        if let Some(disc) = &self.discriminador {
            let mut prop = json!({ "type": "string", "enum": disc.valores });
            if let Some(defecto) = disc.defecto {
                prop["default"] = json!(defecto);
            } else {
                req.push(disc.campo);
            }
            props.insert(disc.campo.to_string(), prop);
        }

        props.insert(
            "detalle".to_string(),
            json!({ "type": "string", "enum": super::recorte::DETALLES }),
        );

        json!({
            "name": self.nombre,
            "title": self.anotaciones.titulo,
            "description": self.descripcion,
            "annotations": {
                "title": self.anotaciones.titulo,
                "readOnlyHint": self.anotaciones.es_solo_lectura,
                "destructiveHint": self.anotaciones.es_destructiva,
                "idempotentHint": self.anotaciones.es_idempotente,
                "openWorldHint": self.anotaciones.usa_red,
            },
            "inputSchema": {
                "type": "object",
                "properties": props,
                "required": req,
            }
        })
    }
}

fn anotar_restricciones(prop_val: &mut Value, clave: &str) {
    match clave {
        "potencia_kw" => {
            prop_val["description"] = json!(
                "Potencia límite de inyección en kW. Para quitar el límite, usa editar con PLimGrid: null."
            );
        }
        "estrategia" => {
            prop_val["enum"] = json!(["autoconsumo", "recorte_picos"]);
        }
        "tipo_sistema" => {
            prop_val["enum"] = json!(["red", "aislado"]);
        }
        "via" => {
            prop_val["enum"] = json!(["pvsyst", "aproximacion"]);
            prop_val["description"] = json!(
                "pvsyst (defecto): la ruta de PVsyst. aproximacion: cálculo propio del MCP, oculto salvo con PVSYST_MCP_APROXIMACIONES=1."
            );
        }
        "medido_kwh" => {
            prop_val["items"] = json!({ "type": "number" });
            prop_val["minItems"] = json!(12);
            prop_val["maxItems"] = json!(12);
        }
        "n_baterias_serie" | "n_baterias_paralelo" => {
            prop_val["minimum"] = json!(1);
            prop_val["description"] = json!("Solo aplica a tipo_sistema:aislado.");
        }
        _ => anotar_gui(prop_val, clave),
    }
}

/// Schema details of the GUI-automation arguments.
fn anotar_gui(prop_val: &mut Value, clave: &str) {
    match clave {
        "campo" => {
            prop_val["description"] =
                json!("Campo sin rótulo: clase VCL y centro en px a 96 DPI dentro del formulario.");
            prop_val["properties"] = json!({
                "clase": {
                    "type": "string",
                    "enum": ["TEdit", "TComboBox", "TRadioButton", "TCheckBox"],
                    "default": "TEdit"
                },
                "x": { "type": "integer" },
                "y": { "type": "integer" }
            });
            prop_val["required"] = json!(["x", "y"]);
        }
        "texto" => {
            prop_val["description"] =
                json!("Texto a escribir, o prefijo del ítem del combo a seleccionar.");
        }
        "pids" => {
            prop_val["items"] = json!({ "type": "integer" });
            prop_val["description"] = json!("Procesos objetivo; si se omite, se usa PVsyst.exe.");
        }
        "lineas" => {
            prop_val["minimum"] = json!(1);
            prop_val["maximum"] = json!(500);
        }
        _ => {}
    }
}

/// Produces a copy of `args` with the given key/value set.
pub(crate) fn con(args: &Value, clave: &str, valor: Value) -> Value {
    let mut copia = match args {
        Value::Object(m) => Value::Object(m.clone()),
        _ => Value::Object(Map::new()),
    };
    if let Some(m) = copia.as_object_mut() {
        m.insert(clave.to_string(), valor);
    }
    copia
}
