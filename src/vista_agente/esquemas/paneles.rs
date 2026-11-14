//! PV module window (`TPVModuleForm`, "Datos básicos" and "Dimensiones y tecnología"
//! pages). Anchors are the control corners in the form's 96-DPI layout, measured live
//! on PVsyst 8.1.7 (the module size ones on 8.1.8, read with `gui_observar que:inventario`);
//! units and file keys were checked against a `.PAN` saved by PVsyst 8.1.8. [INFERENCIA] ranges and
//! the `clave_archivo` of fields not in that `.PAN` have no measured source;
//! file keys are the ones `componente crear_desde_ficha` already writes.

use crate::vista_agente::dominio::{Ancla, CampoEsquema, Entidad};

const fn en(x: i32, y: i32) -> Ancla {
    Ancla { x, y }
}

pub(crate) static PANELES: Entidad = Entidad {
    clave: "components.panels",
    formulario: "TPVModuleForm",
    titulo: "Definición de un módulo FV",
    campos: &[
        CampoEsquema::texto("modelo", "Modelo", en(121, 75)).en_archivo("Model"),
        CampoEsquema::texto("fabricante", "Fabricante", en(491, 75)).en_archivo("Manufacturer"),
        CampoEsquema::texto("nombre_archivo", "Nombre de archivo", en(121, 101)),
        CampoEsquema::numero(
            "potencia_nominal_w",
            "Potencia nom.",
            en(116, 179),
            (0.1, 2000.0),
        )
        .en_archivo("PNom")
        .en("W"),
        CampoEsquema::lista("tecnologia", "Tecnología", en(116, 211)).en_archivo("Technol"),
        CampoEsquema::numero("g_ref_w_m2", "GRef", en(233, 276), (100.0, 2000.0))
            .en_archivo("GRef")
            .en("W/m2"),
        CampoEsquema::numero("t_ref_c", "TRef", en(478, 276), (-50.0, 100.0))
            .en_archivo("TRef")
            .en("°C"),
        CampoEsquema::numero("isc_a", "Isc", en(233, 300), (0.0, 100.0))
            .en_archivo("Isc")
            .en("A"),
        CampoEsquema::numero("voc_v", "Voc", en(478, 300), (0.0, 300.0))
            .en_archivo("Voc")
            .en("V"),
        CampoEsquema::numero("impp_a", "Impp", en(233, 324), (0.0, 100.0))
            .en_archivo("Imp")
            .en("A"),
        CampoEsquema::numero("vmpp_v", "Vmpp", en(478, 324), (0.0, 300.0))
            .en_archivo("Vmp")
            .en("V"),
        CampoEsquema::numero("mu_isc_ma_c", "muIsc", en(233, 348), (-100.0, 100.0))
            .en_archivo("muISC")
            .en("mA/°C"),
        CampoEsquema::entero(
            "celulas_en_serie",
            "Núm células en serie",
            en(478, 348),
            (1, 400),
        )
        .en_archivo("NCelS"),
        CampoEsquema::numero("mu_isc_pct_c", "o muIsc", en(233, 372), (-5.0, 5.0)).en("%/°C"),
        CampoEsquema::numero("t_oper_c", "TOper", en(468, 427), (-50.0, 150.0))
            .en("°C")
            .solo_lectura(),
        // The window takes the size in mm and kg; the file stores `Width`, `Height` and
        // `Depth` in metres. PVsyst refuses to save a module without its size.
        CampoEsquema::numero("longitud_mm", "Longitud", en(124, 121), (0.0, 5000.0))
            .en_archivo("Height")
            .en("mm")
            .en_pagina(1),
        CampoEsquema::numero("anchura_mm", "Anchura", en(124, 145), (0.0, 5000.0))
            .en_archivo("Width")
            .en("mm")
            .en_pagina(1),
        CampoEsquema::numero("espesor_mm", "Espesor", en(124, 169), (0.0, 500.0))
            .en_archivo("Depth")
            .en("mm")
            .en_pagina(1),
        CampoEsquema::numero("peso_kg", "Peso", en(124, 193), (0.0, 500.0))
            .en_archivo("Weight")
            .en("kg")
            .en_pagina(1),
        CampoEsquema::numero("g_oper_w_m2", "GOper", en(243, 427), (0.0, 2000.0))
            .en("W/m2")
            .solo_lectura(),
    ],
};
