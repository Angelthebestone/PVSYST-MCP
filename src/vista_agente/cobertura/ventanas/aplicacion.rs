//! Coverage of the Application windows.

use super::super::dominio::{no_aplica, solo_acciones, Ventana};

pub(super) const VENTANAS: &[Ventana] = &[
    solo_acciones("TPreferencesForm", "En vivo no muestra campos de texto, listas desplegables ni casillas legibles por handle (cosecha del 2026-10-27): se maneja con sus botones y clic_texto."),
    solo_acciones("TToolsForm", "Menú de botones que abren las herramientas."),
    solo_acciones(
        "TWorkspaceForm",
        "Solo el botón Cerrar y la lista de espacios.",
    ),
    no_aplica("THelpForm", "Documento de ayuda dibujado: solo captura."),
    no_aplica(
        "MicrosoftWindowsTooltip",
        "Globo de ayuda de Windows, no es una ventana de PVsyst.",
    ),
];
