//! Why selecting a row failed. Every variant says what PVsyst did, so the
//! caller can tell "wrong row" from "the click opened the element".

use crate::error::Error;

#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub(crate) enum ErrorFila {
    #[error("{clase} de {anfitrion} no tiene geometría de filas medida; medida en: {medidas}")]
    ClaseSinGeometria {
        clase: String,
        anfitrion: String,
        medidas: String,
    },
    #[error("El control no muestra ninguna fila: mide {alto_px} px de alto a 96 DPI")]
    SinFilasVisibles { alto_px: i32 },
    #[error(
        "El control muestra {visibles} filas y la banda no se distingue con menos de {minimas}"
    )]
    PocasFilas { visibles: usize, minimas: usize },
    #[error("La fila {indice} no existe: el control muestra {visibles} filas (0..={ultima})")]
    FueraDeRango {
        indice: usize,
        visibles: usize,
        ultima: usize,
    },
    #[error("El punto de clic (x {x_px} px) cae fuera del control, que mide {ancho_px} px de ancho a 96 DPI")]
    FueraDelControl { x_px: i32, ancho_px: i32 },
    #[error(
        "No se pudo traer {clase} al primer plano y darle el foco: el clic iría a otra ventana"
    )]
    SinFoco { clase: String },
    #[error("Tras el clic no hay ninguna fila resaltada: la fila {indice} está vacía o el clic no llegó")]
    NingunaResaltada { indice: usize },
    #[error("Se pidió la fila {pedida} y PVsyst resaltó la {leida}")]
    FilaDistinta { pedida: usize, leida: usize },
    #[error("Hay varias filas resaltadas ({filas:?}): no se sabe cuál es la seleccionada")]
    Ambigua { filas: Vec<usize> },
    #[error("El clic en la fila {indice} activó el elemento: PVsyst abrió {clases}. Ciérrala con accion:cerrar y cancelar:true")]
    Activada { indice: usize, clases: String },
    #[error("El clic en la fila {indice} cerró {clases}: PVsyst lo trató como una acción, no como una selección")]
    VentanaCerrada { indice: usize, clases: String },
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
