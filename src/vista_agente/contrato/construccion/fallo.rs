//! The `error` block of an answer that failed or did not finish: code, reason,
//! probable cause and remedy.

use super::super::tipos::{ErrorVista, Estado};
use super::estado::falla_la_cli;
use super::textos_de;
use crate::idioma::{Idioma, Texto};
use serde_json::Value;

const MAX_CARACTERES_MOTIVO: usize = 300;
/// Reason of a failure the answer does not explain.
const MOTIVO_SIN_DETALLE: Texto = Texto {
    es: "la acción informó un fallo",
    en: "the action reported a failure",
};
const CAUSA_DE_FALLO: Texto = Texto {
    es: "La acción informó un fallo o problemas en su resultado; algunos pasos pudieron aplicarse.",
    en:
        "The action reported a failure or problems in its result; some steps may have been applied.",
};
const REMEDIO_DE_FALLO: Texto = Texto {
    es: "Corrige la causa indicada en motivo y avisos y repite la acción.",
    en: "Fix the cause named in motivo and avisos and repeat the action.",
};

pub(super) fn motivo_de(salida: &Value, idioma: Idioma) -> String {
    let errores = textos_de(salida, "errores");
    let motivo = match errores.is_empty() {
        false => errores.join("; "),
        true => ["detected_error", "error", "mensaje"]
            .iter()
            .find_map(|clave| textos_de(salida, clave).into_iter().next())
            .unwrap_or_else(|| MOTIVO_SIN_DETALLE.en_idioma(idioma).to_string()),
    };
    motivo.chars().take(MAX_CARACTERES_MOTIVO).collect()
}

fn codigo_de(estado: Estado, salida: &Value) -> &'static str {
    match estado {
        Estado::Error if falla_la_cli(salida) => "fallo_cli",
        Estado::Error => "fallo",
        _ => "fallo_parcial",
    }
}

pub(super) fn error_de(estado: Estado, salida: &Value, idioma: Idioma) -> Option<ErrorVista> {
    (estado != Estado::Ok).then(|| ErrorVista {
        codigo: codigo_de(estado, salida).into(),
        motivo: motivo_de(salida, idioma),
        causa_probable: CAUSA_DE_FALLO.en_idioma(idioma),
        que_hacer: REMEDIO_DE_FALLO.en_idioma(idioma),
        claves_validas: Vec::new(),
    })
}
