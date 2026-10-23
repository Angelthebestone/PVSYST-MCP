use crate::adapters::gui::ffi::Rect;

#[allow(clippy::too_many_arguments)]
const fn rect(left: i32, top: i32, right: i32, bottom: i32) -> Rect {
    Rect {
        left,
        top,
        right,
        bottom,
    }
}

use super::*;

#[test]
fn should_scale_a_line_and_its_words_when_capture_is_at_96_dpi_on_a_150_percent_screen() {
    let linea = LineaOcr {
        texto: "Potencia".into(),
        rect: rect(20, 40, 100, 60),
        palabras: vec![PalabraOcr {
            texto: "Potencia".into(),
            rect: rect(20, 40, 100, 60),
        }],
    };
    let escalada = linea.escalada(1.5);
    assert_eq!(escalada.rect, rect(30, 60, 150, 90));
    assert_eq!(escalada.palabras[0].rect, rect(30, 60, 150, 90));
}

#[test]
fn should_keep_a_rect_when_scale_is_one() {
    let r = rect(7, 11, 301, 59);
    assert_eq!(r.escalado(1.0), r);
}

#[test]
fn should_strip_accelerator_ampersand_when_single_ampersand_present() {
    assert_eq!(normalizar_etiqueta("&Guardar"), "Guardar");
    assert_eq!(
        normalizar_etiqueta("Perdidas &detalladas"),
        "Perdidas detalladas"
    );
}

#[test]
fn should_preserve_double_ampersand_as_single_when_escaped() {
    assert_eq!(normalizar_etiqueta("Inyectar && Medir"), "Inyectar & Medir");
}

#[test]
fn should_strip_trailing_colons_and_whitespace_when_present() {
    assert_eq!(normalizar_etiqueta("Potencia nominal:"), "Potencia nominal");
    assert_eq!(normalizar_etiqueta("  Azimut (°) :  "), "Azimut (°)");
}

#[test]
fn should_collapse_multiple_spaces_when_present() {
    assert_eq!(
        normalizar_etiqueta("  Voltaje   en   vacio  "),
        "Voltaje en vacio"
    );
}

#[test]
fn should_return_empty_string_when_input_is_empty() {
    assert_eq!(normalizar_etiqueta(""), "");
    assert_eq!(normalizar_etiqueta("   "), "");
    assert_eq!(normalizar_etiqueta(":::"), "");
}

#[test]
fn should_associate_left_label_when_on_same_row() {
    let control = rect(200, 100, 300, 125);
    let label = LineaOcr {
        texto: "Potencia nominal:".into(),
        rect: rect(50, 103, 190, 120),
        palabras: Vec::new(),
    };
    let resultado = asociar_etiqueta(&control, &[label]);
    assert_eq!(resultado.etiqueta.as_deref(), Some("Potencia nominal"));
    assert!(!resultado.ambiguo);
}

#[test]
fn should_associate_above_label_when_directly_above() {
    let control = rect(100, 150, 200, 175);
    let label = LineaOcr {
        texto: "Tipo de módulo:".into(),
        rect: rect(100, 125, 180, 142),
        palabras: Vec::new(),
    };
    let resultado = asociar_etiqueta(&control, &[label]);
    assert_eq!(resultado.etiqueta.as_deref(), Some("Tipo de módulo"));
    assert!(!resultado.ambiguo);
}

#[test]
fn should_prefer_nearest_label_when_both_left_and_above_exist() {
    let control = rect(200, 200, 300, 225);
    let label_cerca = LineaOcr {
        texto: "Etiqueta izquierda:".into(),
        rect: rect(120, 202, 192, 220), // 8 px a la izquierda
        palabras: Vec::new(),
    };
    let label_lejos = LineaOcr {
        texto: "Etiqueta superior:".into(),
        rect: rect(200, 140, 280, 155), // 45 px arriba
        palabras: Vec::new(),
    };
    let resultado = asociar_etiqueta(&control, &[label_cerca, label_lejos]);
    assert_eq!(resultado.etiqueta.as_deref(), Some("Etiqueta izquierda"));
    assert!(!resultado.ambiguo);
}

#[test]
fn should_report_ambiguity_when_two_left_captions_are_equally_close() {
    let control = rect(200, 200, 300, 225);
    let label_a = LineaOcr {
        texto: "Campo A:".into(),
        rect: rect(130, 195, 185, 220), // dx = 15 px, 5 px above the centre
        palabras: Vec::new(),
    };
    let label_b = LineaOcr {
        texto: "Campo B:".into(),
        rect: rect(130, 205, 185, 230), // dx = 15 px, 5 px below the centre
        palabras: Vec::new(),
    };
    let resultado = asociar_etiqueta(&control, &[label_a, label_b]);
    assert!(resultado.etiqueta.is_none());
    assert!(resultado.ambiguo);
    assert_eq!(resultado.candidatos.len(), 2);
}

#[test]
fn should_prefer_the_left_caption_when_a_heading_sits_directly_above_the_field() {
    let control = rect(200, 200, 300, 225);
    let izquierda = LineaOcr {
        texto: "Factor Uc".into(),
        rect: rect(20, 202, 150, 223), // 50 px to the left, same row
        palabras: Vec::new(),
    };
    let encabezado = LineaOcr {
        texto: "U = Uc + Uv * Vel. viento".into(),
        rect: rect(200, 170, 300, 190), // 10 px above, aligned
        palabras: Vec::new(),
    };
    let resultado = asociar_etiqueta(&control, &[encabezado, izquierda]);
    assert_eq!(resultado.etiqueta.as_deref(), Some("Factor Uc"));
    assert!(!resultado.ambiguo);
}

#[test]
fn should_return_none_when_label_is_to_the_right() {
    let control = rect(100, 100, 200, 125);
    let label = LineaOcr {
        texto: "Unidad kWp".into(),
        rect: rect(210, 103, 280, 120),
        palabras: Vec::new(),
    };
    let resultado = asociar_etiqueta(&control, &[label]);
    assert_eq!(resultado.etiqueta, None);
    assert!(!resultado.ambiguo);
}

#[test]
fn should_return_none_when_label_is_below() {
    let control = rect(100, 100, 200, 125);
    let label = LineaOcr {
        texto: "Ayuda inferior".into(),
        rect: rect(100, 140, 200, 155),
        palabras: Vec::new(),
    };
    let resultado = asociar_etiqueta(&control, &[label]);
    assert_eq!(resultado.etiqueta, None);
    assert!(!resultado.ambiguo);
}

#[test]
fn should_return_none_when_no_labels_are_within_range() {
    let control = rect(600, 600, 700, 625);
    let label = LineaOcr {
        texto: "Muy lejos:".into(),
        rect: rect(50, 50, 120, 65),
        palabras: Vec::new(),
    };
    let resultado = asociar_etiqueta(&control, &[label]);
    assert_eq!(resultado.etiqueta, None);
    assert!(!resultado.ambiguo);
}

#[test]
fn should_return_none_when_row_misalignment_exceeds_bounds() {
    let control = rect(200, 100, 300, 125);
    let label = LineaOcr {
        texto: "Otra fila:".into(),
        rect: rect(50, 160, 150, 175), // 35 px vertical gap
        palabras: Vec::new(),
    };
    let resultado = asociar_etiqueta(&control, &[label]);
    assert_eq!(resultado.etiqueta, None);
    assert!(!resultado.ambiguo);
}
