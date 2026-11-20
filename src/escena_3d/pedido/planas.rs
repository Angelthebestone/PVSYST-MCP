//! The flat shapes (triangles, rectangle, trapeze, polygon and sector) of PVsyst 8.1.8's
//! `TElementaryObjectForm`; the solids are in `catalogo.rs`.

use super::formas::{apertura, cuenta, giro, longitud, Forma, Parametro};

const ROTACION: Parametro = giro("rotacion_plano_deg", "Rotación en plano");
/// The three sides shared by the triangle and the triangular prism.
pub(super) const LADOS: [Parametro; 3] = [
    longitud("lado_1_2", "Lado 1-2", 1.0),
    longitud("lado_2_3", "Lado 2-3", 1.0),
    longitud("lado_3_1", "Lado 3-1", 1.0),
];

pub(super) const PLANAS: &[Forma] = &[
    Forma::nueva(
        "triangulo",
        ("oeTriangle", "Triángulo"),
        &[LADOS[0], LADOS[1], LADOS[2], ROTACION],
    ),
    Forma::nueva(
        "triangulo_isosceles",
        ("oeTriangleIsocele", "Triángulo Isósceles"),
        &[
            longitud("base", "Base", 1.0),
            longitud("altura", "Altura", 1.0),
            ROTACION,
        ],
    ),
    Forma::nueva(
        "triangulo_rectangulo",
        ("oeTriangleRectangle", "Triángulo Rectángulo"),
        &[
            longitud("base", "Base", 1.0),
            longitud("altura", "Altura", 1.0),
            ROTACION,
        ],
    ),
    Forma::nueva(
        "rectangulo",
        ("oeRectangle", "Rectángulo"),
        &[
            longitud("anchura", "Anchura", 1.0),
            longitud("altura", "Altura", 2.0),
            ROTACION,
        ],
    ),
    Forma::nueva(
        "trapecio",
        ("oeTrapeze", "Trapecio"),
        &[
            longitud("base_inferior", "Base inferior", 3.0),
            longitud("base_superior", "Base superior", 2.0),
            longitud("altura", "Altura", 1.0),
            longitud("desviacion_izquierda", "Desviación izquierda", 0.5),
            ROTACION,
        ],
    ),
    Forma::nueva(
        "poligono_regular",
        ("oeRegularPolygon", "Polígono regular"),
        &[
            longitud("radio", "Radio", 1.0),
            cuenta("num_lados", "Núm. de lados", 6.0),
            ROTACION,
        ],
    ),
    Forma::nueva(
        "sector_seudocircular",
        ("oePseudoCircleSector", "Sector seudocircular"),
        &[
            longitud("radio", "Radio", 1.0),
            apertura("angulo_apertura_deg", "Ángulo de apertura", 180.0),
            cuenta("num_segmentos", "Núm. de segmentos", 8.0),
            ROTACION,
        ],
    ),
];
