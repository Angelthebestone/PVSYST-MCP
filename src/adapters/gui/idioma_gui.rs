//! The language the open PVsyst windows are written in, read from their own
//! captions: the buttons PVsyst draws say it whatever its configuration file
//! holds (a language changed from the menu reaches the file only on exit).
//!
//! This is PVsyst's interface language, not `PVSYST_MCP_IDIOMA`, which only
//! picks the language of the MCP's own messages.

use super::ffi::{self, Hwnd};
use super::proceso::pids_pvsyst;
use super::rotulos::IdiomaGui;
use super::ventanas::{enum_windows_de, text_of, visible};

/// The texts of window `h` and of every window inside it, hidden ones included.
fn textos_de(h: Hwnd) -> Vec<String> {
    std::iter::once(h)
        .chain(ffi::enum_children(h))
        .map(text_of)
        .filter(|t| !t.is_empty())
        .collect()
}

/// The language the captions of `ventanas` betray: all of them vote together.
fn de_ventanas(ventanas: impl IntoIterator<Item = Hwnd>) -> Option<IdiomaGui> {
    let textos: Vec<String> = ventanas.into_iter().flat_map(textos_de).collect();
    IdiomaGui::por_textos(textos.iter().map(String::as_str))
}

/// The language of the running PVsyst, from its visible windows. `None` when
/// PVsyst is not running or the captions prove nothing.
pub(crate) fn en_ejecucion() -> Option<IdiomaGui> {
    de_ventanas(
        pids_pvsyst()
            .into_iter()
            .flat_map(enum_windows_de)
            .filter(|h| visible(*h)),
    )
}

#[cfg(test)]
#[path = "../../../tests/unit/adapters/gui/idioma_gui.rs"]
mod tests;
