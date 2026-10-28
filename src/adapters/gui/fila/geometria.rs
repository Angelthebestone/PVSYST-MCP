//! Where the rows of a list or tree are. Their controls have no child window
//! per row (UIA and MSAA expose nothing), so the geometry is measured once per
//! control class and declared as data; one interpreter turns a row into a point.

use super::error::ErrorFila;

/// Row layout of one control class, in px at 96 DPI from the control's top-left.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct Geometria {
    pub(crate) clase: &'static str,
    /// Top of row 0: header and border above the first row.
    pub(crate) cabecera_px: i32,
    pub(crate) alto_fila_px: i32,
    /// Horizontal point where the click lands; a point without icons or buttons.
    pub(crate) x_px: i32,
}

impl Geometria {
    /// Row slots that fit in a control `alto_control_px` tall. O(1).
    pub(crate) fn filas_visibles(&self, alto_control_px: i32) -> usize {
        let libre = (alto_control_px - self.cabecera_px).max(0);
        usize::try_from(libre / self.alto_fila_px.max(1)).unwrap_or(0)
    }

    /// Vertical centre of row slot `indice`. O(1).
    pub(crate) fn centro_de_fila(&self, indice: usize) -> i32 {
        let fila = i32::try_from(indice).unwrap_or(i32::MAX);
        fila.saturating_mul(self.alto_fila_px)
            .saturating_add(self.cabecera_px + self.alto_fila_px / 2)
    }

    /// `dy` of the click on row `indice`, or why that row cannot be addressed.
    pub(crate) fn dy_de_fila(&self, indice: usize, alto_control_px: i32) -> Result<i32, ErrorFila> {
        let visibles = self.filas_visibles(alto_control_px);
        if indice >= visibles {
            return Err(ErrorFila::FueraDeRango {
                indice,
                visibles,
                ultima: visibles.saturating_sub(1),
            });
        }
        Ok(self.centro_de_fila(indice))
    }
}

#[cfg(test)]
#[path = "../../../../tests/unit/adapters/gui/fila/geometria.rs"]
mod tests;
