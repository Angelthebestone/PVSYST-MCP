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

/// BUG-03: `cerrado:false` used to carry `vista.estado:"ok"` and the summary
/// "Cierre de la ventana esperado".
#[test]
fn should_report_a_wait_that_timed_out_as_partial() {
    use crate::idioma::Idioma;
    use crate::vista_agente::contrato::{construir, Contexto};

    let salida = respuesta_de_espera(0, false, 2000);
    let args = json!({});
    let vista = construir(
        &Contexto {
            tool: "gui_actuar",
            accion: "esperar_cierre",
            args: &args,
            idioma: Idioma::Es,
        },
        &salida,
    );

    let error = vista.error.expect("a window left open is not a plain ok");
    assert_eq!(error.codigo, "fallo_parcial");
    assert!(error.motivo.contains("sigue abierta"), "{}", error.motivo);
}

#[test]
fn should_not_mask_an_error_that_is_not_a_missing_window() {
    let args = json!({"ventana": "x", "proceso": "PVsyst.exe"});

    let error = esperar_cierre_con(&args, |_| Err(Error::Invalid("bad reference".into())))
        .expect_err("a real error is not a trivial success");

    assert!(error.to_string().contains("bad reference"));
}
