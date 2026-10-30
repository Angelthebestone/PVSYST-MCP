//! Coverage of the Near-shading scene editor and its object dialogs.

use super::super::dominio::{basica, no_aplica, solo_acciones, Ventana};

pub(super) const VENTANAS: &[Ventana] = &[
    no_aplica(
        "TPerspectiveForm",
        "Lienzo 3D dibujado: solo captura; los objetos se tratan con sus diálogos.",
    ),
    solo_acciones("TBuildingObjectForm", "En vivo no muestra campos de texto, listas desplegables ni casillas legibles por handle (cosecha del 2026-10-27): se maneja con sus botones y clic_texto."),
    basica("TElementaryObjectForm"),
    basica("TGroundObjectForm"),
    basica("TThinObjectsForm"),
    basica("TImportResultForm"),
    basica("TTrackerFieldForm"),
    basica("TShedFieldForm"),
    basica("TRectFieldForm"),
    basica("TPolyFieldForm"),
];
