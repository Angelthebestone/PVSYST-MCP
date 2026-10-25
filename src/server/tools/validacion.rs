//! Boundary validation for tool arguments: discriminator resolution, required
//! presence, and declared JSON types. Kept apart from the declarations so each
//! file stays small and the validation rules have one home.

use super::declaracion::{nombre_tipo_recibido, Herramienta};
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

    /// Normalizes aliases to canonical names and injects default discriminator if omitted.
    pub(crate) fn preparar_args(&self, args: &Value) -> Result<Value> {
        let mut mapa = match args {
            Value::Object(m) => m.clone(),
            _ => serde_json::Map::new(),
        };

        normalizar_alias_argumentos(self.sinonimos.argumentos, &mut mapa)?;
        normalizar_discriminador(
            self.discriminador.as_ref(),
            self.sinonimos.valores,
            &mut mapa,
        );

        Ok(Value::Object(mapa))
    }

    fn validar_discriminador(&self, args: &Value) -> Result<String> {
        let Some(disc) = &self.discriminador else {
            return Ok(String::new());
        };
        let valor = self.extraer_valor_discriminador(disc, args)?;

        if disc.valores.contains(&valor.as_str()) {
            Ok(valor)
        } else {
            Err(Error::Invalid(format!(
                "Valor '{valor}' no válido para {} ({}); valores: {}",
                self.nombre,
                disc.campo,
                disc.valores.join(", ")
            )))
        }
    }

    fn extraer_valor_discriminador(
        &self,
        disc: &super::declaracion::Discriminador,
        args: &Value,
    ) -> Result<String> {
        match args.get(disc.campo) {
            Some(v) => match v.as_str() {
                Some(s) => Ok(s.to_string()),
                None => Err(Error::Invalid(format!(
                    "Tipo incorrecto para '{}' (esperado string, recibido {})",
                    disc.campo,
                    nombre_tipo_recibido(v)
                ))),
            },
            None => match disc.defecto {
                Some(def) => Ok(def.to_string()),
                None => {
                    let pista =
                        format!("para {}; valores: {}", self.nombre, disc.valores.join(", "));
                    Err(missing(disc.campo, &pista))
                }
            },
        }
    }

    fn validar_detalle(&self, args: &Value) -> Result<()> {
        let Some(v) = valor_no_nulo(args, "detalle") else {
            return Ok(());
        };
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
        if let Some(req) = self
            .requeridos
            .iter()
            .find(|&&r| esta_vacio_o_nulo(args, r))
        {
            return Err(missing(req, self.nombre));
        }
        Ok(())
    }

    fn validar_requeridos_por_accion(&self, args: &Value, accion: &str) -> Result<()> {
        let Some((_, claves)) = self
            .requeridos_por_accion
            .iter()
            .find(|(acc, _)| *acc == accion)
        else {
            return Ok(());
        };
        let destino = if accion.is_empty() {
            self.nombre.to_string()
        } else {
            format!("{} {accion}", self.nombre)
        };
        if let Some(req) = claves.iter().find(|&&r| esta_vacio_o_nulo(args, r)) {
            return Err(missing(req, &destino));
        }
        Ok(())
    }

    fn validar_via(&self, args: &Value) -> Result<()> {
        let Some(v) = valor_no_nulo(args, "via") else {
            return Ok(());
        };
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
            validar_tipo_propiedad(prop_nombre, *prop_tipo, args)?;
        }
        Ok(())
    }
}

fn normalizar_alias_argumentos(
    argumentos: &[(&str, &str)],
    mapa: &mut serde_json::Map<String, Value>,
) -> Result<()> {
    for &(alias, canonico) in argumentos {
        let Some(valor_alias) = mapa.remove(alias) else {
            continue;
        };
        match mapa.get(canonico) {
            Some(valor_canonico) if valor_canonico != &valor_alias => {
                return Err(Error::Invalid(format!(
                    "Conflicto de argumentos: se pasaron '{canonico}' y su alias '{alias}' con valores distintos"
                )));
            }
            Some(_) => {}
            None => {
                mapa.insert(canonico.to_string(), valor_alias);
            }
        }
    }
    Ok(())
}

fn normalizar_discriminador(
    disc_opt: Option<&super::declaracion::Discriminador>,
    valores: &[(&str, &str)],
    mapa: &mut serde_json::Map<String, Value>,
) {
    let Some(disc) = disc_opt else {
        return;
    };
    if let Some(val) = mapa.get_mut(disc.campo) {
        normalizar_valor_discriminador(val, valores);
    } else if let Some(def) = disc.defecto {
        mapa.insert(disc.campo.to_string(), json!(def));
    }
}

fn normalizar_valor_discriminador(val: &mut Value, valores: &[(&str, &str)]) {
    let Some(s) = val.as_str() else {
        return;
    };
    for &(alias, canonico) in valores {
        if s.eq_ignore_ascii_case(alias) {
            *val = Value::String(canonico.to_string());
            break;
        }
    }
}

fn validar_tipo_propiedad(
    prop_nombre: &str,
    prop_tipo: super::declaracion::TipoDato,
    args: &Value,
) -> Result<()> {
    let Some(val) = args.get(prop_nombre) else {
        return Ok(());
    };
    if val.is_null() || prop_tipo.es_compatible(val) {
        return Ok(());
    }
    Err(Error::Invalid(format!(
        "Tipo incorrecto para '{prop_nombre}' (esperado {}, recibido {})",
        prop_tipo.nombre_esperado(),
        nombre_tipo_recibido(val)
    )))
}

fn esta_vacio_o_nulo(args: &Value, clave: &str) -> bool {
    matches!(args.get(clave), None | Some(Value::Null))
}

fn valor_no_nulo<'a>(args: &'a Value, clave: &str) -> Option<&'a Value> {
    args.get(clave).filter(|v| !v.is_null())
}
