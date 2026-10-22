//! Menus of the 3D shading scene editor (`TPerspectiveForm`, reached with the
//! "Construcción / Perspectiva" button of the near-shadings form), as (menu,
//! item) positions: its command ids change with the instance. Only the items
//! that open a dialog are listed; the others (selection modes, views, edits,
//! transformations) change the scene in memory and are described in
//! `docs/gui_mapa.json` under `menus`. Observed live on PVsyst 8.1.7.

use crate::gui_catalogo::dominio::{Efecto, Funcion};

const EDITOR: &str = "TPerspectiveForm";
const ABRE_EL_EDITOR: &str = "variante.sombreados_cercanos.construccion_perspectiva";

pub(super) const FUNCIONES: &[Funcion] = &[
    Funcion::posicion(
        "escena_3d.archivo.leer_escena",
        "Archivo > Leer escena",
        (0, 1),
    )
    .en(EDITOR)
    .requiere(ABRE_EL_EDITOR)
    .abre("TChoixObjetPVForm"),
    Funcion::posicion(
        "escena_3d.archivo.leer_edificio",
        "Archivo > Leer edificio",
        (0, 2),
    )
    .en(EDITOR)
    .requiere(ABRE_EL_EDITOR)
    .abre("TChoixObjetPVForm"),
    Funcion::posicion(
        "escena_3d.archivo.leer_objetos",
        "Archivo > Leer objetos",
        (0, 3),
    )
    .en(EDITOR)
    .requiere(ABRE_EL_EDITOR)
    .abre("TChoixObjetPVForm"),
    Funcion::posicion(
        "escena_3d.archivo.exportar_escena",
        "Archivo > Exportar escena (.SHD)",
        (0, 5),
    )
    .en(EDITOR)
    .requiere(ABRE_EL_EDITOR)
    .con_efecto(Efecto::Modifica)
    .nota("no explorado: escribe un archivo .SHD"),
    Funcion::posicion("escena_3d.archivo.cancelar", "Archivo > Cancelar", (0, 11))
        .en(EDITOR)
        .requiere(ABRE_EL_EDITOR)
        .con_efecto(Efecto::Modifica)
        .nota("no explorado: cierra el editor descartando la escena"),
    Funcion::posicion(
        "escena_3d.archivo.cerrar_escena",
        "Archivo > Cerrar escena",
        (0, 12),
    )
    .en(EDITOR)
    .requiere(ABRE_EL_EDITOR)
    .con_efecto(Efecto::Modifica)
    .nota("no explorado: cierra el editor conservando la escena en la variante"),
    Funcion::posicion(
        "escena_3d.crear.objeto_elemental",
        "Crear > Objeto de sombreado elemental",
        (1, 0),
    )
    .en(EDITOR)
    .requiere(ABRE_EL_EDITOR)
    .abre("TElementaryObjectForm"),
    Funcion::posicion(
        "escena_3d.crear.edificio",
        "Crear > Edificio/Objeto Compuesto",
        (1, 1),
    )
    .en(EDITOR)
    .requiere(ABRE_EL_EDITOR)
    .abre("TBuildingObjectForm"),
    Funcion::posicion(
        "escena_3d.crear.objeto_de_tierra",
        "Crear > Objeto de tierra",
        (1, 2),
    )
    .en(EDITOR)
    .requiere(ABRE_EL_EDITOR)
    .abre("TGroundObjectForm"),
    Funcion::posicion(
        "escena_3d.crear.generador_fv",
        "Crear > Generador FV individual",
        (1, 4),
    )
    .en(EDITOR)
    .requiere(ABRE_EL_EDITOR)
    .abre("TTableFieldForm")
    .nota(
        "deja el editor en un diálogo modal: al cerrarlo pregunta si mantener las modificaciones",
    ),
    Funcion::posicion(
        "escena_3d.herramientas.herramienta_educativa_orientacion",
        "Herramientas > Herramienta educativa para la comprensión de la orientación",
        (6, 2),
    )
    .en(EDITOR)
    .requiere(ABRE_EL_EDITOR)
    .abre("TOrientToolForm"),
    Funcion::posicion(
        "escena_3d.herramientas.sombreado_objetos_delgados",
        "Herramientas > Análisis de sombreado de objetos delgados",
        (6, 3),
    )
    .en(EDITOR)
    .requiere(ABRE_EL_EDITOR)
    .abre("TThinObjectsForm"),
    Funcion::posicion(
        "escena_3d.herramientas.resumen_de_escena",
        "Herramientas > Resumen de escena de sombreado",
        (6, 6),
    )
    .en(EDITOR)
    .requiere(ABRE_EL_EDITOR)
    .abre("TSummaryForm"),
    Funcion::posicion(
        "escena_3d.herramientas.lista_de_objetos",
        "Herramientas > Lista y gestión de objetos",
        (6, 7),
    )
    .en(EDITOR)
    .requiere(ABRE_EL_EDITOR)
    .abre("TShdObjectSelectionForm"),
    Funcion::posicion(
        "escena_3d.herramientas.gestion_de_orientaciones",
        "Herramientas > Gestión de orientaciones",
        (6, 9),
    )
    .en(EDITOR)
    .requiere(ABRE_EL_EDITOR)
    .abre("TOrientManageForm"),
    Funcion::posicion(
        "escena_3d.herramientas.gestion_de_retroceso",
        "Herramientas > Gestión de retroceso",
        (6, 10),
    )
    .en(EDITOR)
    .requiere(ABRE_EL_EDITOR)
    .abre("TBacktrackingForm"),
    Funcion::posicion(
        "escena_3d.herramientas.zonas_de_tabla",
        "Herramientas > Zonas de tabla",
        (6, 11),
    )
    .en(EDITOR)
    .requiere(ABRE_EL_EDITOR)
    .nota("no abrió ventana en la escena vacía probada"),
    Funcion::posicion(
        "escena_3d.herramientas.verificar_escena",
        "Herramientas > Verificar validez de escena",
        (6, 13),
    )
    .en(EDITOR)
    .requiere(ABRE_EL_EDITOR)
    .abre("TMessageForm"),
    Funcion::posicion(
        "escena_3d.vista.especificar_posicion_observador",
        "Vista > Especificar posición del observador",
        (5, 9),
    )
    .en(EDITOR)
    .requiere(ABRE_EL_EDITOR)
    .nota("no abrió ventana: activa un modo del lienzo"),
    Funcion::posicion(
        "escena_3d.ayuda.paginas_de_ayuda",
        "Ayuda > Páginas de ayuda",
        (7, 0),
    )
    .en(EDITOR)
    .requiere(ABRE_EL_EDITOR)
    .abre("THelpForm"),
];
