//! Orientation window of a grid-connected variant (`TOrientationForm`, fixed tilted plane),
//! PVsyst 8.1.8. Anchors are the control corners in the form's 96-DPI layout, measured live.
//! The fields below are the ones a fixed plane shows; the tracker and shed types show others.

use crate::vista_agente::dominio::{Ancla, CampoEsquema, Entidad};

const fn en(x: i32, y: i32) -> Ancla {
    Ancla { x, y }
}

pub(crate) static ORIENTACION: Entidad = Entidad {
    clave: "variante.orientation",
    formulario: "TOrientationForm",
    titulo: "Gestión de orientaciones",
    campos: &[
        CampoEsquema::lista("tipo_de_campo", "Tipo de campo", en(45, 155)).en_archivo("FieldType"),
        CampoEsquema::texto("nombre", "Nombre de la orientación", en(295, 156))
            .en_archivo("Comment"),
        CampoEsquema::numero(
            "inclinacion_plano_deg",
            "Inclinación del plano",
            en(195, 213),
            (0.0, 90.0),
        )
        .en_archivo("FieldTilt")
        .en("°"),
        CampoEsquema::numero("azimut_deg", "Azimut", en(195, 237), (-180.0, 180.0))
            .en_archivo("FieldAzim")
            .en("°"),
        CampoEsquema::numero(
            "pendiente_base_deg",
            "Ángulo de inclinación base",
            en(195, 261),
            (-90.0, 90.0),
        )
        .en_archivo("BaseSlope")
        .en("°"),
    ],
};
