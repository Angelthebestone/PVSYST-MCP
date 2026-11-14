//! Why selecting a row failed. Every variant says what PVsyst did, so the
//! caller can tell "wrong row" from "the click opened the element".

use crate::error::Error;

#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub(crate) enum ErrorFila {
    #[error("{clase} of {anfitrion} has no measured row geometry; measured on: {medidas}")]
    ClaseSinGeometria {
        clase: String,
        anfitrion: String,
        medidas: String,
    },
    #[error("The control shows no row: it is {alto_px} px high at 96 DPI")]
    SinFilasVisibles { alto_px: i32 },
    #[error(
        "The control shows {visibles} rows and the band cannot be told apart with fewer than {minimas}"
    )]
    PocasFilas { visibles: usize, minimas: usize },
    #[error("Row {indice} does not exist: the control shows {visibles} rows (0..={ultima})")]
    FueraDeRango {
        indice: usize,
        visibles: usize,
        ultima: usize,
    },
    #[error("The click point (x {x_px} px) is outside the control, which is {ancho_px} px wide at 96 DPI")]
    FueraDelControl { x_px: i32, ancho_px: i32 },
    #[error(
        "Could not bring {clase} to the front and give it the focus: the click would go to another window. Bring the window to the front with accion:activar and retry"
    )]
    SinFoco { clase: String },
    #[error(
        "No row is highlighted after the click: row {indice} is empty or the click did not arrive"
    )]
    NingunaResaltada { indice: usize },
    #[error("Row {pedida} was requested and PVsyst highlighted {leida}")]
    FilaDistinta { pedida: usize, leida: usize },
    #[error("Row {indice} is highlighted but its text could not be read to check it: {motivo}")]
    SinLectura { indice: usize, motivo: String },
    #[error("Row {indice} is highlighted but it holds \"{leido}\", not \"{pedido}\": the row selected is not the requested element")]
    TextoDistinto {
        indice: usize,
        pedido: String,
        leido: String,
    },
    #[error("Several rows are highlighted ({filas:?}): the selected one is unknown")]
    Ambigua { filas: Vec<usize> },
    #[error("The click on row {indice} activated the element: PVsyst opened {clases}. Close it with accion:cerrar and cancelar:true")]
    Activada { indice: usize, clases: String },
    #[error("The click on row {indice} closed {clases}: PVsyst took it as an action, not as a selection")]
    VentanaCerrada { indice: usize, clases: String },
    #[error(
        "The click on row {indice} closed the control: PVsyst activated the element and closed {clase}"
    )]
    ControlCerrado { indice: usize, clase: String },
}

impl ErrorFila {
    /// The request itself was wrong (nothing was clicked).
    fn es_de_pedido(&self) -> bool {
        matches!(
            self,
            Self::ClaseSinGeometria { .. }
                | Self::SinFilasVisibles { .. }
                | Self::PocasFilas { .. }
                | Self::FueraDeRango { .. }
                | Self::FueraDelControl { .. }
        )
    }

    /// The error the tool returns. Not a `From`: a blanket conversion into
    /// `Error` would make every `?` in the crate ambiguous to infer.
    pub(crate) fn a_error(self) -> Error {
        if self.es_de_pedido() {
            Error::Invalid(self.to_string())
        } else {
            Error::Gui(self.to_string())
        }
    }
}
