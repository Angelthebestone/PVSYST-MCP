//! Boundary validation for tool arguments: discriminator resolution, required
//! presence, and declared JSON types. Kept apart from the declarations so each
//! file stays small and the validation rules have one home.

use super::declaracion::{con, nombre_tipo_recibido, Herramienta};
use crate::args::missing;
use crate::error::{Error, Result};
use serde_json::{json, Value};

impl Herramienta {
    /// Validates arguments at the boundary and returns the resolved discriminator action.
    pub(crate) fn validar(&self, args: &Value) -> Result<String> {
        let accion = self.validar_discriminador(args)?;
        self.validar_detalle(args)?;
        self.validar_requeridos_globales(args)?;
        self.validar_requeridos_por_accion(args, &accion)?;
        self.validar_tipos(args)?;
        self.validar_via(args)?;
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
                    let pista =
                        format!("para {}; valores: {}", self.nombre, disc.valores.join(", "));
                    return Err(missing(disc.campo, &pista));
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
                return Err(missing(req, self.nombre));
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
                    return Err(missing(req, &destino));
                }
            }
        }
        Ok(())
    }

    fn validar_via(&self, args: &Value) -> Result<()> {
        let Some(v) = args.get("via") else {
            return Ok(());
        };
        if v.is_null() {
            return Ok(());
        }
        let Some(via) = v.as_str() else {
            return Ok(());
        };
        if via == "pvsyst" || via == crate::aproximaciones::VIA_APROXIMACION {
            return Ok(());
        }
        Err(Error::Invalid(format!(
            "Valor '{via}' no válido para via; valores: pvsyst, {}",
            crate::aproximaciones::VIA_APROXIMACION
        )))
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
