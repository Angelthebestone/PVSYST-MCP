//! Coverage of the Weather and measured-data windows.

use super::super::dominio::{basica, solo_acciones, Ventana};

pub(super) const VENTANAS: &[Ventana] = &[
    basica("TMeteoImportForm"),
    basica("TMeteoConversionForm"),
    basica("TMeteoCompareForm"),
    basica("TMeteoMensForm"),
    basica("TMeteoTablesGraphsForm"),
    basica("TDatamesFusionForm"),
    solo_acciones(
        "TMetWizard",
        "Asistente por pasos: se maneja con sus botones.",
    ),
    solo_acciones("TDamMainForm", "Menú de botones de datos medidos."),
    basica("TTableFieldForm"),
    basica("TParamEditForm"),
];
