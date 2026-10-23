use super::*;

fn palabra(texto: &str, left: i32, right: i32) -> PalabraOcr {
    PalabraOcr {
        texto: texto.to_string(),
        rect: Rect {
            left,
            top: 10,
            right,
            bottom: 22,
        },
    }
}

fn linea(palabras: Vec<PalabraOcr>) -> LineaOcr {
    let caja = palabras
        .iter()
        .skip(1)
        .fold(palabras[0].rect, |a, p| a.union(&p.rect));
    LineaOcr {
        texto: String::new(),
        rect: caja,
        palabras,
    }
}

#[test]
fn should_split_line_in_two_when_a_gap_exceeds_word_height() {
    let l = linea(vec![palabra("Azimut", 0, 40), palabra("Albedo", 120, 160)]);

    let segmentos = segmentar_lineas(&[l], &[]);

    let textos: Vec<_> = segmentos.iter().map(|s| s.texto.as_str()).collect();
    assert_eq!(textos, ["Azimut", "Albedo"]);
}

#[test]
fn should_keep_one_segment_when_words_are_a_space_apart() {
    let l = linea(vec![
        palabra("Núm.", 0, 30),
        palabra("módulos", 34, 80),
        palabra("en", 84, 96),
        palabra("serie", 100, 130),
    ]);

    let segmentos = segmentar_lineas(&[l], &[]);

    assert_eq!(segmentos.len(), 1);
    assert_eq!(segmentos[0].texto, "Núm. módulos en serie");
    assert_eq!(
        segmentos[0].rect,
        Rect {
            left: 0,
            top: 10,
            right: 130,
            bottom: 22
        }
    );
}

#[test]
fn should_drop_the_words_drawn_inside_a_control_when_segmenting() {
    let l = linea(vec![palabra("Azimut", 0, 40), palabra("0", 90, 98)]);
    let campo = Rect {
        left: 80,
        top: 5,
        right: 120,
        bottom: 27,
    };

    let segmentos = segmentar_lineas(&[l], &[campo]);

    assert_eq!(segmentos.len(), 1);
    assert_eq!(segmentos[0].texto, "Azimut");
}

#[test]
fn should_return_nothing_when_every_word_belongs_to_a_control() {
    let l = linea(vec![palabra("OK", 10, 30)]);

    assert!(segmentar_lineas(
        &[l],
        &[Rect {
            left: 0,
            top: 0,
            right: 50,
            bottom: 40
        }]
    )
    .is_empty());
}
