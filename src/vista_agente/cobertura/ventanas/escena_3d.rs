//! Coverage of the Near-shading scene editor and its object dialogs.

use super::super::dominio::{basica, no_aplica, Ventana};

pub(super) const VENTANAS: &[Ventana] = &[
    no_aplica(
        "TPerspectiveForm",
        "Lienzo 3D dibujado: solo captura; los objetos se tratan con sus diálogos.",
    ),
    basica("TBuildingObjectForm"),
    basica("TElementaryObjectForm"),
    basica("TGroundObjectForm"),
    basica("TThinObjectsForm"),
];
