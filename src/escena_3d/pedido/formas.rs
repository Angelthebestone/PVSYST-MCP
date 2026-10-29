//! What a shape is, as data: its key, PVsyst's type name and the parameters in
//! the order of `Params=` in the file. The rows are in `catalogo.rs`.

use std::collections::BTreeMap;

pub(super) use super::catalogo::FORMAS;

/// Values of one object by key, in meters and degrees.
pub(super) type Valores = BTreeMap<&'static str, f64>;

/// Largest and smallest values the MCP accepts. PVsyst's own limits were not
/// measured, so these only keep absurd requests out; they are not PVsyst's.
const MINIMO_LONGITUD_M: f64 = 0.01;
const MAXIMO_LONGITUD_M: f64 = 1000.0;
const MAXIMO_INCLINACION_DEG: f64 = 89.9;
const MAXIMO_GIRO_DEG: f64 = 360.0;
const MAXIMO_APERTURA_DEG: f64 = 360.0;
const MAXIMO_CUENTA: f64 = 100.0;

/// One value of a shape, in the order the file keeps it.
#[derive(Debug, Clone, Copy)]
pub(crate) struct Parametro {
    pub(crate) clave: &'static str,
    /// The label of the field in PVsyst's dialog.
    pub(crate) etiqueta: &'static str,
    pub(crate) unidad: &'static str,
    /// What PVsyst's dialog shows for a new object.
    pub(crate) defecto: f64,
    /// An omitted optional value takes `defecto`.
    pub(crate) obligatorio: bool,
    pub(super) minimo: f64,
    pub(super) maximo: f64,
}

/// A shape: its agent key, PVsyst's type name and its parameters.
pub(crate) struct Forma {
    pub(crate) tipo: &'static str,
    /// `ObjetElemType` in the saved file.
    pub(crate) tipo_pvsyst: &'static str,
    /// The entry of the shape combo in PVsyst's dialog.
    pub(crate) nombre_pvsyst: &'static str,
    pub(crate) parametros: &'static [Parametro],
    /// Size of the box it fills, from its values: (x, y, z) in meters. Only
    /// for the shapes whose geometry is proven.
    pub(super) extension: Option<fn(&Valores) -> [f64; 3]>,
}

impl Forma {
    /// `nombres` is (`ObjetElemType`, entry of PVsyst's combo).
    pub(super) const fn nueva(
        tipo: &'static str,
        nombres: (&'static str, &'static str),
        parametros: &'static [Parametro],
    ) -> Forma {
        Forma {
            tipo,
            tipo_pvsyst: nombres.0,
            nombre_pvsyst: nombres.1,
            parametros,
            extension: None,
        }
    }

    pub(super) const fn con_extension(self, extension: fn(&Valores) -> [f64; 3]) -> Forma {
        Forma {
            extension: Some(extension),
            ..self
        }
    }
}

/// `nombres` is (key, label); `valores` is (default, minimum, maximum).
const fn parametro(
    nombres: (&'static str, &'static str),
    unidad: &'static str,
    valores: (f64, f64, f64),
) -> Parametro {
    Parametro {
        clave: nombres.0,
        etiqueta: nombres.1,
        unidad,
        defecto: valores.0,
        obligatorio: true,
        minimo: valores.1,
        maximo: valores.2,
    }
}

pub(super) const fn longitud(
    clave: &'static str,
    etiqueta: &'static str,
    defecto: f64,
) -> Parametro {
    parametro(
        (clave, etiqueta),
        "m",
        (defecto, MINIMO_LONGITUD_M, MAXIMO_LONGITUD_M),
    )
}

/// An inclination or an opening angle, in degrees.
pub(super) const fn angulo(clave: &'static str, etiqueta: &'static str, defecto: f64) -> Parametro {
    parametro(
        (clave, etiqueta),
        "deg",
        (defecto, 0.0, MAXIMO_INCLINACION_DEG),
    )
}

/// A number of sides, segments, arms or rungs.
pub(super) const fn cuenta(clave: &'static str, etiqueta: &'static str, defecto: f64) -> Parametro {
    parametro((clave, etiqueta), "", (defecto, 1.0, MAXIMO_CUENTA))
}

/// A turn in the plane of the shape; optional, 0 when omitted.
pub(super) const fn giro(clave: &'static str, etiqueta: &'static str) -> Parametro {
    Parametro {
        obligatorio: false,
        ..parametro(
            (clave, etiqueta),
            "deg",
            (0.0, -MAXIMO_GIRO_DEG, MAXIMO_GIRO_DEG),
        )
    }
}

/// A value whose label was not read (the dialog scrolls and the OCR missed it).
pub(super) const fn sin_leer(clave: &'static str, defecto: f64) -> Parametro {
    parametro(
        (clave, "(etiqueta no leída)"),
        "",
        (defecto, -MAXIMO_LONGITUD_M, MAXIMO_LONGITUD_M),
    )
}

/// An opening angle goes up to a full turn, unlike an inclination.
pub(super) const fn apertura(
    clave: &'static str,
    etiqueta: &'static str,
    defecto: f64,
) -> Parametro {
    parametro(
        (clave, etiqueta),
        "deg",
        (defecto, 1.0, MAXIMO_APERTURA_DEG),
    )
}

pub(super) const PARAMETROS_DE_CAJA: [Parametro; 3] = [
    longitud("anchura_dx", "Anchura (DX)", 1.0),
    longitud("longitud_dy", "Longitud (DY)", 1.0),
    longitud("altura_dz", "Altura (DZ)", 1.0),
];

fn valor(v: &Valores, clave: &str) -> f64 {
    v.get(clave).copied().unwrap_or(0.0)
}

/// The box of a parallelepiped: DX along x, DY along y, DZ along z.
pub(super) fn caja(v: &Valores) -> [f64; 3] {
    [
        valor(v, "anchura_dx"),
        valor(v, "longitud_dy"),
        valor(v, "altura_dz"),
    ]
}

/// The shape named `tipo`. O(27).
pub(crate) fn forma_de(tipo: &str) -> Option<&'static Forma> {
    FORMAS.iter().find(|f| f.tipo == tipo)
}
