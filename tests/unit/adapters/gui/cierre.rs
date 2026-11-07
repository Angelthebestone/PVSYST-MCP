//! What a close leaves behind: a parent selector can reappear as a modal, and
//! the answer must name it with the exact class and the next `cerrar`.

use super::{reabiertas, VentanaViva};

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
