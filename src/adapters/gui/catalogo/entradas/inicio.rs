//! Main form (`THomeForm`): menu bar and start-page buttons.
//!
//! Menu command ids were read from the open popups of PVsyst 8.1.7. Ids not
//! listed (Licencia 74, Ayuda 76-98, Idioma 130-141, and the recent workspaces
//! and projects, which follow the user's history) are reachable with
//! `gui_actuar accion:menu`.

use crate::adapters::gui::catalogo::dominio::{Efecto, Funcion, VENTANA_INICIO};
use crate::adapters::gui::rotulos::pvsyst::{
    BASES_DE_DATOS, INICIO_AISLADO, INICIO_BOMBEO, INICIO_RED,
};

const DISENO: &str = "TDimensForm";
const PROYECTO: &str = "TProjectForm";

pub(super) const FUNCIONES: &[Funcion] = &[
    Funcion::menu(
        "archivo.espacio_de_trabajo",
        "Archivo > Espacio de trabajo",
        23,
    )
    .abre("TWorkspaceForm"),
    Funcion::menu(
        "archivo.borrar_espacios_recientes",
        "Archivo > Espacios de trabajo recientes > Borrar la lista",
        36,
    )
    .con_efecto(Efecto::Modifica),
    Funcion::menu(
        "archivo.actualizar_bases_de_datos",
        "Archivo > Actualizar bases de datos",
        37,
    )
    .con_efecto(Efecto::Modifica),
    Funcion::menu(
        "archivo.importar_componentes_carpeta",
        "Archivo > Importar componentes > desde carpeta",
        40,
    )
    .con_efecto(Efecto::Modifica),
    Funcion::menu(
        "archivo.importar_componentes_zip",
        "Archivo > Importar componentes > desde ZIP",
        41,
    )
    .con_efecto(Efecto::Modifica),
    Funcion::menu(
        "archivo.exportar_componentes",
        "Archivo > Exportar componentes",
        42,
    )
    .con_efecto(Efecto::Modifica),
    Funcion::menu(
        "archivo.importar_proyectos",
        "Archivo > Importar proyectos",
        44,
    )
    .con_efecto(Efecto::Modifica),
    Funcion::menu(
        "archivo.exportar_proyectos",
        "Archivo > Exportar proyectos",
        45,
    )
    .con_efecto(Efecto::Modifica),
    Funcion::menu(
        "archivo.borrar_proyectos_recientes",
        "Archivo > Proyectos recientes > Borrar la lista",
        54,
    )
    .con_efecto(Efecto::Modifica),
    Funcion::menu(
        "archivo.exportar_registros",
        "Archivo > Exportar registros",
        56,
    )
    .con_efecto(Efecto::Modifica),
    Funcion::menu("archivo.salida", "Archivo > Salida", 58).con_efecto(Efecto::Cierra),
    Funcion::menu(
        "diseno_preliminar.conectado_a_la_red",
        "Diseño preliminar > Conectado a la red",
        60,
    )
    .abre(DISENO),
    Funcion::menu(
        "diseno_preliminar.independiente",
        "Diseño preliminar > Independiente",
        61,
    )
    .abre(DISENO),
    Funcion::menu("diseno_preliminar.bombeo", "Diseño preliminar > Bombeo", 62).abre(DISENO),
    Funcion::menu(
        "proyecto.nuevo_conectado_a_la_red",
        "Proyecto > Nuevo proyecto conectado a la red",
        64,
    )
    .abre(PROYECTO),
    Funcion::menu(
        "proyecto.nuevo_independiente",
        "Proyecto > Nuevo proyecto independiente",
        65,
    )
    .abre(PROYECTO),
    Funcion::menu(
        "proyecto.nuevo_bombeo",
        "Proyecto > Nuevo proyecto de bombeo",
        66,
    )
    .abre(PROYECTO),
    Funcion::menu(
        "proyecto.nuevo_red_cc",
        "Proyecto > Nuevo proyecto Red-CC",
        67,
    )
    .abre(PROYECTO),
    Funcion::menu("proyecto.cargar", "Proyecto > Cargar proyecto", 68).abre("TProjectLoadForm"),
    Funcion::menu(
        "configuraciones.preferencias",
        "Configuraciones > Preferencias",
        70,
    )
    .abre("TPreferencesForm"),
    Funcion::menu(
        "configuraciones.parametros_avanzados",
        "Configuraciones > Editar parámetros avanzados",
        71,
    )
    .abre("TParamEditForm"),
    Funcion::boton_en_idiomas("inicio.conectado_a_la_red", VENTANA_INICIO, INICIO_RED),
    Funcion::boton_en_idiomas("inicio.independiente", VENTANA_INICIO, INICIO_AISLADO),
    Funcion::boton_en_idiomas("inicio.bombeo", VENTANA_INICIO, INICIO_BOMBEO),
    Funcion::boton_en_idiomas("inicio.bases_de_datos", VENTANA_INICIO, BASES_DE_DATOS)
        .abre("TDatabaseForm"),
    Funcion::boton("inicio.herramientas", VENTANA_INICIO, "Herramientas").abre("TToolsForm"),
    Funcion::boton("inicio.datos_medidos", VENTANA_INICIO, "Datos medidos").abre("TDamMainForm"),
    Funcion::boton("inicio.administrar_espacio", VENTANA_INICIO, "Administrar")
        .abre("TWorkspaceForm"),
    Funcion::boton("inicio.cambiar_espacio", VENTANA_INICIO, "Cambiar"),
];
