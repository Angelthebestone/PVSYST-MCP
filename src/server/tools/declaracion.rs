//! Definition of MCP tools: schema declarations, property types,
//! discriminator rules and validation logic forming a single source of truth.

use crate::config::Paths;
use crate::error::{Error, Result};
use serde_json::{json, Map, Value};

/// Access level required to execute or discover a tool.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Acceso {
    /// Available to every MCP client session.
    Normal,
    /// Requires `PVSYST_MCP_EXPERTO=1` on the server.
    Experto,
}

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

/// A consolidated MCP tool declaration coupling schema, validation, and handler.
pub(crate) struct Herramienta {
    pub(crate) nombre: &'static str,
    pub(crate) descripcion: &'static str,
    pub(crate) acceso: Acceso,
    pub(crate) propiedades: &'static [(&'static str, TipoDato)],
    pub(crate) requeridos: &'static [&'static str],
    pub(crate) discriminador: Option<Discriminador>,
    pub(crate) requeridos_por_accion: &'static [(&'static str, &'static [&'static str])],
    pub(crate) ejecutar: fn(&Paths, &Value) -> Result<Value>,
}

impl Herramienta {
    /// Builds the MCP tool schema exposed to clients in `tools/list`.
    pub(crate) fn esquema(&self) -> Value {
        let mut props = Map::new();
        let mut req: Vec<&str> = self.requeridos.to_vec();

        for (clave, tipo) in self.propiedades {
            let mut prop_val = tipo.esquema_json();
            if *clave == "workspace" {
                prop_val["description"] = json!(
                    "Ruta del workspace que reemplaza la raíz de confinamiento para esta llamada. Requiere cliente MCP de confianza."
                );
            }
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
            "description": self.descripcion,
            "inputSchema": {
                "type": "object",
                "properties": props,
                "required": req,
                "additionalProperties": false,
            }
        })
    }

    /// Validates arguments at the boundary and returns the resolved discriminator action.
    pub(crate) fn validar(&self, args: &Value) -> Result<String> {
        let accion = self.validar_discriminador(args)?;
        self.validar_detalle(args)?;
        self.validar_requeridos_globales(args)?;
        self.validar_requeridos_por_accion(args, &accion)?;
        self.validar_tipos(args)?;
        Ok(accion)
    }

    /// Injects the resolved discriminator value if omitted by the caller.
    pub(crate) fn preparar_args(&self, args: &Value, accion: &str) -> Value {
        let Some(disc) = &self.discriminador else {
            return args.clone();
        };
        if args.get(disc.campo).is_none() {
            con(args, disc.campo, json!(accion))
        } else {
            args.clone()
        }
    }

    fn validar_discriminador(&self, args: &Value) -> Result<String> {
        let Some(disc) = &self.discriminador else {
            return Ok(String::new());
        };
        let valor = match args.get(disc.campo) {
            Some(v) => {
                let Some(s) = v.as_str() else {
                    return Err(Error::Invalid(format!(
                        "Tipo incorrecto para '{}' (esperado string, recibido {})",
                        disc.campo,
                        nombre_tipo_recibido(v)
                    )));
                };
                s
            }
            None => match disc.defecto {
                Some(def) => def,
                None => {
                    return Err(Error::Invalid(format!(
                        "Falta '{}' para {}; valores: {}",
                        disc.campo,
                        self.nombre,
                        disc.valores.join(", ")
                    )));
                }
            },
        };

        if disc.valores.contains(&valor) {
            Ok(valor.to_string())
        } else {
            Err(Error::Invalid(format!(
                "Valor '{valor}' no válido para {} ({}); valores: {}",
                self.nombre,
                disc.campo,
                disc.valores.join(", ")
            )))
        }
    }

    fn validar_detalle(&self, args: &Value) -> Result<()> {
        let Some(v) = args.get("detalle") else {
            return Ok(());
        };
        if v.is_null() {
            return Ok(());
        }
        let Some(d) = v.as_str() else {
            return Err(Error::Invalid(format!(
                "Tipo incorrecto para 'detalle' (esperado string, recibido {})",
                nombre_tipo_recibido(v)
            )));
        };
        if !super::recorte::DETALLES.contains(&d) {
            return Err(Error::Invalid(format!(
                "Valor '{d}' no válido para detalle; valores: {}",
                super::recorte::DETALLES.join(", ")
            )));
        }
        Ok(())
    }

    fn validar_requeridos_globales(&self, args: &Value) -> Result<()> {
        for req in self.requeridos {
            if args.get(*req).is_none_or(Value::is_null) {
                return Err(Error::Invalid(format!(
                    "Falta '{req}' para {}",
                    self.nombre
                )));
            }
        }
        Ok(())
    }

    fn validar_requeridos_por_accion(&self, args: &Value, accion: &str) -> Result<()> {
        if let Some((_, claves)) = self
            .requeridos_por_accion
            .iter()
            .find(|(acc, _)| *acc == accion)
        {
            for req in *claves {
                if args.get(*req).is_none_or(Value::is_null) {
                    let destino = if accion.is_empty() {
                        self.nombre.to_string()
                    } else {
                        format!("{} {accion}", self.nombre)
                    };
                    return Err(Error::Invalid(format!("Falta '{req}' para {destino}")));
                }
            }
        }
        Ok(())
    }

    fn validar_tipos(&self, args: &Value) -> Result<()> {
        for (prop_nombre, prop_tipo) in self.propiedades {
            if let Some(val) = args.get(*prop_nombre) {
                if !val.is_null() && !prop_tipo.es_compatible(val) {
                    return Err(Error::Invalid(format!(
                        "Tipo incorrecto para '{prop_nombre}' (esperado {}, recibido {})",
                        prop_tipo.nombre_esperado(),
                        nombre_tipo_recibido(val)
                    )));
                }
            }
        }
        Ok(())
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
