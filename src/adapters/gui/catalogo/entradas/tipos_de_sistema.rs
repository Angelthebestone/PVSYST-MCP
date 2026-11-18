//! Buttons of the project window that exist only for some system types, and
//! the system window each type opens. They need a project of that type
//! loaded (`proyecto.nuevo_*` then Carga, or a recent project); observed live
//! on the PVsyst demo projects.
//!
//! The window «Necesidades usuario» opens depends on the project: measured on
//! 8.1.8, the demo `_DEMO_StandAlone` opens `TDomesticUseForm`, while a
//! stand-alone project created with `disenar_sistema tipo_sistema:aislado`
//! opens `TUserNeedsForm`. The entry declares the demo one.

use crate::adapters::gui::catalogo::dominio::Funcion;
use crate::adapters::gui::rotulos::pvsyst::SISTEMA;

const HOST: &str = "TProjectForm";

pub(super) const FUNCIONES: &[Funcion] = &[
    // `_DEMO_StandAlone` opens `TDomesticUseForm` here and a synthetic
    // stand-alone project opens `TUserNeedsForm`, so `abre` holds for the demo.
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
    Funcion::boton_en_idiomas("variante.sistema_independiente", HOST, SISTEMA)
        .abre("TSystemeBattForm")
        .nota("requiere un proyecto de tipo sistema independiente abierto"),
    Funcion::boton_en_idiomas("variante.sistema_bombeo", HOST, SISTEMA)
        .abre("TSystemPumpingForm")
        .nota("requiere un proyecto de tipo sistema de bombeo abierto"),
    Funcion::boton_en_idiomas("variante.sistema_red_cc", HOST, SISTEMA)
        .abre("TSystemegridDCForm")
        .nota("requiere un proyecto de tipo sistema de red CC abierto"),
];
