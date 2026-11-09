use super::*;
use crate::vista_agente::contrato::{construir, Contexto};

/// BUG-03: `coincidencias:0` used to carry `vista.estado:"ok"`.
#[test]
fn should_report_a_click_without_ocr_match_as_partial() {
    let mut resultado = json!({ "accion": "clic_texto", "coincidencias": 0 });
    marcar_sin_ocr(&mut resultado, "NoExiste", 0);
    let args = json!({});

    let vista = construir(
        &Contexto {
            tool: "gui_actuar",
            accion: "clic_texto",
            args: &args,
        },
        &resultado,
    );

    let error = vista.error.expect("an unconfirmed click is not a plain ok");
    assert_eq!(error.codigo, "fallo_parcial");
    assert!(error.motivo.contains("NoExiste"), "{}", error.motivo);
}

#[test]
fn should_leave_a_click_with_an_ocr_match_as_it_was() {
    let mut resultado = json!({ "accion": "clic_texto", "coincidencias": 1 });

    marcar_sin_ocr(&mut resultado, "Cerrar", 1);

    assert!(resultado.get("error").is_none());
}
