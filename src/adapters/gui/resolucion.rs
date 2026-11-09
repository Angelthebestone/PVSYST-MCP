//! Which window or control a reference («0x1A2B», a class, a title or part of
//! one) names.

use super::ffi::{self, enum_children, Hwnd};
use super::ventanas::{class_of, text_of, visible};

/// A window or control a reference may name.
pub(crate) struct Candidato {
    h: Hwnd,
    titulo: String,
    clase: String,
    visible: bool,
    es_hijo: bool,
}

impl Candidato {
    fn de(h: Hwnd, es_hijo: bool) -> Self {
        Self {
            h,
            titulo: text_of(h),
            clase: class_of(h),
            visible: visible(h),
            es_hijo,
        }
    }

    fn coincide(&self, referencia: &str, exacta: bool) -> bool {
        [&self.titulo, &self.clase]
            .iter()
            .any(|n| nombra(&n.to_lowercase(), referencia, exacta))
    }
}

/// Whether the name `n` is the reference (`exacta`) or holds it.
fn nombra(n: &str, referencia: &str, exacta: bool) -> bool {
    if exacta {
        n == referencia
    } else {
        n.contains(referencia)
    }
}

/// The candidate a reference names. A whole title or class beats a part of one
/// (the button «No» is not the form whose title says «cerca**no**s»), and a
/// top-level window beats a control of the same quality; a visible one beats a
/// hidden one only within the same quality, and a hidden one is the last resort.
pub(crate) fn elegir(candidatos: &[Candidato], referencia: &str) -> Option<Hwnd> {
    let referencia = referencia.to_lowercase();
    let mut oculto: Option<Hwnd> = None;
    // A whole name beats part of one; a top-level window beats a control of the same quality.
    for (exacta, es_hijo) in [(true, false), (true, true), (false, false), (false, true)] {
        let (visible, primero) = coincidentes(candidatos, (&referencia, exacta), es_hijo);
        if visible.is_some() {
            return visible;
        }
        oculto = oculto.or(primero);
    }
    oculto
}

/// The first visible candidate and the first of any visibility, of the top-level
/// windows or of the controls (`es_hijo`), whose name matches the reference.
fn coincidentes(
    candidatos: &[Candidato],
    (referencia, exacta): (&str, bool),
    es_hijo: bool,
) -> (Option<Hwnd>, Option<Hwnd>) {
    let mut coinciden = candidatos
        .iter()
        .filter(|c| c.es_hijo == es_hijo && c.coincide(referencia, exacta));
    let visible = coinciden.clone().find(|c| c.visible).map(|c| c.h);
    (visible, coinciden.next().map(|c| c.h))
}

/// Resolve a window reference: `0xHEX`, else class/title among the target
/// process' top windows and their controls (see [`elegir`]).
pub(crate) fn resolve(haystack: &[Hwnd], reference: &str) -> Option<Hwnd> {
    let hex_val = reference
        .strip_prefix("0x")
        .or_else(|| reference.strip_prefix("0X"))
        .and_then(|hex| isize::from_str_radix(hex, 16).ok());
    if let Some(v) = hex_val {
        if ffi::es_ventana(v) {
            return Some(v);
        }
    }
    let candidatos: Vec<Candidato> = haystack
        .iter()
        .map(|&h| Candidato::de(h, false))
        .chain(
            haystack
                .iter()
                .flat_map(|&h| enum_children(h))
                .map(|c| Candidato::de(c, true)),
        )
        .collect();
    elegir(&candidatos, reference)
}

#[cfg(test)]
#[path = "../../../tests/unit/adapters/gui/resolucion.rs"]
mod tests;
