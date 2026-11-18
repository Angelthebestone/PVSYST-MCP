//! `gui_actuar` on the 3D editor: the scene is checked against the file PVsyst saves.

use super::super::{entrada, Entrada};
use super::pasos::{TRAS_VISTA, VER_VENTANAS};
use crate::idioma::Texto;

pub(super) const ESCENA: Entrada = entrada(
    "gui_actuar",
    "escena",
    Texto {
        es: "Escena 3D: validada (simular) o creada en el editor y comparada con el archivo que PVsyst guardó.",
        en: "3D scene: validated (simular) or created in the editor and compared with the file PVsyst saved.",
    },
)
.luego(TRAS_VISTA);

pub(super) const ESCENA_LEER: Entrada = entrada(
    "gui_actuar",
    "escena_leer",
    Texto {
        es: "Objetos de la escena abierta en el editor 3D, leídos del archivo que PVsyst exportó.",
        en: "Objects of the scene open in the 3D editor, read from the file PVsyst exported.",
    },
)
.luego(&[VER_VENTANAS]);

pub(super) const ESCENA_IMPORTAR: Entrada = entrada(
    "gui_actuar",
    "escena_importar",
    Texto {
        es: "Malla importada en el editor 3D por PVsyst; la escena que PVsyst exportó confirma los objetos.",
        en: "Mesh imported into the 3D editor by PVsyst; the scene PVsyst exported confirms the objects.",
    },
)
.luego(&[VER_VENTANAS]);
