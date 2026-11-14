//! Reading a row: its band on screen and the text OCR found in it.

use super::{banda_en_pantalla, exigir_texto, texto_en_banda};
use crate::adapters::gui::ffi::Rect;
use crate::adapters::gui::fila::geometria::Geometria;
use crate::adapters::gui::ocr::dominio::LineaOcr;

const ARBOL: Geometria = Geometria {
    clase: "TVirtualStringTree",
    anfitrion: "TProjectLoadForm",
    titulo_anfitrion: None,
    cabecera_px: 21,
    alto_fila_px: 19,
    x_px: 100,
};

fn linea(texto: &str, left: i32, centro_y: i32) -> LineaOcr {
    LineaOcr {
        texto: texto.into(),
        rect: Rect {
            left,
            top: centro_y - 5,
            right: left + 50,
            bottom: centro_y + 5,
        },
        palabras: vec![],
    }
}

#[test]
fn should_place_the_row_band_below_the_header_at_the_screen_scale() {
    // Row 1 spans 40..59 px at 96 DPI (header 21 + one row of 19); at 144 DPI it is 1.5x
    // (the bottom edge, 88.5, truncates) from the control top.
    let (arriba, abajo) = banda_en_pantalla(&ARBOL, 1, (1000, 144));

    assert_eq!((arriba, abajo), (1000 + 60, 1000 + 88));
}

#[test]
fn should_read_only_the_lines_inside_the_row_band_from_left_to_right() {
    let lineas = [
        linea("Proyecto B", 10, 100),
        linea("2026", 200, 100),
        linea("Otra fila", 10, 130),
    ];

    assert_eq!(texto_en_banda(&lineas, (90, 110)), "Proyecto B 2026");
}

#[test]
fn should_accept_a_row_that_holds_the_requested_text_ignoring_case_and_accents() {
    exigir_texto("Planta Fotovoltaica  QA41_Sol", "qa41_sol", 0).expect("same project");
    exigir_texto("Cañón", "canon", 2).expect("accents folded");
}

/// 4.1-10: row 0 was highlighted, but it was another project.
#[test]
fn should_refuse_a_highlighted_row_that_holds_another_element() {
    let error = exigir_texto("Proyecto Viejo", "QA41_Sol", 0)
        .expect_err("another project")
        .to_string();

    assert!(error.contains("Proyecto Viejo"), "{error}");
    assert!(error.contains("QA41_Sol"), "{error}");
    assert!(error.contains("not the requested element"), "{error}");
}
