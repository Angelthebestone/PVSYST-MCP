//! Captions of the buttons and options the MCP's own steps press on PVsyst's
//! home, project, database and horizon windows. Spanish measured live on
//! 8.1.8; English, French and German inferred and still to be measured on an
//! install in that language. Window classes and tab indexes are not here: they
//! are the same in every language.

use super::Rotulo;

pub(crate) const INICIO_RED: Rotulo<'static> = Rotulo::nuevo(
    "Conectado a la red",
    ("Grid-connected", "Couplé au réseau", "Netzgekoppelt"),
);
pub(crate) const INICIO_AISLADO: Rotulo<'static> =
    Rotulo::nuevo("Independiente", ("Stand alone", "Autonome", "Inselanlage"));
pub(crate) const INICIO_BOMBEO: Rotulo<'static> =
    Rotulo::nuevo("Bombeo", ("Pumping", "Pompage", "Pumpen"));
pub(crate) const INICIO_RED_DC: Rotulo<'static> =
    Rotulo::nuevo("Red DC", ("DC-grid", "Réseau DC", "DC-Netz"));

pub(crate) const BASES_DE_DATOS: Rotulo<'static> = Rotulo::nuevo(
    "Bases de datos",
    ("Databases", "Bases de données", "Datenbanken"),
);
pub(crate) const MODULOS_PV: Rotulo<'static> =
    Rotulo::nuevo("Módulos PV", ("PV modules", "Modules PV", "PV-Module"));
pub(crate) const INVERSOR_DE_RED: Rotulo<'static> = Rotulo::nuevo(
    "Inversor de red",
    ("Grid inverter", "Onduleur réseau", "Netzwechselrichter"),
);

pub(crate) const SISTEMA: Rotulo<'static> =
    Rotulo::nuevo("Sistema", ("System", "Système", "System"));
pub(crate) const HORIZONTE: Rotulo<'static> =
    Rotulo::nuevo("Horizonte", ("Horizon", "Horizon", "Horizont"));
pub(crate) const GESTION_ENERGIA: Rotulo<'static> = Rotulo::nuevo(
    "Gestión de la energía",
    (
        "Energy management",
        "Gestion de l'énergie",
        "Energiemanagement",
    ),
);

pub(crate) const LEER_IMPORTAR: Rotulo<'static> = Rotulo::nuevo(
    "Leer / Importar",
    ("Read / Import", "Lire / Importer", "Lesen / Importieren"),
);
pub(crate) const OPCION_PVGIS: Rotulo<'static> =
    Rotulo::nuevo("PVGIS", ("PVGIS", "PVGIS", "PVGIS"));
pub(crate) const IMPORTAR_DESDE_WEB: Rotulo<'static> = Rotulo::nuevo(
    "Importar desde web",
    (
        "Import from web",
        "Importer depuis le web",
        "Aus dem Web importieren",
    ),
);

/// The button of the elementary-object dialog of the 3D scene that adds the object.
pub(crate) const CERRAR_OBJETO: Rotulo<'static> = Rotulo::nuevo(
    "Cerrar objeto",
    ("Close object", "Fermer l'objet", "Objekt schließen"),
);

pub(super) const TABLA: &[Rotulo<'static>] = &[
    INICIO_RED,
    INICIO_AISLADO,
    INICIO_BOMBEO,
    INICIO_RED_DC,
    BASES_DE_DATOS,
    MODULOS_PV,
    INVERSOR_DE_RED,
    SISTEMA,
    HORIZONTE,
    GESTION_ENERGIA,
    LEER_IMPORTAR,
    IMPORTAR_DESDE_WEB,
    CERRAR_OBJETO,
];
