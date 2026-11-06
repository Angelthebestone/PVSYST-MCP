use super::construccion::{construir, Contexto};
use super::tipos::{Estado, Vista};
use serde_json::{json, Value};

fn vista_de(tool: &str, accion: &str, salida: &Value) -> Vista {
    let args = json!({});
    construir(
        &Contexto {
            tool,
            accion,
            args: &args,
        },
        salida,
    )
}

#[test]
fn should_report_an_error_when_the_cli_answers_success_false() {
    let salida = json!({
        "success": false,
        "detected_error": "Proceso terminó con código de error 1",
        "exit_code": 1,
    });
    let vista = vista_de("pvsyst_cli", "", &salida);
    assert_eq!(vista.estado, Estado::Error);
    let error = vista.error.expect("error block");
    assert_eq!(error.codigo, "fallo_cli");
    assert!(
        error.motivo.contains("código de error 1"),
        "{}",
        error.motivo
    );
}

#[test]
fn should_report_partial_when_a_listing_says_it_is_incomplete() {
    let salida = json!({"completo": false, "error": "más de 30 menús", "menus": []});
    let vista = vista_de("gui_observar", "menus", &salida);
    assert_eq!(vista.estado, Estado::Parcial);
    let error = vista.error.expect("error block");
    assert_eq!(error.codigo, "fallo_parcial");
    assert!(error.motivo.contains("más de 30 menús"), "{}", error.motivo);
}

#[test]
fn should_warn_and_not_fail_when_the_run_produced_its_output_despite_a_process_error() {
    let salida = json!({
        "success": true,
        "confirmed_by_output_file": true,
        "detected_error": "exit 1",
    });
    let vista = vista_de("simular", "ejecutar", &salida);
    assert_eq!(vista.estado, Estado::Ok);
    assert!(
        vista.avisos.iter().any(|a| a.contains("exit 1")),
        "{:?}",
        vista.avisos
    );
}

#[test]
fn should_be_partial_when_a_scene_answer_counts_fields_that_differ() {
    let salida = json!({ "creado": true, "verificado": false, "campos_que_difieren": 2 });

    let vista = vista_de("gui_actuar", "escena", &salida);

    assert_eq!(vista.estado, Estado::Parcial);
}
