use super::*;
use crate::idioma::Idioma;
use crate::progreso::SeguimientoTarea;
use crate::server::tools::{call_seguido, Solicitud};
use serde_json::json;

/// Tool name and arguments of one call.
type Llamada<'a> = (&'a str, &'a Value);

fn con_solicitud<T>(
    paths: &Paths,
    (name, args): Llamada<'_>,
    uso: impl FnOnce(&Solicitud<'_>) -> T,
) -> T {
    let seguimiento = SeguimientoTarea::inerte();
    uso(&Solicitud {
        paths,
        name,
        args,
        seguimiento: &seguimiento,
    })
}

fn call(paths: &Paths, name: &str, args: &Value) -> crate::error::Result<Value> {
    con_solicitud(paths, (name, args), call_seguido)
}

fn vista_del_error(paths: &Paths, llamada: Llamada<'_>, error: &Error) -> Option<Value> {
    con_solicitud(paths, llamada, |s| vista_de_error(s, error))
}

/// Every `(tool, accion)` the registry declares; tools without a discriminator use `""`.
fn acciones_declaradas() -> Vec<(&'static str, &'static str)> {
    REGISTRO
        .iter()
        .flat_map(|h| match &h.discriminador {
            Some(d) => d.valores.iter().map(|v| (h.nombre, *v)).collect(),
            None => vec![(h.nombre, "")],
        })
        .collect()
}

fn contexto<'a>(tool: &'a str, accion: &'a str, args: &'a Value) -> Contexto<'a> {
    Contexto {
        tool,
        accion,
        args,
        idioma: Idioma::Es,
    }
}

pub(super) fn con_vista((tool, accion): (&str, &str), args: &Value, salida: Value) -> Value {
    let mut v = salida;
    anadir_vista(
        &Paths::para_test(std::env::temp_dir()),
        &contexto(tool, accion, args),
        &mut v,
    );
    v["vista"].clone()
}

#[test]
fn should_give_an_executable_next_step_when_summarizing_any_action() {
    let paths = Paths::para_test(std::env::temp_dir());
    for (tool, accion) in acciones_declaradas() {
        let vista = con_vista((tool, accion), &json!({}), json!({}));
        assert_eq!(vista["estado"], "ok", "{tool} {accion}");
        assert!(
            vista["resumen"].as_str().is_some_and(|r| !r.is_empty()),
            "{tool} {accion}: sin resumen"
        );
        let pasos = vista["siguiente"].as_array().cloned().unwrap_or_default();
        assert!(!pasos.is_empty(), "{tool} {accion}: sin siguiente");
        for paso in pasos {
            let destino = paso["tool"].as_str().unwrap_or_default();
            assert!(
                acepta_paso(&paths, destino, &paso["argumentos"]),
                "{tool} {accion}: el paso {paso} no es ejecutable"
            );
        }
    }
}

#[test]
fn should_mark_the_flow_partial_when_some_steps_worked() {
    let salida = json!({
        "ok": false,
        "pasos": [{ "ok": true, "paso": "sitio" }, { "ok": false, "paso": "simulacion" }],
        "errores": ["paso 'simulacion' fallido"],
        "proyecto": "Sistema_8kWp",
    });
    let vista = con_vista(("disenar_sistema", ""), &json!({}), salida);
    assert_eq!(vista["estado"], "parcial");
    assert_eq!(vista["error"]["motivo"], "paso 'simulacion' fallido");
    assert_eq!(
        vista["siguiente"][0]["tool"], "entorno",
        "diagnose, not results"
    );
}

#[test]
fn should_label_the_origin_when_values_are_computed_by_the_mcp() {
    let salida = json!({ "origen": "aproximacion_mcp", "energia_kwh": 1200.5 });
    let vista = con_vista(("sitio", "geometria_solar"), &json!({}), salida);
    assert_eq!(vista["origen"], "aproximacion_mcp");
    assert_eq!(vista["campos"][0]["unidad"], "kWh");
}

#[test]
fn should_cap_the_view_fields_without_cutting_the_answer_when_the_form_has_many() {
    let campos: Vec<Value> = (0..60)
        .map(|i| json!({ "clave": format!("campo_{i}"), "valor": "1", "unidad": "W", "editable": true }))
        .collect();
    let mut respuesta = json!({ "campos": campos });
    anadir_vista(
        &Paths::para_test(std::env::temp_dir()),
        &contexto("gui_observar", "vista", &json!({})),
        &mut respuesta,
    );
    let vista = &respuesta["vista"];
    assert_eq!(vista["campos"].as_array().map(Vec::len), Some(12));
    assert_eq!(vista["campos"][0]["editable"], true);
    assert_eq!(vista["recortado"][0]["clave"], "vista.campos");
    assert_eq!(vista["recortado"][0]["total"], 60);
    assert_eq!(respuesta["campos"].as_array().map(Vec::len), Some(60));
}

#[test]
fn should_report_code_cause_and_valid_keys_when_a_call_fails() {
    let paths = Paths::para_test(std::env::temp_dir());
    let args = json!({ "accion": "doctor" });
    let error = call(&paths, "proyecto", &args).expect_err("doctor needs a project");
    let vista = vista_del_error(&paths, ("proyecto", &args), &error).expect("known tool");
    assert_eq!(vista["estado"], "error");
    assert_eq!(vista["error"]["codigo"], "missing_arg");
    assert_eq!(vista["error"]["claves_validas"], json!(["proyecto"]));
    assert!(vista["error"]["que_hacer"]
        .as_str()
        .is_some_and(|t| !t.is_empty()));
    assert_eq!(vista["siguiente"][0]["tool"], "entorno");
}
