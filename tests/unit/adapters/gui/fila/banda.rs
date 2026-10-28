use crate::adapters::gui::captura::ImagenRgb;
use crate::adapters::gui::fila::banda::{filas_seleccionadas, Capturas};
use crate::adapters::gui::fila::geometria::Geometria;

const LISTA: Geometria = Geometria {
    clase: "lista",
    anfitrion: "TForm",
    titulo_anfitrion: None,
    cabecera_px: 2,
    alto_fila_px: 13,
    x_px: 100,
};
const ANCHO: usize = 40;
const FILAS: usize = 6;
const BLANCO: [u8; 3] = [255, 255, 255];
const AZUL: [u8; 3] = [153, 180, 209];
const ROJO: [u8; 3] = [220, 40, 40];

fn pintar_linea(rgb: &mut [u8], (y, color): (usize, [u8; 3])) {
    for x in 0..ANCHO {
        let ini = (y * ANCHO + x) * 3;
        rgb[ini..ini + 3].copy_from_slice(&color);
    }
}

fn lienzo(filas: usize) -> Vec<u8> {
    BLANCO.repeat(ANCHO * (2 + 13 * filas))
}

/// White list of `filas` slots with each `(fila, color)` painted over its whole row.
fn imagen_de(filas: usize, pintadas: &[(usize, [u8; 3])]) -> ImagenRgb {
    let mut rgb = lienzo(filas);
    for (fila, color) in pintadas {
        for y in 2 + fila * 13..2 + (fila + 1) * 13 {
            pintar_linea(&mut rgb, (y, *color));
        }
    }
    ImagenRgb {
        rgb,
        ancho: ANCHO,
        alto: 2 + 13 * filas,
    }
}

fn imagen(seleccion: &[usize]) -> ImagenRgb {
    let pintadas: Vec<_> = seleccion.iter().map(|f| (*f, AZUL)).collect();
    imagen_de(FILAS, &pintadas)
}

fn seleccion(antes: &ImagenRgb, despues: &ImagenRgb) -> Vec<usize> {
    filas_seleccionadas(&Capturas { antes, despues }, &LISTA)
}

#[test]
fn should_read_the_row_the_click_selected_when_the_selection_moved() {
    assert_eq!(seleccion(&imagen(&[0]), &imagen(&[3])), vec![3]);
}

#[test]
fn should_read_row_zero_when_the_band_touches_the_border() {
    assert_eq!(seleccion(&imagen(&[]), &imagen(&[0])), vec![0]);
}

#[test]
fn should_read_the_last_row_when_the_band_is_at_the_bottom() {
    assert_eq!(seleccion(&imagen(&[2]), &imagen(&[5])), vec![5]);
}

#[test]
fn should_read_the_row_when_it_was_already_selected_and_nothing_changed() {
    assert_eq!(seleccion(&imagen(&[4]), &imagen(&[4])), vec![4]);
}

#[test]
fn should_read_nothing_when_no_row_is_highlighted() {
    assert!(seleccion(&imagen(&[]), &imagen(&[])).is_empty());
}

#[test]
fn should_report_every_band_when_two_rows_are_highlighted_and_nothing_changed() {
    assert_eq!(seleccion(&imagen(&[1, 4]), &imagen(&[1, 4])), vec![1, 4]);
}

#[test]
fn should_ignore_a_status_coloured_row_when_the_click_selected_another_one() {
    let antes = imagen_de(FILAS, &[(1, ROJO), (0, AZUL)]);
    let despues = imagen_de(FILAS, &[(1, ROJO), (3, AZUL)]);
    assert_eq!(seleccion(&antes, &despues), vec![3]);
}

#[test]
fn should_not_take_a_status_coloured_row_for_the_selection_when_the_click_changed_nothing_else() {
    let rojo = imagen_de(FILAS, &[(1, ROJO), (3, AZUL)]);
    assert_eq!(seleccion(&rojo, &rojo), vec![1, 3]);
}

#[test]
fn should_read_a_single_item_list_when_the_rest_of_the_slots_are_empty() {
    assert_eq!(seleccion(&imagen(&[]), &imagen(&[0])), vec![0]);
}

#[test]
fn should_ignore_text_when_only_a_few_pixels_of_the_row_differ() {
    let mut despues = imagen(&[]);
    for x in 0..ANCHO / 4 {
        let ini = ((2 + 13 * 2 + 6) * ANCHO + x) * 3;
        despues.rgb[ini..ini + 3].copy_from_slice(&[0, 0, 0]);
    }
    assert!(seleccion(&imagen(&[]), &despues).is_empty());
}

#[test]
fn should_read_nothing_when_the_capture_is_empty() {
    let vacia = ImagenRgb {
        rgb: Vec::new(),
        ancho: 0,
        alto: 0,
    };
    assert!(seleccion(&vacia, &vacia).is_empty());
}
