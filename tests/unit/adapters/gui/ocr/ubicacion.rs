use super::*;

fn caja(top: i32) -> Rect {
    Rect {
        left: 10,
        top,
        right: 50,
        bottom: top + 12,
    }
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
