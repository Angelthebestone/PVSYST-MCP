use crate::adapters::gui::captura::ImagenRgb;
use crate::adapters::gui::fila::banda::filas_resaltadas;
use crate::adapters::gui::fila::geometria::Geometria;

const LISTA: Geometria = Geometria {
    clase: "lista",
    cabecera_px: 2,
    alto_fila_px: 13,
    x_px: 100,
};
const ANCHO: usize = 40;
const ALTO: usize = 2 + 13 * 6;
const BLANCO: [u8; 3] = [255, 255, 255];
const AZUL: [u8; 3] = [153, 180, 209];

fn pintar_linea(rgb: &mut [u8], y: usize) {
    for x in 0..ANCHO {
        let ini = (y * ANCHO + x) * 3;
        rgb[ini..ini + 3].copy_from_slice(&AZUL);
    }
}

/// White list with the bands of `resaltadas` painted over their whole row.
fn imagen(resaltadas: &[usize]) -> ImagenRgb {
    let mut rgb = BLANCO.repeat(ANCHO * ALTO);
    for fila in resaltadas {
        for y in 2 + fila * 13..2 + (fila + 1) * 13 {
            pintar_linea(&mut rgb, y);
        }
    }
    ImagenRgb {
        rgb,
        ancho: ANCHO,
        alto: ALTO,
    }
}

#[test]
fn should_read_the_row_when_one_band_is_painted() {
    assert_eq!(filas_resaltadas(&imagen(&[3]), &LISTA), vec![3]);
}

#[test]
fn should_read_row_zero_when_the_band_touches_the_border() {
    assert_eq!(filas_resaltadas(&imagen(&[0]), &LISTA), vec![0]);
}

#[test]
fn should_read_the_last_row_when_the_band_is_at_the_bottom() {
    assert_eq!(filas_resaltadas(&imagen(&[5]), &LISTA), vec![5]);
}

#[test]
fn should_read_nothing_when_no_row_is_highlighted() {
    assert!(filas_resaltadas(&imagen(&[]), &LISTA).is_empty());
}

#[test]
fn should_report_every_band_when_two_rows_are_highlighted() {
    assert_eq!(filas_resaltadas(&imagen(&[1, 4]), &LISTA), vec![1, 4]);
}

#[test]
fn should_ignore_text_when_only_a_few_pixels_of_the_row_differ() {
    let mut img = imagen(&[]);
    for x in 0..ANCHO / 4 {
        let ini = ((2 + 13 * 2 + 6) * ANCHO + x) * 3;
        img.rgb[ini..ini + 3].copy_from_slice(&[0, 0, 0]);
    }
    assert!(filas_resaltadas(&img, &LISTA).is_empty());
}

#[test]
fn should_read_nothing_when_the_capture_is_empty() {
    let vacia = ImagenRgb {
        rgb: Vec::new(),
        ancho: 0,
        alto: 0,
    };
    assert!(filas_resaltadas(&vacia, &LISTA).is_empty());
}
