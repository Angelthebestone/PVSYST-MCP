//! Captions of PVsyst's own windows in the languages of its interface.
//!
//! The server never needs to know the language to find a button: a [`Rotulo`]
//! holds the same caption in all four languages and matches a window text in
//! any of them. Only the Spanish captions were measured on a live PVsyst; the
//! rest are inferred from PVsyst's documentation and fail closed (the button is
//! not found) until they are measured on an install in that language.
//!
//! This is the language of PVsyst's windows, set in PVsyst. It is not
//! `PVSYST_MCP_IDIOMA`, which only picks the language of the MCP's messages.

pub(crate) mod dialogos;
pub(crate) mod pvsyst;

use std::fmt;

/// The language of PVsyst's user interface that the captions cover.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Idioma {
    Es,
    En,
    Fr,
    De,
}

impl Idioma {
    pub(crate) const TODOS: [Self; 4] = [Self::Es, Self::En, Self::Fr, Self::De];
}

/// One caption in each language of PVsyst's interface.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct Rotulo<'a> {
    pub(crate) es: &'a str,
    pub(crate) en: &'a str,
    pub(crate) fr: &'a str,
    pub(crate) de: &'a str,
}

impl Rotulo<'static> {
    /// A caption in Spanish and its English, French and German wordings.
    pub(crate) const fn nuevo(
        es: &'static str,
        (en, fr, de): (&'static str, &'static str, &'static str),
    ) -> Self {
        Self { es, en, fr, de }
    }
}

impl<'a> Rotulo<'a> {
    /// A caption that is the same in every language, or one measured in a single
    /// language: it matches only that text.
    pub(crate) const fn igual(texto: &'a str) -> Self {
        Self {
            es: texto,
            en: texto,
            fr: texto,
            de: texto,
        }
    }

    pub(crate) fn en_idioma(&self, idioma: Idioma) -> &'a str {
        match idioma {
            Idioma::Es => self.es,
            Idioma::En => self.en,
            Idioma::Fr => self.fr,
            Idioma::De => self.de,
        }
    }

    /// Whether `texto` is this caption in any language, exactly.
    pub(crate) fn coincide(&self, texto: &str) -> bool {
        Idioma::TODOS.iter().any(|i| self.en_idioma(*i) == texto)
    }
}

impl<'a> From<&'a str> for Rotulo<'a> {
    fn from(texto: &'a str) -> Self {
        Self::igual(texto)
    }
}

/// The distinct wordings, for an error that names what was looked for.
impl fmt::Display for Rotulo<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let textos = Idioma::TODOS.map(|i| self.en_idioma(i));
        let distintos: Vec<&str> = textos
            .iter()
            .enumerate()
            .filter(|(i, t)| !textos.iter().take(*i).any(|antes| antes == *t))
            .map(|(_, t)| *t)
            .collect();
        write!(f, "{}", distintos.join("/"))
    }
}

#[cfg(test)]
#[path = "../../../../tests/unit/adapters/gui/rotulos/mod.rs"]
mod tests;
