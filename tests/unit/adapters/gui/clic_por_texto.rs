use super::*;
use crate::idioma::Idioma;
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
            idioma: Idioma::Es,
        },
        &resultado,
    );

    let error = vista.error.expect("an unconfirmed click is not a plain ok");
    assert_eq!(error.codigo, "fallo_parcial");
    assert!(error.motivo.contains("NoExiste"), "{}", error.motivo);
}

/// GUI-09: `indice:5` with one match clicked that match through UIA.
#[test]
fn should_not_click_through_uia_when_the_index_is_out_of_range() {
    use crate::adapters::gui::ffi::Rect;
    let una_caja = [Rect {
        left: 0,
        top: 0,
        right: 10,
        bottom: 10,
    }];

    let rechazo = elegir_caja(&una_caja, Some(5), "Herramientas").expect_err("index 5 of 1");
    let sin_nada = elegir_caja(&[], None, "Herramientas").expect_err("not drawn");

    assert!(!recurre_a_uia(&rechazo));
    assert!(recurre_a_uia(&sin_nada));
}

#[test]
fn should_leave_a_click_with_an_ocr_match_as_it_was() {
    let mut resultado = json!({ "accion": "clic_texto", "coincidencias": 1 });

    marcar_sin_ocr(&mut resultado, "Cerrar", 1);

    assert!(resultado.get("error").is_none());
}
