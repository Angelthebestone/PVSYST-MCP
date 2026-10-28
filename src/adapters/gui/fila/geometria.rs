//! Where the rows of a list or tree are. Their controls have no child window
//! per row (UIA and MSAA expose nothing), so the geometry is measured once per
//! control and declared as data; one interpreter turns a row into a point.

use super::error::ErrorFila;

/// Fewest row slots in which a selection band can be told from the background.
pub(super) const FILAS_MINIMAS: usize = 3;

/// Row layout of one control, in px at 96 DPI from the control's top-left. It
/// holds for the control as measured: the class and the window that hosts it.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct Geometria {
    pub(crate) clase: &'static str,
    /// Class of the top-level window the control was measured in.
    pub(crate) anfitrion: &'static str,
    /// Part of that window's title, when its class has several windows.
    pub(crate) titulo_anfitrion: Option<&'static str>,
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
        if visibles == 0 {
            return Err(ErrorFila::SinFilasVisibles {
                alto_px: alto_control_px,
            });
        }
        if visibles < FILAS_MINIMAS {
            return Err(ErrorFila::PocasFilas {
                visibles,
                minimas: FILAS_MINIMAS,
            });
        }
        if indice >= visibles {
            return Err(ErrorFila::FueraDeRango {
                indice,
                visibles,
                ultima: visibles - 1,
            });
        }
        Ok(self.centro_de_fila(indice))
    }

    /// `(x, dy)` of the click on row `indice` of a control `ancho` x `alto` px
    /// (96 DPI): the point must lie inside it, or the click would hit another window.
    pub(crate) fn punto_de_fila(
        &self,
        indice: usize,
        (ancho_px, alto_px): (i32, i32),
    ) -> Result<(i32, i32), ErrorFila> {
        let dy = self.dy_de_fila(indice, alto_px)?;
        if self.x_px >= ancho_px {
            return Err(ErrorFila::FueraDelControl {
                x_px: self.x_px,
                ancho_px,
            });
        }
        Ok((self.x_px, dy))
    }
}

#[cfg(test)]
#[path = "../../../../tests/unit/adapters/gui/fila/geometria.rs"]
mod tests;
