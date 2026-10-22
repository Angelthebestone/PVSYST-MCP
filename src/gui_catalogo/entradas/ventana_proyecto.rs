//! Menu bar of the project window (`TProjectForm`): Proyecto, Sitio, Variante
//! and Notas del usuario, as (menu, item) positions counted from 0 with
//! separators included. Its command ids are not stable (they moved by three
//! between two runs), so they are read from the open popup at call time.
//! Invoke them while no other dialog is open: with a modal dialog up PVsyst
//! raised an internal exception, and the catalogue refuses to.

use crate::gui_catalogo::dominio::{Efecto, Funcion};

const VENTANA: &str = "TProjectForm";

pub(super) const FUNCIONES: &[Funcion] = &[
    Funcion::posicion(
        "ventana_proyecto.proyecto.nuevo",
        "Proyecto > Crear un nuevo proyecto",
        (0, 0),
    )
    .en(VENTANA),
    Funcion::posicion(
        "ventana_proyecto.proyecto.cargar",
        "Proyecto > Cargar proyecto",
        (0, 1),
    )
    .en(VENTANA),
    Funcion::posicion(
        "ventana_proyecto.proyecto.guardar",
        "Proyecto > Guardar el proyecto activo",
        (0, 2),
    )
    .en(VENTANA)
    .con_efecto(Efecto::Modifica),
    Funcion::posicion(
        "ventana_proyecto.proyecto.eliminar",
        "Proyecto > Eliminar proyecto activo",
        (0, 3),
    )
    .en(VENTANA)
    .con_efecto(Efecto::Modifica),
    Funcion::posicion(
        "ventana_proyecto.proyecto.importar",
        "Proyecto > Importar un proyecto",
        (0, 5),
    )
    .en(VENTANA)
    .abre("#32770")
    .con_efecto(Efecto::Modifica),
    Funcion::posicion(
        "ventana_proyecto.proyecto.exportar",
        "Proyecto > Exportar el proyecto",
        (0, 6),
    )
    .en(VENTANA)
    .con_efecto(Efecto::Modifica),
    Funcion::posicion(
        "ventana_proyecto.proyecto.configuracion",
        "Proyecto > Configuración del proyecto",
        (0, 8),
    )
    .en(VENTANA)
    .abre("TSettingsForm"),
    Funcion::posicion(
        "ventana_proyecto.proyecto.ayuda",
        "Proyecto > Ayuda sobre el proyecto",
        (0, 10),
    )
    .en(VENTANA),
    Funcion::posicion(
        "ventana_proyecto.proyecto.salida",
        "Proyecto > Salida",
        (0, 12),
    )
    .en(VENTANA)
    .con_efecto(Efecto::Cierra),
    Funcion::posicion(
        "ventana_proyecto.sitio.nuevo",
        "Sitio > Nuevo sitio",
        (1, 0),
    )
    .en(VENTANA)
    .abre("TGeoSiteForm"),
    Funcion::posicion(
        "ventana_proyecto.sitio.abrir",
        "Sitio > Abrir un sitio",
        (1, 1),
    )
    .en(VENTANA)
    .abre("TChoixObjetPVForm"),
    Funcion::posicion(
        "ventana_proyecto.sitio.detalles",
        "Sitio > Ver los detalles del sitio",
        (1, 2),
    )
    .en(VENTANA)
    .abre("TGeoSiteForm"),
    Funcion::posicion(
        "ventana_proyecto.variante.crear",
        "Variante > Crear una nueva variante",
        (2, 0),
    )
    .en(VENTANA)
    .con_efecto(Efecto::Modifica),
    Funcion::posicion(
        "ventana_proyecto.variante.guardar",
        "Variante > Guardar la variante activa",
        (2, 1),
    )
    .en(VENTANA)
    .con_efecto(Efecto::Modifica),
    Funcion::posicion(
        "ventana_proyecto.variante.eliminar",
        "Variante > Eliminar variante activa",
        (2, 2),
    )
    .en(VENTANA)
    .con_efecto(Efecto::Modifica),
    Funcion::posicion(
        "ventana_proyecto.variante.importar",
        "Variante > Importar variante de otro proyecto",
        (2, 4),
    )
    .en(VENTANA)
    .con_efecto(Efecto::Modifica),
    Funcion::posicion(
        "ventana_proyecto.variante.gestionar",
        "Variante > Gestionar variantes",
        (2, 5),
    )
    .en(VENTANA)
    .abre("TVCManageForm"),
    Funcion::posicion(
        "ventana_proyecto.variante.ayuda",
        "Variante > Ayuda sobre las variantes",
        (2, 7),
    )
    .en(VENTANA),
    Funcion::posicion(
        "ventana_proyecto.notas.proyecto",
        "Notas del usuario > Notas del proyecto",
        (3, 0),
    )
    .en(VENTANA)
    .abre("TSummaryForm"),
    Funcion::posicion(
        "ventana_proyecto.notas.variante",
        "Notas del usuario > Notas de variante",
        (3, 1),
    )
    .en(VENTANA)
    .abre("TSummaryForm"),
];
