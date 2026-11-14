//! A window betrays its language by the captions of its buttons.

use super::de_ventanas;
use crate::adapters::gui::rotulos::dialogos::{CANCELAR, GUARDAR};
use crate::adapters::gui::rotulos::pvsyst::BASES_DE_DATOS;
use crate::adapters::gui::rotulos::IdiomaGui;
use crate::adapters::gui::ventana_falsa::{Hijo, VentanaFalsa};

#[test]
fn should_tell_the_language_of_a_form_from_its_buttons() {
    for idioma in IdiomaGui::TODOS {
        let form = VentanaFalsa::crear(
            "THomeForm",
            "PVsyst",
            vec![
                Hijo::de("TPngBitBtn", (10, 10, 150, 40))
                    .con_texto(BASES_DE_DATOS.en_idioma(idioma)),
                Hijo::de("TPngBitBtn", (10, 50, 150, 80)).con_texto(CANCELAR.en_idioma(idioma)),
                // A hidden page speaks too: the language does not depend on the tab shown.
                Hijo::de("TPngBitBtn", (10, 90, 150, 120))
                    .con_texto(GUARDAR.en_idioma(idioma))
                    .oculto(),
            ],
        );

        assert_eq!(de_ventanas([form.hwnd(0)]), Some(idioma));
    }
}

#[test]
fn should_not_guess_the_language_of_a_form_with_no_known_caption() {
    let form = VentanaFalsa::crear(
        "TUnknownForm",
        "Whatever",
        vec![Hijo::de("TEdit", (10, 10, 150, 40)).con_texto("OK")],
    );

    assert_eq!(de_ventanas([form.hwnd(0)]), None);
}
