//! Coverage of the Components and databases.

use super::super::dominio::{basica, solo_acciones, Ventana};

pub(super) const VENTANAS: &[Ventana] = &[
    basica("TPVModuleForm").con_esquema("components.panels"),
    basica("TGridInverterForm"),
    basica("TBatteryForm"),
    basica("TPumpForm"),
    basica("TPumpControllerForm"),
    basica("TSAControllerForm"),
    basica("TGeneratorForm"),
    basica("TContactForm"),
    basica("TGeoSiteForm"),
    basica("TChoixObjetPVForm"),
    solo_acciones(
        "TDatabaseForm",
        "Menú de botones que abren cada base de datos.",
    ),
];
