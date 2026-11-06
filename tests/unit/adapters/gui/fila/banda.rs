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
