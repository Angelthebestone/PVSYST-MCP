//! Grid inverter window (`TGridInverterForm`, "Parámetros principales" and "Curva de
//! eficiencia" pages),
//! PVsyst 8.1.8. Anchors are the control corners in the form's 96-DPI layout,
//! measured live; file keys and units come from an `.OND` PVsyst saved.

use crate::vista_agente::dominio::{Ancla, CampoEsquema, Entidad};

const fn en(x: i32, y: i32) -> Ancla {
    Ancla { x, y }
}

pub(crate) static INVERSORES: Entidad = Entidad {
    clave: "components.inverters",
    formulario: "TGridInverterForm",
    titulo: "Definición del inversor de red",
    campos: &[
        CampoEsquema::texto("modelo", "Modelo", en(123, 80)).en_archivo("Model"),
        CampoEsquema::texto("fabricante", "Fabricante", en(485, 80)).en_archivo("Manufacturer"),
        CampoEsquema::texto("nombre_archivo", "Nombre de archivo", en(123, 106)),
        CampoEsquema::numero(
            "voltaje_mpp_min_v",
            "Voltaje MPP mínimo",
            en(267, 193),
            (0.0, 2000.0),
        )
        .en_archivo("VMppMin")
        .en("V"),
        CampoEsquema::numero(
            "voltaje_min_pnom_v",
            "Voltaje mín. para PNom",
            en(267, 217),
            (0.0, 2000.0),
        )
        .en("V"),
        CampoEsquema::numero(
            "corriente_entrada_max_a",
            "Corriente de entrada máxima",
            en(267, 241),
            (0.0, 5000.0),
        )
        .en("A"),
        CampoEsquema::numero(
            "voltaje_mpp_nominal_v",
            "Voltaje MPP nominal",
            en(267, 265),
            (0.0, 2000.0),
        )
        .en_archivo("VmppNom")
        .en("V"),
        CampoEsquema::numero(
            "voltaje_mpp_max_v",
            "Voltaje MPP máximo",
            en(267, 289),
            (0.0, 2000.0),
        )
        .en_archivo("VMPPMax")
        .en("V"),
        CampoEsquema::numero(
            "voltaje_fv_max_abs_v",
            "Voltaje FV máx. absoluto",
            en(267, 313),
            (0.0, 2000.0),
        )
        .en_archivo("VAbsMax")
        .en("V"),
        CampoEsquema::numero(
            "voltaje_red_v",
            "Voltaje de red",
            en(527, 272),
            (0.0, 2000.0),
        )
        .en_archivo("VOutConv")
        .en("V"),
        CampoEsquema::numero(
            "potencia_ca_nominal_kw",
            "Potencia de CA nominal",
            en(527, 297),
            (0.0, 10000.0),
        )
        .en_archivo("PNomConv")
        .en("kW"),
        CampoEsquema::numero(
            "potencia_ca_max_kw",
            "Potencia CA máxima",
            en(527, 321),
            (0.0, 10000.0),
        )
        .en_archivo("PMaxOUT")
        .en("kW"),
        CampoEsquema::numero(
            "corriente_ca_nominal_a",
            "Corriente AC nominal",
            en(527, 345),
            (0.0, 20000.0),
        )
        .en("A"),
        CampoEsquema::numero(
            "corriente_ca_max_a",
            "Corriente AC máxima",
            en(527, 369),
            (0.0, 20000.0),
        )
        .en("A"),
        CampoEsquema::numero(
            "umbral_potencia_kw",
            "Umbral de potencia",
            en(221, 354),
            (0.0, 1000.0),
        )
        .en_archivo("PSeuil")
        .en("kW"),
        CampoEsquema::numero(
            "eficiencia_max_pct",
            "Eficiencia máx.",
            en(479, 127),
            (0.0, 100.0),
        )
        .en_archivo("EfficMax")
        .en("%")
        .en_pagina(1),
        CampoEsquema::numero(
            "eficiencia_euro_pct",
            "Eficiencia EURO",
            en(479, 154),
            (0.0, 100.0),
        )
        .en_archivo("EfficEuro")
        .en("%")
        .en_pagina(1),
    ],
};
