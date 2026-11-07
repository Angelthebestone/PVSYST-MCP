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
