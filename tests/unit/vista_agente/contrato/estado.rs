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
fn should_report_an_error_when_the_cli_timed_out() {
    let vista = vista_de("simular", "ejecutar", &json!({"timed_out": true}));
    assert_eq!(vista.estado, Estado::Error);
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
fn should_report_ok_when_the_cli_succeeded() {
    let salida = json!({"success": true, "exit_code": 0});
    assert_eq!(vista_de("pvsyst_cli", "", &salida).estado, Estado::Ok);
}

#[test]
fn should_not_list_a_cut_when_the_list_is_as_long_as_its_total() {
    let salida = json!({ "colisiones": [], "colisiones_total": 0 });
    let vista = vista_de("gui_actuar", "escena", &salida);
    assert!(vista.recortado.is_empty(), "{:?}", vista.recortado);
}

#[test]
fn should_list_the_cut_when_the_list_is_shorter_than_its_total() {
    let salida = json!({ "colisiones": [1, 2, 3], "colisiones_total": 1770 });
    let vista = vista_de("gui_actuar", "escena", &salida);
    assert_eq!(vista.recortado.len(), 1);
    assert_eq!(vista.recortado.first().map(|r| r.total), Some(1770));
}

fn primer_paso((tool, accion): (&str, &str), args: &Value, salida: &Value) -> Option<Value> {
    let vista = construir(&Contexto { tool, accion, args }, salida);
    let paso = vista.siguiente.first()?;
    Some(json!({ "tool": paso.tool, "argumentos": paso.argumentos }))
}

#[test]
fn should_list_the_components_of_the_kind_created_when_it_is_an_inverter() {
    let args = json!({ "accion": "crear_desde_ficha", "tipo": "OND", "nombre_nuevo": "X" });
    let paso = primer_paso(("componente", "crear_desde_ficha"), &args, &json!({})).expect("a step");
    assert_eq!(paso["tool"], "listar");
    assert_eq!(
        paso["argumentos"],
        json!({ "que": "componentes", "tipo": "OND" })
    );
}

#[test]
fn should_take_the_kind_from_the_answer_when_the_request_has_none() {
    let args = json!({ "accion": "crear", "plantilla": "X.OND" });
    let paso =
        primer_paso(("componente", "crear"), &args, &json!({ "kind": "OND" })).expect("step");
    assert_eq!(paso["argumentos"]["tipo"], "OND");
}

#[test]
fn should_fall_back_to_the_module_list_when_the_kind_is_unknown() {
    let args = json!({ "accion": "crear" });
    let paso = primer_paso(("componente", "crear"), &args, &json!({})).expect("a step");
    assert_eq!(paso["argumentos"], json!({ "que": "modulos" }));
}
