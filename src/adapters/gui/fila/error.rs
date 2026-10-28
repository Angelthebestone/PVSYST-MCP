//! Why selecting a row failed. Every variant says what PVsyst did, so the
//! caller can tell "wrong row" from "the click opened the element".

use crate::error::Error;

#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub(crate) enum ErrorFila {
    #[error("{clase} no tiene geometría de filas medida; admite: {admitidas}")]
    ClaseSinGeometria { clase: String, admitidas: String },
    #[error("La fila {indice} no existe: el control muestra {visibles} filas (0..={ultima})")]
    FueraDeRango {
        indice: usize,
        visibles: usize,
        ultima: usize,
    },
    #[error("Tras el clic no hay ninguna fila resaltada: la fila {indice} está vacía o el clic no llegó")]
    NingunaResaltada { indice: usize },
    #[error("Se pidió la fila {pedida} y PVsyst resaltó la {leida}")]
    FilaDistinta { pedida: usize, leida: usize },
    #[error("Hay varias filas resaltadas ({filas:?}): no se sabe cuál es la seleccionada")]
    Ambigua { filas: Vec<usize> },
    #[error("El clic en la fila {indice} activó el elemento: PVsyst abrió {clases}. Ciérrala con accion:cerrar y cancelar:true")]
    Activada { indice: usize, clases: String },
    #[error(
        "El clic en la fila {indice} cerró el control: PVsyst activó el elemento y cerró {clase}"
    )]
    ControlCerrado { indice: usize, clase: String },
}

impl ErrorFila {
    /// The request itself was wrong (nothing was clicked).
    fn es_de_pedido(&self) -> bool {
        matches!(
            self,
            Self::ClaseSinGeometria { .. } | Self::FueraDeRango { .. }
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
