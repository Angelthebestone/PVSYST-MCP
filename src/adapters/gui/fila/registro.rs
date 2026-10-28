//! Row geometry measured per control (PVsyst 8.1.7, 144 DPI, captures at the
//! 96-DPI layout). The row height is not global: each control has its own, and
//! a control of the same class in another window is not assumed to match.

use super::error::ErrorFila;
use super::geometria::Geometria;

/// `TVirtualStringTree` of `TProjectLoadForm`: header 21 px, rows of 19 px.
/// Measured on rows 0, 1, 3 and 18 (the last one).
const ARBOL_DE_PROYECTOS: Geometria = Geometria {
    clase: "TVirtualStringTree",
    anfitrion: "TProjectLoadForm",
    titulo_anfitrion: None,
    cabecera_px: 21,
    alto_fila_px: 19,
    x_px: 100,
};

/// `TListBox` of the module chooser (`TChoixObjetPVForm`): border 2 px, rows of
/// 13 px. The header above it is a separate `TStringGrid` one row tall: address
/// the list, not it. The other 14 choosers of that class were not measured.
const LISTA_DE_MODULOS: Geometria = Geometria {
    clase: "TListBox",
    anfitrion: "TChoixObjetPVForm",
    titulo_anfitrion: Some("Eligiendo un módulo FV"),
    cabecera_px: 2,
    alto_fila_px: 13,
    x_px: 100,
};

const TABLAS: &[Geometria] = &[ARBOL_DE_PROYECTOS, LISTA_DE_MODULOS];

/// A control as found on screen: its class and the window that hosts it.
#[derive(Debug, Clone, Copy)]
pub(crate) struct Control<'a> {
    pub(crate) clase: &'a str,
    pub(crate) anfitrion: &'a str,
    pub(crate) titulo: &'a str,
}

impl Geometria {
    fn corresponde_a(&self, control: &Control<'_>) -> bool {
        self.clase == control.clase
            && self.anfitrion == control.anfitrion
            && self
                .titulo_anfitrion
                .is_none_or(|parte| control.titulo.contains(parte))
    }
}

/// Declared geometry of `control`, only when that very control was measured.
pub(crate) fn geometria_de(control: &Control<'_>) -> Result<&'static Geometria, ErrorFila> {
    TABLAS
        .iter()
        .find(|g| g.corresponde_a(control))
        .ok_or_else(|| ErrorFila::ClaseSinGeometria {
            clase: control.clase.to_string(),
            anfitrion: control.anfitrion.to_string(),
            medidas: medidas(),
        })
}

fn medidas() -> String {
    TABLAS
        .iter()
        .map(|g| format!("{} de {}", g.clase, g.anfitrion))
        .collect::<Vec<_>>()
        .join(", ")
}

#[cfg(test)]
#[path = "../../../../tests/unit/adapters/gui/fila/registro.rs"]
mod tests;
