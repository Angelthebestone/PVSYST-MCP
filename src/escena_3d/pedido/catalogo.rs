//! The 27 elementary shapes of PVsyst 8.1.8's `TElementaryObjectForm`, as data.
//!
//! Measured live on 2026-10-29: each shape was created with its default values,
//! the scene exported with `Archivo > Exportar escena (.SHD)` and the file read
//! back (`tests/fixtures/escena_3d/Formas27_8_1_8.SHD`). Each row is the agent
//! key, PVsyst's own type name (`ObjetElemType`), the label of the combo and
//! the parameters in the order of `Params=` (the order of the dialog, top to
//! bottom). Then every shape was built again with a different value in each
//! parameter and the saved file matched the request (0 differences), so the
//! order is proven for all 27.

use super::formas::{
    angulo, apertura, caja, cuenta, giro, longitud, Forma, Parametro, PARAMETROS_DE_CAJA,
};

const ROTACION: Parametro = giro("rotacion_plano_deg", "Rotación en plano");
const LADOS: [Parametro; 3] = [
    longitud("lado_1_2", "Lado 1-2", 1.0),
    longitud("lado_2_3", "Lado 2-3", 1.0),
    longitud("lado_3_1", "Lado 3-1", 1.0),
];
const TECHO_ANCHO_LARGO: [Parametro; 5] = [
    longitud("ancho_base", "Ancho base", 8.0),
    longitud("longitud_superior", "Longitud superior", 12.0),
    angulo(
        "inclinacion_techo_deg",
        "Ángulo de inclinación del techo",
        30.0,
    ),
    angulo("aguilon_1_angulo_deg", "Aguilón 1 ángulo", 0.0),
    angulo("aguilon_2_angulo_deg", "Aguilón 2 ángulo", 0.0),
];

