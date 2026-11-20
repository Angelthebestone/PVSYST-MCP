//! Coverage of the Components and databases.

use super::super::dominio::{basica, solo_acciones, Ventana};

pub(super) const VENTANAS: &[Ventana] = &[
    solo_acciones("TSaveComponentForm", "Confirmación al pulsar OK en un componente: una casilla de compatibilidad y los botones «Guardar como» y «Cancelar»."),
    basica("TSaveObjPVForm"),
    basica("TPVModuleForm").con_esquema("components.panels"),
    basica("TGridInverterForm").con_esquema("components.inverters"),
    basica("TBatteryForm").con_esquema("components.batteries"),
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
