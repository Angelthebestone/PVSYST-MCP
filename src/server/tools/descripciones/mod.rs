//! Argument descriptions of the tool schemas, so a client reads what a key means,
//! its unit and its range before it calls. A key that every tool reads the same
//! way is described once in `comunes`; a key that means something in one tool only
//! is described in that tool's table, which wins over the shared one.

use super::declaracion::Anotador;
use crate::idioma::{Idioma, Texto};
use serde_json::{json, Value};

mod componente;
mod comunes;
mod disenar_sistema;
mod editar;
mod entorno;
mod escribir_archivo;
mod gui_actuar;
mod gui_observar;
mod leer;
mod listar;
mod proyecto;
mod pvsyst_cli;
mod resultados;
mod simular;
mod sitio;
mod workspace;

/// What one argument key means, in both languages.
pub(super) struct Entrada {
    clave: &'static str,
    texto: Texto,
}

/// An argument description: `clave` and its text in Spanish and English.
const fn entrada(clave: &'static str, es: &'static str, en: &'static str) -> Entrada {
    Entrada {
        clave,
        texto: Texto { es, en },
    }
}

/// The table of one tool; a tool that has none yet uses only the shared keys.
fn tabla(herramienta: &str) -> &'static [Entrada] {
    match herramienta {
        "componente" => componente::TABLA,
        "disenar_sistema" => disenar_sistema::TABLA,
        "editar" => editar::TABLA,
        "entorno" => entorno::TABLA,
        "escribir_archivo" => escribir_archivo::TABLA,
        "gui_actuar" => gui_actuar::TABLA,
        "gui_observar" => gui_observar::TABLA,
        "leer" => leer::TABLA,
        "listar" => listar::TABLA,
        "proyecto" => proyecto::TABLA,
        "pvsyst_cli" => pvsyst_cli::TABLA,
        "resultados" => resultados::TABLA,
        "simular" => simular::TABLA,
        "sitio" => sitio::TABLA,
        "workspace" => workspace::TABLA,
        _ => &[],
    }
}

/// Description of `clave` for `herramienta`: its own table first, then the shared one.
pub(super) fn describir(herramienta: &str, clave: &str, idioma: Idioma) -> Option<&'static str> {
    tabla(herramienta)
        .iter()
        .chain(comunes::TABLA)
        .find(|e| e.clave == clave)
        .map(|e| e.texto.en_idioma(idioma))
}

/// Gives the property its description when nothing else wrote one: the
/// annotators that know a constraint (`restricciones_esquema`) say it better.
pub(super) fn anotar(an: &Anotador, prop_val: &mut Value, clave: &str) {
    let Some(obj) = prop_val.as_object_mut() else {
        return;
    };
    if obj.contains_key("description") {
        return;
    }
    if let Some(texto) = describir(an.herramienta.nombre, clave, an.idioma) {
        obj.insert("description".to_string(), json!(texto));
    }
}

/// The `detalle` every tool accepts; the schema adds it outside the declared keys.
pub(super) fn detalle(idioma: Idioma) -> &'static str {
    describir("", "detalle", idioma).unwrap_or_default()
}

#[cfg(test)]
#[path = "../../../../tests/unit/server/tools/descripciones_tests.rs"]
mod tests;
