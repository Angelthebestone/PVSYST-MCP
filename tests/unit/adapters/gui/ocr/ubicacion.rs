use super::*;

fn palabra(texto: &str, [left, right]: [i32; 2], top: i32) -> PalabraOcr {
    PalabraOcr {
        texto: texto.to_string(),
        rect: Rect {
            left,
            top,
            right,
            bottom: top + 12,
        },
    }
}

fn linea(palabras: Vec<PalabraOcr>) -> LineaOcr {
    LineaOcr {
        texto: palabras
            .iter()
            .map(|p| p.texto.as_str())
            .collect::<Vec<_>>()
            .join(" "),
        rect: Rect::default(),
        palabras,
    }
}

fn caja(top: i32) -> Rect {
    Rect {
        left: 10,
        top,
        right: 50,
        bottom: top + 12,
    }
}

#[test]
fn should_fold_case_accents_and_punctuation_when_normalizing() {
    assert_eq!(
        plegar("Configuración del  Proyecto:"),
        "configuraciondelproyecto"
    );
}

#[test]
fn should_return_the_box_of_the_words_when_the_text_spans_two_of_them() {
    let l = linea(vec![
        palabra("Ejecutar", [10, 60], 100),
        palabra("simulación", [66, 130], 100),
    ]);

    let cajas = ubicar_texto(&[l], "ejecutar simulacion");

    assert_eq!(
        cajas,
        [Rect {
            left: 10,
            top: 100,
            right: 130,
            bottom: 112
        }]
    );
}

#[test]
fn should_return_the_box_of_three_words_when_the_text_spans_three_of_them() {
    let l = linea(vec![
        palabra("Gestión", [0, 40], 0),
        palabra("de", [46, 58], 0),
        palabra("orientaciones", [64, 140], 0),
    ]);

    let cajas = ubicar_texto(&[l], "gestion de orientaciones");

    assert_eq!(
        cajas,
        [Rect {
            left: 0,
            top: 0,
            right: 140,
            bottom: 12
        }]
    );
}

#[test]
fn should_match_inside_a_word_when_the_needle_is_a_fragment() {
    let l = linea(vec![palabra("Cliente", [5, 55], 20)]);

    assert_eq!(ubicar_texto(&[l], "lient").len(), 1);
}

#[test]
fn should_list_every_occurrence_in_reading_order_when_the_text_repeats() {
    let abajo = linea(vec![palabra("Abrir", [10, 40], 300)]);
    let arriba = linea(vec![
        palabra("Abrir", [200, 230], 50),
        palabra("sitio", [236, 270], 50),
    ]);

    let cajas = ubicar_texto(&[abajo, arriba], "abrir");

    assert_eq!(cajas.iter().map(|c| c.top).collect::<Vec<_>>(), [50, 300]);
}

#[test]
fn should_list_both_boxes_when_the_text_repeats_on_one_line() {
    let l = linea(vec![
        palabra("Abrir", [0, 30], 0),
        palabra("y", [36, 42], 0),
        palabra("Abrir", [48, 78], 0),
    ]);

    let cajas = ubicar_texto(&[l], "abrir");

    assert_eq!(cajas.iter().map(|c| c.left).collect::<Vec<_>>(), [0, 48]);
}

#[test]
fn should_find_nothing_when_the_needle_is_blank_or_absent() {
    let l = linea(vec![palabra("Guardar", [0, 40], 0)]);

    assert!(ubicar_texto(std::slice::from_ref(&l), "  ").is_empty());
    assert!(ubicar_texto(&[l], "eliminar").is_empty());
}

#[test]
fn should_skip_a_word_that_is_only_punctuation_when_it_sits_inside_the_match() {
    let l = linea(vec![
        palabra("Pérdidas", [0, 50], 0),
        palabra("-", [54, 58], 0),
        palabra("Envejecimiento", [62, 150], 0),
    ]);

    let cajas = ubicar_texto(&[l], "perdidas envejecimiento");

    assert_eq!(
        cajas,
        [Rect {
            left: 0,
            top: 0,
            right: 150,
            bottom: 12
        }]
    );
}

#[test]
fn should_pick_the_only_match_when_the_text_is_drawn_once() {
    let cajas = [caja(40)];

    assert_eq!(elegir_caja(&cajas, None, "Cliente").ok(), Some(&cajas[0]));
}

#[test]
fn should_refuse_to_guess_when_the_text_is_drawn_twice_without_index() {
    let cajas = [caja(40), caja(90)];

    let error = elegir_caja(&cajas, None, "Abrir");

    assert!(
        matches!(&error, Err(Error::Invalid(m)) if m.contains("2 veces") && m.contains("indice")),
        "{error:?}"
    );
}

#[test]
fn should_pick_the_indexed_match_when_the_text_is_drawn_twice() {
    let cajas = [caja(40), caja(90)];

    assert_eq!(elegir_caja(&cajas, Some(1), "Abrir").ok(), Some(&cajas[1]));
}

#[test]
fn should_report_not_found_when_nothing_matches() {
    let error = elegir_caja(&[], None, "Cliente");

    assert!(matches!(error, Err(Error::NotFound(_))));
}

#[test]
fn should_reject_an_index_past_the_matches() {
    let cajas = [caja(40)];

    let error = elegir_caja(&cajas, Some(3), "Cliente");

    assert!(matches!(error, Err(Error::Invalid(_))));
}
