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
