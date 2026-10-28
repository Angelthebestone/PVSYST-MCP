//! Row geometry measured per control class (PVsyst 8.1.7, 144 DPI, captures
//! at the 96-DPI layout). The row height is not global: each control has its own.

use super::error::ErrorFila;
use super::geometria::Geometria;

/// `TVirtualStringTree` of `TProjectLoadForm`: header 21 px, rows of 19 px.
/// Measured on rows 0, 1, 3 and 18 (the last one).
const ARBOL_DE_PROYECTOS: Geometria = Geometria {
    clase: "TVirtualStringTree",
    cabecera_px: 21,
    alto_fila_px: 19,
    x_px: 100,
};

/// `TListBox` of `TChoixObjetPVForm`: border 2 px, rows of 13 px. The header
/// above it is a separate `TStringGrid` one row tall: address the list, not it.
const LISTA_DE_COMPONENTES: Geometria = Geometria {
    clase: "TListBox",
    cabecera_px: 2,
    alto_fila_px: 13,
    x_px: 100,
};

const TABLAS: &[Geometria] = &[ARBOL_DE_PROYECTOS, LISTA_DE_COMPONENTES];

/// Declared geometry of control class `clase`.
pub(crate) fn geometria_de(clase: &str) -> Result<&'static Geometria, ErrorFila> {
    TABLAS
        .iter()
        .find(|g| g.clase == clase)
        .ok_or_else(|| ErrorFila::ClaseSinGeometria {
            clase: clase.to_string(),
            admitidas: clases_admitidas(),
        })
}

fn clases_admitidas() -> String {
    TABLAS
        .iter()
        .map(|g| g.clase)
        .collect::<Vec<_>>()
        .join(", ")
}

#[cfg(test)]
#[path = "../../../../tests/unit/adapters/gui/fila/registro.rs"]
mod tests;
