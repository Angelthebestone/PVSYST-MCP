use super::*;
use crate::progreso::SeguimientoTarea;
use crate::server::tools::{call_seguido, recorte, Solicitud};
use crate::vista_agente::contrato::catalogo;
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

pub(super) fn vista_del_error(paths: &Paths, llamada: Llamada<'_>, error: &Error) -> Option<Value> {
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
    Contexto { tool, accion, args }
}

pub(super) fn con_vista((tool, accion): (&str, &str), args: &Value, salida: Value) -> Value {
    let mut v = salida;
    anadir_vista(&Paths::resolve(), &contexto(tool, accion, args), &mut v);
    v["vista"].clone()
}

#[test]
fn should_describe_every_declared_tool_action_in_the_catalogue() {
    let sin_entrada: Vec<_> = acciones_declaradas()
        .into_iter()
        .filter(|(tool, accion)| catalogo::buscar(tool, accion).is_none())
        .collect();
    assert!(sin_entrada.is_empty(), "sin entrada: {sin_entrada:?}");
}

#[test]
fn should_not_keep_catalogue_entries_for_actions_that_no_longer_exist() {
    let declaradas = acciones_declaradas();
    let huerfanas: Vec<_> = catalogo::todas()
        .filter(|e| !declaradas.contains(&(e.tool, e.accion)))
        .map(|e| (e.tool, e.accion))
        .collect();
    assert!(huerfanas.is_empty(), "entradas huérfanas: {huerfanas:?}");
}

#[test]
fn should_give_state_summary_and_an_executable_next_step_for_every_action() {
    let paths = Paths::resolve();
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
fn should_take_the_project_from_the_request_when_suggesting_the_next_step() {
    let args = json!({ "proyecto": "Casa" });
    let vista = con_vista(("proyecto", "dimensionar"), &args, json!({ "ok": true }));
    assert_eq!(vista["siguiente"][0]["tool"], "proyecto");
    assert_eq!(vista["siguiente"][0]["argumentos"]["proyecto"], "Casa");
    assert_eq!(vista["siguiente"][0]["argumentos"]["accion"], "doctor");
}

#[test]
fn should_take_the_project_from_the_answer_when_the_request_has_none() {
    let salida = json!({ "proyecto": "Sistema_8kWp", "ok": true });
    let vista = con_vista(("disenar_sistema", ""), &json!({}), salida);
    assert_eq!(
        vista["siguiente"][0]["argumentos"],
        json!({ "que": "perdidas", "proyecto": "Sistema_8kWp" })
    );
}

#[test]
fn should_drop_a_next_step_whose_required_value_is_unknown() {
    let vista = con_vista(("proyecto", "dimensionar"), &json!({}), json!({}));
    let herramientas: Vec<_> = vista["siguiente"]
        .as_array()
        .into_iter()
        .flatten()
        .map(|p| p["tool"].clone())
        .collect();
    assert_eq!(herramientas, [json!("listar")], "doctor needs a project");
}

#[test]
fn should_carry_the_workspace_of_the_request_into_the_next_steps() {
    let args = json!({ "workspace": "C:/ws" });
    let vista = con_vista(("listar", "proyectos"), &args, json!({}));
    assert_eq!(vista["siguiente"][0]["argumentos"]["workspace"], "C:/ws");
}

#[test]
fn should_mark_a_failed_flow_as_partial_when_a_step_worked() {
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
fn should_mark_a_flow_as_error_when_no_step_worked() {
    let salida = json!({ "ok": false, "pasos": [{ "ok": false }] });
    let vista = con_vista(("disenar_sistema", ""), &json!({}), salida);
    assert_eq!(vista["estado"], "error");
}

#[test]
fn should_label_values_of_an_approximation_as_computed_by_the_mcp() {
    let salida = json!({ "origen": "aproximacion_mcp", "energia_kwh": 1200.5 });
    let vista = con_vista(("sitio", "geometria_solar"), &json!({}), salida);
    assert_eq!(vista["origen"], "aproximacion_mcp");
    assert_eq!(vista["campos"][0]["unidad"], "kWh");
}

#[test]
fn should_report_what_the_detail_level_cut_and_how_to_ask_for_the_rest() {
    let mut v = json!({ "items": (0..20).collect::<Vec<u32>>() });
    let args = json!({});
    let respuesta = recorte::Respuesta {
        herramienta: "listar",
        accion: "archivos",
        ms: 1,
        ignorados: Vec::new(),
    };
    recorte::decorar(&mut v, &args, respuesta);
    anadir_vista(
        &Paths::resolve(),
        &contexto("listar", "archivos", &args),
        &mut v,
    );
    assert_eq!(
        v["vista"]["recortado"],
        json!([{ "clave": "items", "mostrados": 13, "total": 20, "pedir_resto": "detalle:completo" }])
    );
}

#[test]
fn should_cap_the_form_fields_of_the_view_and_say_the_answer_keeps_them_all() {
    let campos: Vec<Value> = (0..60)
        .map(|i| json!({ "clave": format!("campo_{i}"), "valor": "1", "unidad": "W", "editable": true }))
        .collect();
    let mut respuesta = json!({ "campos": campos });
    anadir_vista(
        &Paths::resolve(),
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
fn should_give_a_failed_call_a_code_a_cause_and_the_keys_it_needs() {
    let paths = Paths::resolve();
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

#[test]
fn should_list_every_declared_key_when_the_value_of_an_argument_is_invalid() {
    let paths = Paths::resolve();
    let args = json!({ "que": "nada" });
    let error = call(&paths, "resultados", &args).expect_err("unknown value");
    let vista = vista_del_error(&paths, ("resultados", &args), &error).expect("known tool");
    let claves = vista["error"]["claves_validas"]
        .as_array()
        .cloned()
        .unwrap_or_default();
    assert!(claves.contains(&json!("proyecto")), "{claves:?}");
}

#[test]
fn should_build_no_vista_for_an_unknown_tool() {
    let paths = Paths::resolve();
    let error = Error::NotFound("Unknown tool".into());
    assert!(vista_del_error(&paths, ("no_existe", &json!({})), &error).is_none());
}
