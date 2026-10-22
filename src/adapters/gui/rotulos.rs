//! Declarative tables of PVsyst GUI captions and window classes by language.

/// Supported languages for the PVsyst user interface.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Idioma {
    Es,
}

/// Button captions and choices for generic window and message actions.
pub(crate) struct RotulosAcciones {
    pub(crate) botones_cierre: &'static [&'static str],
    pub(crate) botones_mensaje: &'static [&'static str],
    pub(crate) botones_excepcion: &'static [&'static str],
}

pub(crate) const fn rotulos_acciones(idioma: Idioma) -> RotulosAcciones {
    match idioma {
        Idioma::Es => RotulosAcciones {
            botones_cierre: &["Cerrar", "Cancelar", "Salir", "No"],
            botones_mensaje: &["OK", "Aceptar", "Continuar", "Cerrar", "Cancelar"],
            botones_excepcion: &["Continuar"],
        },
    }
}

/// Button captions and prompt fragments for crash recovery dialogs.
pub(crate) struct RotulosRecuperacion {
    pub(crate) boton_reiniciar: &'static str,
    pub(crate) boton_salir: &'static str,
    pub(crate) aviso_inestable: &'static [&'static str],
}

pub(crate) const fn rotulos_recuperacion(idioma: Idioma) -> RotulosRecuperacion {
    match idioma {
        Idioma::Es => RotulosRecuperacion {
            boton_reiniciar: "Reiniciar PVsyst",
            boton_salir: "No",
            aviso_inestable: &["inestable", "unstable"],
        },
    }
}

/// Button captions and dismiss choices for PVsyst session forms.
pub(crate) struct RotulosSesion {
    pub(crate) cerrar: &'static str,
    pub(crate) salidas: &'static [&'static str],
    pub(crate) botones_mensaje: &'static [&'static str],
}

pub(crate) const fn rotulos_sesion(idioma: Idioma) -> RotulosSesion {
    match idioma {
        Idioma::Es => RotulosSesion {
            cerrar: "Cerrar",
            salidas: &["Cancelar", "Cerrar", "Salida", "No"],
            botones_mensaje: &["OK", "Sí", "Aceptar", "Continuar", "Cerrar"],
        },
    }
}
