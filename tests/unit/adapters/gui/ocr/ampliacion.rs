use super::*;

fn gris(valor: u8, pixeles: usize) -> Vec<u8> {
    (0..pixeles)
        .flat_map(|_| [valor, valor, valor, 255])
        .collect()
}

fn azules(bytes: &[u8]) -> Vec<u8> {
    bytes.chunks(BYTES_POR_PIXEL).map(|p| p[0]).collect()
}

#[test]
fn should_keep_flat_colour_and_alpha_when_enlarging_a_uniform_image() {
    let origen = gris(90, 4);
    let imagen = ImagenBgra {
        pixeles: &origen,
        ancho: 2,
        alto: 2,
    };

    let salida = ampliar(imagen, Region { ancho: 2, alto: 2 }, 3);

    assert_eq!(salida.len(), 6 * 6 * BYTES_POR_PIXEL);
    assert!(salida
        .chunks(BYTES_POR_PIXEL)
        .all(|p| p == [90, 90, 90, 255]));
}

#[test]
fn should_overshoot_both_sides_of_the_edge_when_enlarging_a_dark_to_bright_pair() {
    let origen: Vec<u8> = [[40, 40, 40, 255], [200, 200, 200, 255]].concat();
    let imagen = ImagenBgra {
        pixeles: &origen,
        ancho: 2,
        alto: 1,
    };

    let salida = azules(&ampliar(
        imagen,
        Region { ancho: 2, alto: 1 },
        FACTOR_MAXIMO,
    ));
    let fila = &salida[..2 * FACTOR_MAXIMO as usize];

    // Interpolation alone never leaves [40, 200]; sharpening pushes the dark
    // side of the edge below 40 and the bright side above 200.
    assert!(fila.iter().any(|v| *v < 40), "{fila:?}");
    assert!(fila.iter().any(|v| *v > 200), "{fila:?}");
    assert_eq!((fila[0], fila[fila.len() - 1]), (40, 200));
}

#[test]
fn should_ignore_pixels_outside_region_when_cropping_a_wider_capture() {
    let origen: Vec<u8> = [[10, 10, 10, 255], [250, 250, 250, 255]].concat();
    let imagen = ImagenBgra {
        pixeles: &origen,
        ancho: 2,
        alto: 1,
    };

    let salida = ampliar(imagen, Region { ancho: 1, alto: 1 }, 2);

    assert!(salida.chunks(BYTES_POR_PIXEL).all(|p| p[0] == 10));
}

#[test]
fn should_pick_the_largest_factor_the_ocr_limit_allows_when_the_region_is_small_or_wide() {
    let casos = [
        ((300, 200), Some(FACTOR_MAXIMO)),
        ((866, 700), Some(3)),
        ((867, 700), Some(2)),
        ((1300, 800), Some(2)),
        ((1301, 800), Some(1)),
        ((LADO_MAXIMO_OCR, 100), Some(1)),
    ];
    for ((ancho, alto), esperado) in casos {
        assert_eq!(
            factor_para(Region { ancho, alto }),
            esperado,
            "{ancho}x{alto}"
        );
    }
}

#[test]
fn should_refuse_to_enlarge_when_the_region_already_exceeds_the_ocr_limit() {
    assert_eq!(
        factor_para(Region {
            ancho: LADO_MAXIMO_OCR + 1,
            alto: 100
        }),
        None
    );
    assert_eq!(
        factor_para(Region {
            ancho: 3840,
            alto: 2160
        }),
        None
    );
}
