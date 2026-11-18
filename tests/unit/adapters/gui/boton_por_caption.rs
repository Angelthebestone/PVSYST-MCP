use super::*;
use crate::adapters::gui::ventana_falsa::{Hijo, VentanaFalsa};
use crate::adapters::gui::ventanas::rect_of;

/// `TChoixObjetPVForm` names «Abrir» a combo box and a button: the click must
/// press the button, not whichever of the two it met first.
fn form_con_combo_y_boton_abrir() -> VentanaFalsa {
    VentanaFalsa::crear(
        "TChoixObjetPVForm",
        "Eligiendo un sitio geográfico",
        vec![
            Hijo::de("TComboBox", (20, 40, 180, 64)).con_texto("Abrir"),
            Hijo::de("TButton", (200, 40, 280, 64)).con_texto("Abrir"),
        ],
    )
}

/// Screen centre of child `indice` of a fake form (0 is the form).
fn centro(ventana: &VentanaFalsa, indice: usize) -> (i32, i32) {
    let r = rect_of(ventana.hwnd(indice));
    ((r.left + r.right) / 2, (r.top + r.bottom) / 2)
}

#[test]
fn should_click_the_button_when_a_combo_shows_the_same_caption() {
    let form = form_con_combo_y_boton_abrir();

    let objetivo = boton_con_caption(form.hwnd(0), "Abrir", Idioma::Es)
        .expect("the button carries the caption");

    assert_eq!(objetivo, Some(centro(&form, 2)));
}

/// GUI-07 for `clic_texto`: a disabled button swallows the click, and the answer used to be
/// a dead `verificado:false` that hid the caption the caller asked about.
#[test]
fn should_refuse_a_disabled_button_whose_caption_is_the_text() {
    let form = VentanaFalsa::crear(
        "TChoixObjetPVForm",
        "Eligiendo un objeto",
        vec![
            Hijo::de("TComboBox", (20, 40, 180, 64)).con_texto("Abrir"),
            Hijo::de("TButton", (200, 40, 280, 64))
                .con_texto("Abrir")
                .deshabilitado(),
        ],
    );

    let error = boton_con_caption(form.hwnd(0), "Abrir", Idioma::Es)
        .expect_err("a disabled button swallows the click");

    let Error::Gui(motivo) = error else {
        panic!("a disabled button is not a bad argument: {error:?}");
    };
    assert!(motivo.contains("TButton"), "{motivo}");
    assert!(motivo.contains(&form.referencia(2)), "{motivo}");
    let (x, y) = centro(&form, 2);
    assert!(motivo.contains(&format!("({x}, {y})")), "{motivo}");
}

/// The same caption on a disabled and an enabled button: the enabled one is the place a
/// person means, so the refusal must not fire.
#[test]
fn should_click_the_enabled_button_when_a_disabled_one_shares_the_caption() {
    let form = VentanaFalsa::crear(
        "TSystemegridForm",
        "Sistema",
        vec![
            Hijo::de("TButton", (20, 40, 100, 64))
                .con_texto("Abrir")
                .deshabilitado(),
            Hijo::de("TBitBtn", (200, 40, 280, 64)).con_texto("Abrir"),
        ],
    );

    let objetivo = boton_con_caption(form.hwnd(0), "Abrir", Idioma::Es)
        .expect("the enabled button carries the caption");

    assert_eq!(objetivo, Some(centro(&form, 2)));
}

/// Two buttons of `TSystemegridForm` are captioned «Abrir» (measured): pressing
/// one of them would be a guess, so the call names both instead.
#[test]
fn should_name_every_button_that_shares_the_caption_instead_of_guessing() {
    let form = VentanaFalsa::crear(
        "TSystemegridForm",
        "Sistema",
        vec![
            Hijo::de("TButton", (20, 40, 100, 64)).con_texto("Abrir"),
            Hijo::de("TBitBtn", (200, 40, 280, 64)).con_texto("&Abrir"),
        ],
    );

    let error = boton_con_caption(form.hwnd(0), "Abrir", Idioma::Es)
        .expect_err("two buttons carry the caption");

    let Error::Gui(motivo) = error else {
        panic!("a caption shared by two buttons is not a bad argument: {error:?}");
    };
    assert!(
        motivo.contains("TButton") && motivo.contains("TBitBtn"),
        "{motivo}"
    );
    for indice in [1, 2] {
        let (x, y) = centro(&form, indice);
        assert!(motivo.contains(&format!("({x}, {y})")), "{motivo}");
    }
}
