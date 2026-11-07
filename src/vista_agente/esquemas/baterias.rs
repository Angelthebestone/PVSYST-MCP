//! Battery window (`TBatteryForm`, "Datos básicos" page), PVsyst 8.1.8. Anchors
//! are the control corners in the form's 96-DPI layout, measured live; file keys
//! and units come from a `.BTR` PVsyst saved. The file stores `ResInterne` in ohm
//! (the window shows mOhm) and `EfficI` as a fraction (the window shows %).

use crate::vista_agente::dominio::{Ancla, CampoEsquema, Entidad};

const fn en(x: i32, y: i32) -> Ancla {
    Ancla { x, y }
}

pub(crate) static BATERIAS: Entidad = Entidad {
    clave: "components.batteries",
    formulario: "TBatteryForm",
    titulo: "Definición de una batería",
    campos: &[
        CampoEsquema::texto("modelo", "Modelo", en(121, 80)).en_archivo("Model"),
        CampoEsquema::texto("fabricante", "Fabricante", en(515, 80)).en_archivo("Manufacturer"),
        CampoEsquema::texto("nombre_archivo", "Nombre de archivo", en(121, 106)),
        CampoEsquema::lista("tecnologia", "Tecnología", en(101, 164)).en_archivo("BattTechnol"),
        CampoEsquema::entero(
            "elementos_en_serie",
            "Núm. de elementos en serie",
            en(243, 269),
            (1, 1000),
        )
        .en_archivo("NCellSerie"),
        CampoEsquema::numero(
            "voltaje_nominal_v",
            "Voltaje nominal",
            en(243, 295),
            (0.1, 100.0),
        )
        .en_archivo("VNomCell")
        .en("V"),
        CampoEsquema::numero(
            "capacidad_cio_ah",
            "Capacidad en C10",
            en(243, 321),
            (0.1, 100000.0),
        )
        .en_archivo("CapNomC10")
        .en("Ah"),
        CampoEsquema::numero(
            "resistencia_interna_mohm",
            "Resistencia interna @ temp. ref.",
            en(243, 348),
            (0.0, 10000.0),
        )
        .en_archivo("ResInterne")
        .en("mOhm"),
        CampoEsquema::numero(
            "temperatura_ref_c",
            "Temperatura de referencia",
            en(243, 374),
            (-50.0, 100.0),
        )
        .en_archivo("BattTempRef")
        .en("°C"),
        CampoEsquema::numero(
            "eficiencia_coulombica_pct",
            "Eficiencia coulómbica",
            en(243, 401),
            (1.0, 100.0),
        )
        .en_archivo("EfficI")
        .en("%"),
    ],
};
