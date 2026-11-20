//! The steps the GUI entries suggest, and the read of the GUI that follows each one.

use super::super::{paso, Paso, DE_VENTANA, DIAGNOSTICO};
use crate::idioma::Texto;

pub(super) const VER_ENTORNO: Paso = DIAGNOSTICO;

pub(super) const VER_VENTANAS: Paso = paso(
    "gui_observar",
    Texto {
        es: "Ver las ventanas de PVsyst",
        en: "See the PVsyst windows",
    },
)
.con(&[("que", "ventanas")]);

pub(super) const VER_ESQUEMA: Paso = paso(
    "gui_observar",
    Texto {
        es: "Ver los formularios con vista de agente",
        en: "See the forms with an agent view",
    },
)
.con(&[("que", "esquema")]);

pub(super) const VER_FUNCIONES: Paso = paso(
    "gui_observar",
    Texto {
        es: "Ver las funciones de la GUI",
        en: "See the GUI functions",
    },
)
.con(&[("que", "funciones")]);

pub(super) const VER_FORMULARIO: Paso = paso(
    "gui_observar",
    Texto {
        es: "Leer el formulario abierto por claves",
        en: "Read the open form by key",
    },
)
.con(&[("que", "vista")]);

pub(super) const VER_CONTROLES: Paso = paso(
    "gui_observar",
    Texto {
        es: "Ver los controles de la ventana",
        en: "See the window controls",
    },
)
.con(&[("que", "controles")])
.tomando(DE_VENTANA);

pub(super) const CAPTURAR: Paso = paso(
    "gui_observar",
    Texto {
        es: "Capturar la ventana",
        en: "Capture the window",
    },
)
.con(&[("que", "capturar")])
.tomando(DE_VENTANA);

/// What follows a read of the GUI: the forms with a schema, or the open windows.
pub(super) const TRAS_VISTA: &[Paso] = &[VER_ESQUEMA, VER_VENTANAS];
