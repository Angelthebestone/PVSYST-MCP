//! What a close leaves behind: a parent selector can reappear as a modal, and
//! the answer must name it with the exact class and the next `cerrar`.

use super::{reabiertas, sigue_abierta, VentanaViva};

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

    let texto = sigue_abierta("TPerspectiveForm", Some((0xE09D0, botones)));

    assert!(texto.contains("TMessageForm 0xE09D0"), "{texto}");
    assert!(texto.contains("'No' (0xE09EA)"), "{texto}");
}
