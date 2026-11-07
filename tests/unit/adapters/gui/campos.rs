//! Parsing of the `campo` selector used by the GUI field actions.

use super::*;
use serde_json::json;

#[test]
fn should_treat_an_already_closed_window_as_a_trivial_success() {
    let args = json!({"ventana": "TPVModuleForm", "proceso": "PVsyst.exe"});

    let respuesta = esperar_cierre_con(&args, |_| {
        Err(Error::NotFound("Window 'TPVModuleForm' not found".into()))
    })
    .expect("an already closed window is a success");

    assert_eq!(respuesta["cerrado"], true);
    assert_eq!(respuesta["ya_cerrada"], true);
    assert!(respuesta["aviso"]
        .as_str()
        .unwrap_or_default()
        .contains("already closed"));
}

#[test]
fn should_not_mask_an_error_that_is_not_a_missing_window() {
    let args = json!({"ventana": "x", "proceso": "PVsyst.exe"});

    let error = esperar_cierre_con(&args, |_| Err(Error::Invalid("bad reference".into())))
        .expect_err("a real error is not a trivial success");

    assert!(error.to_string().contains("bad reference"));
}
