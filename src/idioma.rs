//! Client-facing language of the MCP surface: schemas, prompts, resources and
//! messages. Pure domain: it knows nothing about paths or process state, only
//! how to pick a wording.

/// Environment variable that picks the client-facing language.
pub(crate) const VARIABLE_IDIOMA: &str = "PVSYST_MCP_IDIOMA";

/// The language the MCP surface is written in for this installation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Idioma {
    /// English, the default.
    #[default]
    En,
    /// Spanish, with `PVSYST_MCP_IDIOMA=es`.
    Es,
}

impl Idioma {
    /// Pure decision behind [`de_entorno`]: `Es` only for `es` in any casing;
    /// anything else (unset, `en`, unknown) stays `En`.
    pub(crate) fn de_valor(valor: Option<&str>) -> Self {
        match valor.map(str::trim) {
            Some(v) if v.eq_ignore_ascii_case("es") => Self::Es,
            _ => Self::En,
        }
    }

    /// Reads [`VARIABLE_IDIOMA`] from the environment.
    pub(crate) fn de_entorno() -> Self {
        Self::de_valor(std::env::var(VARIABLE_IDIOMA).ok().as_deref())
    }
}

/// One client-facing string in both supported languages.
#[derive(Debug, Clone, Copy)]
pub(crate) struct Texto {
    pub(crate) es: &'static str,
    pub(crate) en: &'static str,
}

impl Texto {
    /// The wording in `idioma`.
    pub(crate) fn en_idioma(&self, idioma: Idioma) -> &'static str {
        match idioma {
            Idioma::Es => self.es,
            Idioma::En => self.en,
        }
    }

    /// The wording in `idioma` with every `{clave}` placeholder substituted.
    pub(crate) fn texto_con(&self, idioma: Idioma, vars: &[(&str, &str)]) -> String {
        vars.iter()
            .fold(self.en_idioma(idioma).to_string(), |acc, (clave, valor)| {
                acc.replace(&format!("{{{clave}}}"), valor)
            })
    }
}

#[cfg(test)]
#[path = "../tests/unit/idioma.rs"]
mod tests;
