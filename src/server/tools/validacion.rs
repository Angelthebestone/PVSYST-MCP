//! Boundary validation for tool arguments: discriminator resolution, required
//! presence, and declared JSON types. Kept apart from the declarations so each
//! file stays small and the validation rules have one home.

use super::declaracion::{nombre_tipo_recibido, Herramienta};
use super::restricciones_esquema::limites_enteros;
use crate::args::missing;
use crate::error::{Error, Result};
use serde_json::{json, Value};

impl Herramienta {
    /// Validates arguments at the boundary and returns the resolved discriminator action.
    pub(crate) fn validar(&self, args: &Value, aproximaciones: bool) -> Result<String> {
        let accion = self.validar_discriminador(args, aproximaciones)?;
        self.validar_detalle(args)?;
        self.validar_requeridos(args, &accion)?;
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

    fn validar_discriminador(&self, args: &Value, aproximaciones: bool) -> Result<String> {
        let Some(disc) = &self.discriminador else {
            return Ok(String::new());
        };
        let valor = self.extraer_valor_discriminador(disc, args)?;

        if disc.valores.contains(&valor.as_str()) {
            Ok(valor)
        } else {
            let visibles: Vec<&str> = disc
                .valores
                .iter()
                .copied()
                .filter(|accion| {
                    aproximaciones || super::aproximadas::de(self.nombre, accion).is_none()
                })
                .collect();
            Err(Error::Invalid(format!(
                "Valor '{valor}' no válido para {} ({}); valores: {}",
                self.nombre,
                disc.campo,
                visibles.join(", ")
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

/// Canonical value behind `valor` when it is a (case-insensitive) alias in `tabla`.
pub(super) fn canonico_de_alias<'a>(
    tabla: &'a [(&'a str, &'a str)],
    valor: &str,
) -> Option<&'a str> {
    tabla
        .iter()
        .find(|(alias, _)| valor.eq_ignore_ascii_case(alias))
        .map(|&(_, canonico)| canonico)
}

fn normalizar_valor_discriminador(val: &mut Value, valores: &[(&str, &str)]) {
    let Some(canonico) = val.as_str().and_then(|s| canonico_de_alias(valores, s)) else {
        return;
    };
    *val = Value::String(canonico.to_string());
}

fn validar_tipo_propiedad(
    prop_nombre: &str,
    prop_tipo: super::declaracion::TipoDato,
    args: &Value,
) -> Result<()> {
    let Some(val) = args.get(prop_nombre) else {
        return Ok(());
    };
    if val.is_null() {
        return Ok(());
    }
    if !prop_tipo.es_compatible(val) {
        return Err(Error::Invalid(format!(
            "Tipo incorrecto para '{prop_nombre}' (esperado {}, recibido {})",
            prop_tipo.nombre_esperado(),
            nombre_tipo_recibido(val)
        )));
    }
    validar_limites_enteros(prop_nombre, val)
}

/// Rejects an integer outside the bounds the schema declares for `prop_nombre`.
fn validar_limites_enteros(prop_nombre: &str, val: &Value) -> Result<()> {
    let (Some((minimo, maximo)), Some(n)) = (limites_enteros(prop_nombre), val.as_i64()) else {
        return Ok(());
    };
    if n >= minimo && maximo.is_none_or(|max| n <= max) {
        return Ok(());
    }
    let rango = maximo.map_or_else(
        || format!("mínimo {minimo}"),
        |max| format!("entre {minimo} y {max}"),
    );
    Err(Error::Invalid(format!(
        "Valor {n} fuera de rango para '{prop_nombre}' ({rango})"
    )))
}

fn valor_no_nulo<'a>(args: &'a Value, clave: &str) -> Option<&'a Value> {
    args.get(clave).filter(|v| !v.is_null())
}
