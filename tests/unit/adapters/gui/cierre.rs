//! What a close leaves behind: a parent selector can reappear as a modal, and
//! the answer must name it with the exact class and the next `cerrar`; a close
//! that stopped at the discard question may answer it, and only it.

use super::{descarte, reabiertas, sigue_abierta, Descarte, VentanaViva};
use crate::adapters::gui::acciones::CLASE_MENSAJE;
use crate::adapters::gui::rotulos::dialogos::{DESCARTAR_CAMBIOS, SI};
use crate::adapters::gui::rotulos::IdiomaGui;

fn viva(h: isize, clase: &str) -> VentanaViva {
    VentanaViva {
        h,
        clase: clase.to_string(),
        titulo: String::new(),
    }
}

#[test]
fn should_report_the_parent_that_reopens_after_closing_an_editor() {
    let antes = [viva(0x10, "THomeForm"), viva(0x20, "TPVModuleForm")];
    let despues = [
        viva(0x10, "THomeForm"),
        viva(0x20, "TPVModuleForm"),
        viva(0x30, "TChoixObjetPVForm"),
    ];

    let reabiertas = reabiertas(&antes, &despues, 0x20);

    assert_eq!(reabiertas.len(), 1);
    assert_eq!(reabiertas[0]["class"], "TChoixObjetPVForm");
    assert_eq!(reabiertas[0]["hwnd"], "0x30");
}

/// GUI-14: the failed close named neither the confirmation nor its buttons.
#[test]
fn should_name_the_confirmation_and_its_buttons_when_a_close_leaves_the_window_open() {
    let botones = vec![(0xE09E0, "Cancelar".into()), (0xE09EA, "No".into())];

    let texto = sigue_abierta(
        "TPerspectiveForm",
        Some((0xE09D0, botones)),
        crate::idioma::Idioma::Es,
    );

    assert!(texto.contains("TMessageForm 0xE09D0"), "{texto}");
    assert!(texto.contains("'No' (0xE09EA)"), "{texto}");
}

/// Walking ~60 windows cost two calls each: a close with changes stopped at
/// «¿Quiere cancelar las modificaciones?» and the question had to be answered
/// apart. `descartar_cambios` may answer that question in the same call, and no
/// other: the 3D editor asks «¿Quiere mantener sus modificaciones en el objeto?»
/// and its «Sí» adds the object, so a wording that is merely close must not
/// count; a GUI in a language with no known wording cannot be told apart at all.
#[test]
fn should_answer_the_discard_question_and_no_other_dialog() {
    let pregunta = |idioma: IdiomaGui| DESCARTAR_CAMBIOS.en_idioma(idioma).to_string();
    for idioma in IdiomaGui::TODOS {
        assert_eq!(
            descarte(CLASE_MENSAJE, &pregunta(idioma), Some(idioma)),
            Descarte::Descartar { boton: SI },
            "{idioma:?}"
        );
    }
    // The wording is looked for in the language the GUI is in, not in any.
    assert_eq!(
        descarte(CLASE_MENSAJE, &pregunta(IdiomaGui::Es), Some(IdiomaGui::En)),
        Descarte::OtroDialogo
    );
    let mantener = "¿Quiere mantener sus modificaciones en el objeto?";
    assert_eq!(
        descarte(CLASE_MENSAJE, mantener, Some(IdiomaGui::Es)),
        Descarte::OtroDialogo
    );
    let aviso = "Accès au fichier refusé";
    assert_eq!(
        descarte(CLASE_MENSAJE, aviso, Some(IdiomaGui::Es)),
        Descarte::OtroDialogo
    );
    assert_eq!(
        descarte("TDEMOForm", &pregunta(IdiomaGui::Es), Some(IdiomaGui::Es)),
        Descarte::OtroDialogo
    );
    // An Italian GUI shows the question in wording this server does not hold.
    assert_eq!(
        descarte(CLASE_MENSAJE, "Vuoi annullare le modifiche?", None),
        Descarte::IdiomaDesconocido
    );
}
