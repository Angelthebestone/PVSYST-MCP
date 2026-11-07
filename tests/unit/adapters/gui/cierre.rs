//! What a close leaves behind: a parent selector can reappear as a modal, and
//! the answer must name it with the exact class and the next `cerrar`.

use super::{aviso, reabiertas, VentanaViva};
use serde_json::json;

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

#[test]
fn should_not_report_the_home_screen_as_a_reopened_form() {
    let antes = [viva(0x20, "TPVModuleForm")];
    let despues = [viva(0x10, "THomeForm"), viva(0x20, "TPVModuleForm")];

    assert!(reabiertas(&antes, &despues, 0x20).is_empty());
}

#[test]
fn should_suggest_the_next_cerrar_when_a_form_reopens() {
    let reabiertas = [json!({"hwnd": "0x30", "class": "TChoixObjetPVForm"})];

    let aviso = aviso(&reabiertas).expect("a warning");

    assert!(aviso.contains("TChoixObjetPVForm"), "{aviso}");
    assert!(aviso.contains("accion:cerrar"), "{aviso}");
    assert!(aviso.contains("0x30"), "{aviso}");
}

#[test]
fn should_stay_quiet_when_nothing_reopens() {
    assert_eq!(aviso(&[]), None);
}
