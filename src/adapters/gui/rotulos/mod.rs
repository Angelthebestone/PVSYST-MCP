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
pub(crate) enum IdiomaGui {
    Es,
    En,
    Fr,
    De,
}

impl IdiomaGui {
    pub(crate) const TODOS: [Self; 4] = [Self::Es, Self::En, Self::Fr, Self::De];
    /// The languages whose captions were measured on a live PVsyst.
    pub(crate) const MEDIDOS: [Self; 1] = [Self::Es];

    pub(crate) const fn codigo(self) -> &'static str {
        match self {
            Self::Es => "es",
            Self::En => "en",
            Self::Fr => "fr",
            Self::De => "de",
        }
    }

    /// The language a `PVsyst.ini` `Dialog Language` value names (`Spanish`,
    /// `English`...); `None` for a language the captions do not cover.
    pub(crate) fn de_nombre(nombre: &str) -> Option<Self> {
        match nombre.trim().to_lowercase().as_str() {
            "spanish" | "español" | "espanol" | "es" => Some(Self::Es),
            "english" | "en" => Some(Self::En),
            "french" | "français" | "francais" | "fr" => Some(Self::Fr),
            "german" | "deutsch" | "de" => Some(Self::De),
            _ => None,
        }
    }

    /// The language the texts of a window betray: each text equal to a known
    /// caption of exactly one language votes for it, and a clear winner is the
    /// answer. A caption that reads the same in several languages ("OK") votes
    /// for none. `None` on a tie or without votes.
    pub(crate) fn por_textos<'a>(textos: impl IntoIterator<Item = &'a str>) -> Option<Self> {
        let votos: Vec<Self> = textos.into_iter().filter_map(idioma_unico).collect();
        let mut conteo: Vec<(Self, usize)> = Self::TODOS
            .into_iter()
            .map(|i| (i, votos.iter().filter(|v| **v == i).count()))
            .collect();
        conteo.sort_by_key(|(_, n)| std::cmp::Reverse(*n));
        match conteo.as_slice() {
            [(i, n), (_, siguiente), ..] if n > siguiente => Some(*i),
            _ => None,
        }
    }
}

/// The only language in which `texto` is a known caption.
fn idioma_unico(texto: &str) -> Option<IdiomaGui> {
    let mut idiomas = IdiomaGui::TODOS
        .into_iter()
        .filter(|i| todos().any(|r| r.en_idioma(*i) == texto));
    match (idiomas.next(), idiomas.next()) {
        (Some(i), None) => Some(i),
        _ => None,
    }
}

/// Every caption the tables know.
fn todos() -> impl Iterator<Item = &'static Rotulo<'static>> {
    dialogos::TABLA.iter().chain(pvsyst::TABLA)
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

    pub(crate) fn en_idioma(&self, idioma: IdiomaGui) -> &'a str {
        match idioma {
            IdiomaGui::Es => self.es,
            IdiomaGui::En => self.en,
            IdiomaGui::Fr => self.fr,
            IdiomaGui::De => self.de,
        }
    }

    /// The wordings by language code when the caption has more than one, else `None`.
    pub(crate) fn por_idioma(&self) -> Option<serde_json::Value> {
        let traducido = IdiomaGui::TODOS
            .iter()
            .any(|i| self.en_idioma(*i) != self.es);
        traducido.then(|| {
            let pares = IdiomaGui::TODOS.map(|i| (i.codigo(), self.en_idioma(i)));
            serde_json::json!(pares
                .into_iter()
                .collect::<std::collections::BTreeMap<_, _>>())
        })
    }

    /// Whether `texto` is this caption in any language, exactly.
    pub(crate) fn coincide(&self, texto: &str) -> bool {
        IdiomaGui::TODOS.iter().any(|i| self.en_idioma(*i) == texto)
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
        let textos = IdiomaGui::TODOS.map(|i| self.en_idioma(i));
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
