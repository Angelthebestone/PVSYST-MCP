//! The `proyecto anadir` element catalogue: which demo variant each element is copied
//! from, and the fields the target system takes.

use crate::error::{Error, Result};
use crate::idioma::Texto;
use crate::projects::tipo_sistema::TipoSistema;

/// One catalogue entry: the demo variant and the object types to copy from it.
pub(crate) struct Elemento {
    pub(crate) nombre: &'static str,
    pub(crate) demo: &'static str,
    pub(crate) tipos: &'static [&'static str],
    pub(crate) complementos: &'static [&'static str],
    pub(crate) campos_sistema: &'static [(&'static str, &'static str)],
    /// System type the element belongs to: PVsyst cannot use it in any other.
    pub(crate) sistema: Option<TipoSistema>,
    /// What the caller has to edit after inserting it, when the content is the demo's.
    pub(crate) aviso: Option<Texto>,
}

/// The demo geometry of a field zone is not the project's.
const ZONA_DE_DEMO: Texto = Texto {
    es: "La geometría insertada es la de la demo (Zone #1 y sus módulos): edítala para que coincida con el proyecto",
    en: "The inserted geometry is the demo's (Zone #1 and its modules): edit it to match the project",
};

pub(crate) const ELEMENTOS: &[Elemento] = &[
    Elemento {
        nombre: "baterias",
        demo: "_DEMO_Residential.VC5",
        tipos: &["pvGridStorage"],
        complementos: &["carga"],
        campos_sistema: &[
            ("Flags", "$10010040"),
            ("BatteryFile", "Generic_LI_Module_180Ah.BTR"),
            ("CycleSOWBeg", "1.000"),
            ("StaticSOWBeg", "1.000"),
            ("TypeTempBatt", "1"),
            ("TempBattRef", "20.0"),
            ("TempBattMois", "20,20,20,20,20,20,20,20,20,20,20,20,"),
            ("NBattSerie", "1"),
            ("NBattParall", "2"),
        ],
        sistema: None,
        aviso: None,
    },
    Elemento {
        nombre: "carga",
        demo: "_DEMO_Commercial.VC3",
        tipos: &["pvLoad"],
        complementos: &[],
        campos_sistema: &[],
        sistema: None,
        aviso: None,
    },
    Elemento {
        nombre: "horizonte",
        demo: "_DEMO_Residential.VC1",
        tipos: &["pvHorizon"],
        complementos: &[],
        campos_sistema: &[],
        sistema: None,
        aviso: None,
    },
    Elemento {
        nombre: "envejecimiento",
        demo: "_DEMO_Commercial.VC8",
        tipos: &["pvAgeing"],
        complementos: &[],
        campos_sistema: &[],
        sistema: None,
        aviso: None,
    },
    Elemento {
        nombre: "bifacial",
        demo: "_DEMO_Utility.VC2",
        tipos: &["pvBiFacial", "pvBiFacialSheds"],
        complementos: &[],
        campos_sistema: &[],
        sistema: None,
        aviso: None,
    },
    Elemento {
        nombre: "carbono",
        demo: "_DEMO_Residential.VC6",
        tipos: &["pvCarbonBalance"],
        complementos: &[],
        campos_sistema: &[],
        sistema: None,
        aviso: None,
    },
    Elemento {
        nombre: "domo",
        demo: "_DEMO_Commercial.VC7",
        tipos: &["pvDomeOrient"],
        complementos: &[],
        campos_sistema: &[],
        sistema: None,
        aviso: None,
    },
    Elemento {
        nombre: "zona_campo",
        demo: "_DEMO_Tracking.VC2",
        tipos: &["pvShdFieldZone"],
        complementos: &[],
        campos_sistema: &[],
        sistema: None,
        aviso: Some(ZONA_DE_DEMO),
    },
    Elemento {
        nombre: "bifacial_seguidor",
        demo: "_DEMO_Utility.VC5",
        tipos: &["pvBiFacial", "pvBiFacialTrackers"],
        complementos: &[],
        campos_sistema: &[],
        sistema: None,
        aviso: None,
    },
    Elemento {
        nombre: "circuito_hidraulico",
        demo: "_DEMO_Pumping.VC0",
        tipos: &["pvHydrCircuit"],
        complementos: &[],
        campos_sistema: &[
            ("SystemType", "Pumping"),
            ("PumpFile", "Generic_6kW_H60_160_FR10.PMP"),
        ],
        sistema: Some(TipoSistema::Bombeo),
        aviso: None,
    },
];

pub(crate) fn catalogo_de(nombre: &str) -> Result<&'static Elemento> {
    ELEMENTOS
        .iter()
        .find(|e| e.nombre == nombre)
        .ok_or_else(|| {
            let validos: Vec<&str> = ELEMENTOS.iter().map(|e| e.nombre).collect();
            Error::NotFound(format!(
                "Elemento '{nombre}' desconocido; válidos: {}",
                validos.join(", ")
            ))
        })
}
