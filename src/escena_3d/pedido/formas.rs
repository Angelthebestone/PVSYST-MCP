//! The elementary shapes an agent can ask for, as data: the key and unit of
//! each dimension, the slot it takes in the 39-byte block of the file
//! (`docs/pvsyst_format.md` §7.3) and the box it fills.

use std::collections::BTreeMap;

/// Values of one object by key, in meters and degrees.
pub(super) type Valores = BTreeMap<&'static str, f64>;

/// Largest and smallest values the MCP accepts. PVsyst's own limits were not
/// measured, so these only keep absurd requests out; they are not PVsyst's.
pub(super) const MINIMO_LONGITUD_M: f64 = 0.01;
pub(super) const MAXIMO_LONGITUD_M: f64 = 1000.0;
pub(super) const MAXIMO_INCLINACION_DEG: f64 = 89.9;
pub(super) const MAXIMO_RADIO_M: f64 = 100.0;

/// One dimension of a shape.
#[derive(Debug, Clone, Copy)]
pub(crate) struct Parametro {
    pub(crate) clave: &'static str,
    pub(crate) unidad: &'static str,
    /// Slot of the six `Real48` values in the shape block.
    pub(crate) ranura: usize,
    pub(super) minimo: f64,
    pub(super) maximo: f64,
}

/// A shape: its agent name, its file code and what it needs.
pub(crate) struct Forma {
    pub(crate) tipo: &'static str,
    pub(crate) codigo: u16,
    pub(crate) nombre_pvsyst: &'static str,
    pub(crate) parametros: &'static [Parametro],
    /// Size of the box it fills, from its values: (x, y, z) in meters.
    pub(super) extension: fn(&Valores) -> [f64; 3],
}

const fn longitud(clave: &'static str, ranura: usize) -> Parametro {
    Parametro {
        clave,
        unidad: "m",
        ranura,
        minimo: MINIMO_LONGITUD_M,
        maximo: MAXIMO_LONGITUD_M,
    }
}

const fn radio(clave: &'static str, ranura: usize) -> Parametro {
    Parametro {
        clave,
        unidad: "m",
        ranura,
        minimo: MINIMO_LONGITUD_M,
        maximo: MAXIMO_RADIO_M,
    }
}

const fn inclinacion(clave: &'static str, ranura: usize) -> Parametro {
    Parametro {
        clave,
        unidad: "deg",
        ranura,
        minimo: 0.0,
        maximo: MAXIMO_INCLINACION_DEG,
    }
}

fn valor(v: &Valores, clave: &str) -> f64 {
    v.get(clave).copied().unwrap_or(0.0)
}

fn caja(v: &Valores) -> [f64; 3] {
    [valor(v, "largo"), valor(v, "ancho"), valor(v, "alto")]
}

fn tejado(v: &Valores) -> [f64; 3] {
    let cumbrera = valor(v, "ancho") / 2.0 * valor(v, "inclinacion_deg").to_radians().tan();
    [valor(v, "largo"), valor(v, "ancho"), cumbrera]
}

fn chimenea(v: &Valores) -> [f64; 3] {
    [valor(v, "ancho"), valor(v, "largo"), valor(v, "alto")]
}

fn arbol(v: &Valores) -> [f64; 3] {
    let diametro = 2.0 * valor(v, "radio_copa");
    [diametro, diametro, valor(v, "altura")]
}

/// Shapes confirmed by reading files PVsyst saved (§7.3). Others the editor
/// offers stay unproven until PVsyst can save them.
pub(crate) const FORMAS: &[Forma] = &[
    Forma {
        tipo: "paralelepipedo",
        codigo: 11,
        nombre_pvsyst: "Parallélépipède",
        parametros: &[
            longitud("largo", 0),
            longitud("ancho", 1),
            longitud("alto", 2),
        ],
        extension: caja,
    },
    Forma {
        tipo: "tejado_dos_aguas",
        codigo: 23,
        nombre_pvsyst: "Toiture + pignons",
        parametros: &[
            longitud("largo", 0),
            longitud("ancho", 1),
            inclinacion("inclinacion_deg", 2),
        ],
        extension: tejado,
    },
    Forma {
        tipo: "chimenea",
        codigo: 25,
        nombre_pvsyst: "Prism - chimney",
        parametros: &[
            longitud("ancho", 0),
            longitud("largo", 1),
            longitud("alto", 2),
            inclinacion("inclinacion_deg", 3),
        ],
        extension: chimenea,
    },
    Forma {
        tipo: "arbol",
        codigo: 21,
        nombre_pvsyst: "Tree",
        parametros: &[
            radio("radio_copa", 0),
            radio("radio_tronco", 1),
            longitud("altura", 2),
        ],
        extension: arbol,
    },
];

/// The shape named `tipo`. O(4).
pub(crate) fn forma_de(tipo: &str) -> Option<&'static Forma> {
    FORMAS.iter().find(|f| f.tipo == tipo)
}
