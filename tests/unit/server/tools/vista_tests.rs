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
    Contexto { tool, accion, args }
}

fn con_vista((tool, accion): (&str, &str), args: &Value, salida: Value) -> Value {
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
fn should_list_the_form_fields_the_gui_already_keys_without_cutting_them() {
    let campos: Vec<Value> = (0..60)
        .map(|i| json!({ "clave": format!("campo_{i}"), "valor": "1", "unidad": "W", "editable": true }))
        .collect();
    let vista = con_vista(
        ("gui_observar", "vista"),
        &json!({}),
        json!({ "campos": campos }),
    );
    assert_eq!(vista["campos"].as_array().map(Vec::len), Some(60));
    assert_eq!(vista["campos"][0]["editable"], true);
    assert!(vista.get("recortado").is_none());
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

#[test]
fn should_ask_to_repeat_the_call_with_confirmation_when_trial_mode_needs_it() {
    let args = json!({ "latitud": 40.4, "longitud": -3.7, "kwp": 8 });
    let salida = json!({
        "requiere_confirmacion": true,
        "mensaje": "Esta acción consume 2 ejecuciones de prueba; repítela con confirmar:true",
    });
    let vista = con_vista(("disenar_sistema", ""), &args, salida);
    assert_eq!(vista["estado"], "parcial");
    assert_eq!(vista["error"]["codigo"], "requiere_confirmacion");
    assert!(vista["resumen"]
        .as_str()
        .is_some_and(|r| r.starts_with("No se ejecutó nada")));
    assert_eq!(vista["siguiente"].as_array().map(Vec::len), Some(1));
    assert_eq!(vista["siguiente"][0]["tool"], "disenar_sistema");
    assert_eq!(
        vista["siguiente"][0]["argumentos"],
        json!({ "latitud": 40.4, "longitud": -3.7, "kwp": 8, "confirmar": true })
    );
}

#[test]
fn should_offer_the_real_run_after_a_dry_run() {
    let args = json!({ "accion": "dimensionar", "proyecto": "Casa", "dry_run": true });
    let vista = con_vista(
        ("proyecto", "dimensionar"),
        &args,
        json!({ "dry_run": true }),
    );
    assert_eq!(vista["estado"], "ok");
    assert!(vista["resumen"]
        .as_str()
        .is_some_and(|r| r.starts_with("Ensayo")));
    assert_eq!(
        vista["siguiente"][0]["argumentos"],
        json!({ "accion": "dimensionar", "proyecto": "Casa" })
    );
}

#[test]
fn should_not_suggest_results_of_a_project_the_answer_never_names() {
    let vista = con_vista(("disenar_sistema", ""), &json!({}), json!({ "ok": true }));
    let herramientas: Vec<_> = vista["siguiente"]
        .as_array()
        .into_iter()
        .flatten()
        .map(|p| p["tool"].clone())
        .collect();
    assert_eq!(herramientas, [json!("entorno")]);
}
