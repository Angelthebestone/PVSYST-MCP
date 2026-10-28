//! Coverage of the Results and plot windows.

use super::super::dominio::{basica, no_aplica, solo_acciones, Ventana};

pub(super) const VENTANAS: &[Ventana] = &[
    basica("TResultForm"),
    basica("TResultTabForm"),
    solo_acciones(
        "TResultPlotForm",
        "Lista de gráficos sin campos de texto: se maneja con sus botones y clic_texto.",
    ),
    basica("TResultPlotHorForm"),
    basica("TSimulPlotForm"),
    solo_acciones(
        "TSimulValHorForm",
        "Lista de variables sin controles de campo; solo Aceptar/Cancelar.",
    ),
    basica("TArrayLossesPlotForm"),
    no_aplica(
        "TPlotLossDiagramF",
        "Diagrama de pérdidas dibujado: solo captura.",
    ),
];
