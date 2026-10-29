//! Coverage of the Components and databases.

use super::super::dominio::{basica, solo_acciones, Ventana};

pub(super) const VENTANAS: &[Ventana] = &[
    basica("TPVModuleForm").con_esquema("components.panels"),
    basica("TGridInverterForm").con_esquema("components.inverters"),
    basica("TBatteryForm"),
    basica("TPumpForm"),
    basica("TPumpControllerForm"),
    basica("TSAControllerForm"),
    basica("TGeneratorForm"),
    basica("TContactForm"),
    solo_acciones("TGeoSiteForm", "En vivo no muestra campos de texto, listas desplegables ni casillas legibles por handle (cosecha del 2026-10-27): se maneja con sus botones y clic_texto."),
    basica("TChoixObjetPVForm"),
    solo_acciones(
        "TDatabaseForm",
        "Menú de botones que abren cada base de datos.",
    ),
];
