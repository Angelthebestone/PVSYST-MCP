//! Argument aliases: canonical names, discriminator defaults and conflicts.

use super::declaracion::Discriminador;
use crate::error::{Error, Result};
use crate::idioma::{Idioma, Texto};
use serde_json::{json, Value};

const CONFLICTO_ALIAS: Texto = Texto {
    es: "Conflicto de argumentos: se pasaron '{canonico}' y su alias '{alias}' con valores distintos",
    en: "Argument conflict: both '{canonico}' and its alias '{alias}' were passed with different values",
};

pub(super) fn normalizar_alias_argumentos(
    argumentos: &[(&str, &str)],
    mapa: &mut serde_json::Map<String, Value>,
    idioma: Idioma,
) -> Result<()> {
    for &(alias, canonico) in argumentos {
        let Some(valor_alias) = mapa.remove(alias) else {
            continue;
        };
        match mapa.get(canonico) {
            Some(valor_canonico) if valor_canonico != &valor_alias => {
                return Err(Error::Invalid(
                    CONFLICTO_ALIAS.texto_con(idioma, &[("canonico", canonico), ("alias", alias)]),
                ));
            }
            Some(_) => {}
            None => {
                mapa.insert(canonico.to_string(), valor_alias);
            }
        }
    }
    Ok(())
}

pub(super) fn normalizar_discriminador(
    disc_opt: Option<&Discriminador>,
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
    let Some(canonico) = val.as_str().and_then(|s| canonico_de_alias(valores, s)) else {
        return;
    };
    *val = Value::String(canonico.to_string());
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