pub(crate) const FORMAS: &[Forma] = &[
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
    Forma::nueva(
        "paralelepipedo",
        ("oeParallelepipede", "Paralelepípedo"),
        &PARAMETROS_DE_CAJA,
    )
    .con_extension(caja),
    Forma::nueva(
        "piramide_cuadrada",
        ("oeSquarePyramid", "Pirámide Cuadrada"),
        &[
            longitud("base", "Base", 1.0),
            longitud("altura", "Altura", 2.0),
        ],
    ),
    Forma::nueva(
        "prisma_triangular",
        ("oeTriangularprism", "Prisma Triangular"),
        &[
            LADOS[0],
            LADOS[1],
            LADOS[2],
            longitud("altura", "Altura", 1.0),
        ],
    ),
    Forma::nueva(
        "prisma_hexagonal",
        ("oeHexagonalPrism", "Prisma Hexagonal"),
        &[
            longitud("diametro", "Diámetro", 1.0),
            longitud("altura", "Altura", 1.0),
        ],
    ),
    Forma::nueva(
        "prisma_octogonal",
        ("oeOctogonalPrism", "Prisma Octogonal"),
        &[
            longitud("diametro", "Diámetro", 1.0),
            longitud("altura", "Altura", 1.0),
        ],
    ),
    Forma::nueva(
        "porcion_de_cilindro",
        ("oePortionCylinder", "Porción de Cilindro"),
        &[
            longitud("radio", "Radio", 1.0),
            apertura("angulo_apertura_deg", "Ángulo de apertura", 180.0),
            cuenta("num_segmentos", "Núm. de segmentos", 9.0),
            longitud("altura", "Altura", 1.0),
        ],
    ),
    Forma::nueva(
        "casa_techo_dos_lados",
        ("oeHouse", "Casa + techo de 2 lados"),
        &[
            longitud("anchura_dx", "Anchura (DX)", 8.0),
            longitud("longitud_dy", "Longitud (DY)", 12.0),
            longitud("altura_dz", "Altura (DZ)", 5.0),
            angulo(
                "inclinacion_techo_deg",
                "Ángulo de inclinación del techo",
                30.0,
            ),
            longitud("aleros_dos_aguas", "Aleros a dos aguas", 0.5),
            longitud("aleros_laterales", "Aleros laterales", 0.5),
        ],
    ),
    Forma::nueva(
        "casa_techo_asimetrico",
        ("oeAsymHouse", "Casa, techo asimétrico"),
        &[
            longitud("anchura_dx", "Anchura (DX)", 8.0),
            longitud("longitud_dy", "Longitud (DY)", 12.0),
            longitud("altura_superior", "Altura en la parte superior", 5.0),
            angulo(
                "inclinacion_techo_1_deg",
                "Ángulo de inclinación del techo 1",
                30.0,
            ),
            angulo(
                "inclinacion_techo_2_deg",
                "Ángulo de inclinación del techo 2",
                0.5,
            ),
            longitud("proporcion_techo_1", "Proporción del techo 1", 0.5),
        ],
    ),
    Forma::nueva(
        "techo_diedro",
        ("oeRoof_Like_Diedre", "En forma de techo - diedro"),
        &TECHO_ANCHO_LARGO,
    ),
    Forma::nueva(
        "tejado_una_agua_aguilones",
        ("oeOne_Sided_Roof_Gables", "Tejado 1 fase + aguilones"),
        &TECHO_ANCHO_LARGO,
    ),
    Forma::nueva(
        "techo_dos_aguas_hastial",
        ("oeTwo_Sided_Roof_Gables", "Techo de 2 aguas + Hastial"),
        &TECHO_ANCHO_LARGO,
    ),
    Forma::nueva(
        "techo_cuatro_lados",
        ("oeFour_sided_Roof", "Techo de 4 lados"),
        &[
            longitud("ancho_base", "Ancho base", 8.0),
            longitud("longitud_base", "Longitud base", 12.0),
            longitud("longitud_superior", "Longitud superior", 8.0),
            angulo(
                "inclinacion_techo_deg",
                "Ángulo de inclinación del techo",
                30.0,
            ),
        ],
    ),
    Forma::nueva(
        "buhardilla",
        ("oeMansard", "Buhardilla"),
        &[
            longitud("ancho_frontal", "Ancho frontal", 1.5),
            longitud("altura_frontal", "Altura frontal", 1.5),
            angulo(
                "inclinacion_techo_deg",
                "Ángulo de inclinación del techo",
                45.0,
            ),
            angulo("inclinacion_base_deg", "Ángulo de inclinación base", 30.0),
        ],
    ),
    Forma::nueva(
        "prisma_chimenea",
        ("oePrism_Chimney", "Prisma - chimenea"),
        &[
            longitud("profundidad", "Profundidad", 1.0),
            longitud("anchura", "Anchura", 1.0),
            longitud("altura", "Altura", 1.0),
            angulo(
                "inclinacion_techo_deg",
                "Ángulo de inclinación del techo",
                30.0,
            ),
        ],
    ),
    Forma::nueva(
        "arbol",
        ("oeTree", "Árbol"),
        &[
            longitud("altura_punto_medio", "Altura punto medio", 3.0),
            longitud("altura_media", "Altura media", 3.0),
            longitud("altura_parte_baja", "Altura parte baja", 2.0),
            longitud("altura_tronco", "Altura de tronco", 2.0),
            longitud("diametro_medio", "Diámetro medio", 4.0),
            longitud("diametro_tronco", "Diámetro de tronco", 0.5),
        ],
    ),
    Forma::nueva(
        "cable",
        ("oeCable", "Cable"),
        &[
            longitud("longitud", "Longitud", 10.0),
            longitud("pendiente", "Pendiente", 1.0),
            longitud("espesor", "Espesor", 0.2),
            cuenta("num_segmentos", "Núm. de segmentos", 6.0),
            angulo("angulo_pendiente_deg", "Ángulo de la pendiente", 0.0),
        ],
    ),
    Forma::nueva(
        "barandilla_un_travesano",
        ("oeHandrail1Crossbar", "Barandilla un travesaño"),
        &[
            longitud("longitud", "Longitud", 10.0),
            longitud("altura", "Altura", 1.0),
            longitud("espesor", "Espesor", 0.05),
            cuenta("num_peldanos", "Número de peldaños", 5.0),
        ],
    ),
    Forma::nueva(
        "barandilla_dos_travesanos",
        ("oeHandrail2Crossbar", "Barandilla dos travesaños"),
        &[
            longitud("longitud", "Longitud", 10.0),
            longitud("altura", "Altura", 1.0),
            longitud("altura_segundo_travesano", "Altura del 2do travesaño", 0.6),
            longitud("espesor", "Espesor", 0.05),
            cuenta("num_peldanos", "Número de peldaños", 5.0),
        ],
    ),
    Forma::nueva(
        "turbina_eolica",
        ("oeWindTurbine", "Turbina eólica"),
        &[
            longitud("altura_soporte", "Altura del soporte", 40.0),
            longitud("soporte_menor_diametro", "Soporte de menor diámetro", 4.0),
            longitud(
                "diametro_superior_soporte",
                "Diámetro superior del soporte",
                3.0,
            ),
            longitud("pala_longitud_completa", "Pala de longitud completa", 30.0),
            longitud("pala_longitud_pequena", "Pala de longitud pequeña", 5.0),
            longitud("pala_ancho_original", "Ancho original de la pala", 3.0),
            longitud("pala_ancho_mediano", "Ancho mediano de la pala", 3.5),
            longitud("pala_ancho_final", "Ancho final de la pala", 1.0),
            longitud("generador_longitud", "Longitud del generador", 5.0),
            longitud("generador_diametro", "Diámetro del generador", 4.0),
            angulo("pala_angulo_deg", "Ángulo de las palas", 0.0),
        ],
    ),
    Forma::nueva(
        "pilon_electrico",
        ("oeElectricPylon", "Pilón eléctrico"),
        &[
            longitud("altura", "Altura", 40.0),
            longitud("longitud_base", "Longitud base", 8.0),
            longitud("longitud_superior", "Longitud superior", 5.0),
            longitud("anchura", "Anchura", 4.0),
            cuenta("num_brazos", "Número de brazos", 3.0),
            longitud("altura_primer_brazo", "Altura del primer brazo", 20.0),
            longitud("longitud_brazo", "Longitud del brazo", 12.0),
            longitud("ancho_brazo", "Ancho del brazo", 2.0),
            longitud("longitud_aislador", "Longitud del aislador", 0.3),
            longitud("altura_aislador", "Altura del aislador", 3.0),
            longitud("espesor_malla", "Espesor de malla", 0.4),
        ],
    ),
];
