//! Buttons of the project window that exist only for some system types, and
//! the system window each type opens. They need a project of that type
//! loaded (`proyecto.nuevo_*` then Carga, or a recent project); observed live
//! on the PVsyst demo projects.

use crate::gui_catalogo::dominio::Funcion;

const HOST: &str = "TProjectForm";

pub(super) const FUNCIONES: &[Funcion] = &[
    Funcion::boton(
        "variante.necesidades_usuario_independiente",
        HOST,
        "Necesidades usuario",
    )
    .abre("TDomesticUseForm")
    .nota("requiere un proyecto de tipo sistema independiente abierto"),
    Funcion::boton("variante.necesidades_agua", HOST, "Necesidades de agua")
        .abre("TWaterUseForm")
        .nota("requiere un proyecto de tipo sistema de bombeo abierto"),
    Funcion::boton("variante.sistema_independiente", HOST, "Sistema")
        .abre("TSystemeBattForm")
        .nota("requiere un proyecto de tipo sistema independiente abierto"),
    Funcion::boton("variante.sistema_bombeo", HOST, "Sistema")
        .abre("TSystemPumpingForm")
        .nota("requiere un proyecto de tipo sistema de bombeo abierto"),
    Funcion::boton("variante.sistema_red_cc", HOST, "Sistema")
        .abre("TSystemegridDCForm")
        .nota("requiere un proyecto de tipo sistema de red CC abierto"),
];
