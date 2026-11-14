//! System window of a grid-connected variant (`TSystemegridForm`): the pre-sizing help and the
//! array layout, PVsyst 8.1.8. Anchors are the control corners in the form's 96-DPI layout,
//! measured live. The option comes before its field so a fill chooses the mode first.

use crate::vista_agente::dominio::{Ancla, CampoEsquema, Entidad};

const fn en(x: i32, y: i32) -> Ancla {
    Ancla { x, y }
}

pub(crate) static SISTEMA: Entidad = Entidad {
    clave: "variante.system",
    formulario: "TSystemegridForm",
    titulo: "Definición del sistema conectado a red",
    campos: &[
        CampoEsquema::opcion("usar_potencia_planeada", "Potencia planeada", en(1044, 97)),
        CampoEsquema::numero(
            "potencia_planeada_kwp",
            "Potencia planeada",
            en(1059, 95),
            (0.0, 1_000_000.0),
        )
        .en_archivo("PNomSizing")
        .en("kWp"),
        CampoEsquema::opcion("usar_area_disponible", "Área disponible", en(1044, 120)),
        CampoEsquema::numero(
            "area_disponible_m2",
            "Área disponible",
            en(1059, 120),
            (0.0, 100_000_000.0),
        )
        .en("m²"),
        CampoEsquema::entero(
            "n_inversores",
            "Número de inversores",
            en(503, 377),
            (1, 10_000),
        )
        .en_archivo("NInverter"),
        CampoEsquema::entero(
            "n_modulos_serie",
            "Módulos en serie",
            en(458, 508),
            (1, 1000),
        )
        .en_archivo("NModSerie"),
        CampoEsquema::entero("n_cadenas", "Número de cadenas", en(458, 536), (1, 100_000))
            .en_archivo("NStringCh"),
    ],
};
