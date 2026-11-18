//! The `avisos` list: every key of the answer that carries a warning, without
//! repeats, cut to the first few.

use super::super::campos::CLAVES_AVISO;
use super::super::tipos::Recorte;
use super::textos_de;
use crate::idioma::{Idioma, Texto};
use serde_json::Value;
use std::collections::HashSet;

/// Warnings listed in `vista`; the answer keeps all of them in full.
const MAX_AVISOS: usize = 5;
const MAX_CARACTERES_AVISO: usize = 160;
const PEDIR_AVISOS: Texto = Texto {
    es: "leer avisos completos en la propia respuesta",
    en: "read the full warnings in the answer itself",
};

pub(super) fn avisos_de(salida: &Value) -> Vec<String> {
    let mut avisos: Vec<String> = CLAVES_AVISO
        .iter()
        .flat_map(|clave| textos_de(salida, clave))
        .collect();
    let mut vistos = HashSet::new();
    avisos.retain(|a| vistos.insert(a.clone()));
    avisos
        .into_iter()
        .map(|a| a.chars().take(MAX_CARACTERES_AVISO).collect())
        .collect()
}

/// The first warnings, and the cut when there are more: the answer keeps all of them.
pub(super) fn acotar_avisos(
    mut avisos: Vec<String>,
    idioma: Idioma,
) -> (Vec<String>, Option<Recorte>) {
    let total = avisos.len();
    avisos.truncate(MAX_AVISOS);
    let recorte = (total > MAX_AVISOS).then(|| Recorte {
        clave: "vista.avisos".into(),
        mostrados: MAX_AVISOS,
        total,
        pedir_resto: PEDIR_AVISOS.en_idioma(idioma),
    });
    (avisos, recorte)
}
