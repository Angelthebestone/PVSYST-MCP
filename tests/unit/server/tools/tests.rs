//! Unit and behavioral tests for MCP tool declarations, validation and dispatch.

use super::*;
use crate::config::Paths;
use crate::progreso::SeguimientoTarea;
use serde_json::{json, Value};

fn call(paths: &Paths, name: &str, args: &Value) -> Result<Value> {
    let seguimiento = SeguimientoTarea::inerte();
    let solicitud = Solicitud {
        paths,
        name,
        args,
        seguimiento: &seguimiento,
    };
    call_seguido(&solicitud)
}

#[test]
fn should_reject_missing_required_arg() {
    let paths = Paths::resolve();
    // 'leer' requires 'archivo'
    let err = call(&paths, "leer", &json!({})).expect_err("should reject missing required arg");
    let msg = err.to_string();
    assert!(
        msg.contains("Missing required argument 'archivo'"),
        "error message must name the missing field: {msg}"
    );
}

#[test]
fn should_reject_wrong_type_arg_mentioning_tipo() {
    let paths = Paths::resolve();
    // 'base64' is declared boolean; passing a string "yes" must fail at boundary
    let err = call(
        &paths,
        "leer",
        &json!({ "archivo": "test.PAN", "base64": "yes" }),
    )
    .expect_err("should reject string for boolean property");
    let msg = err.to_string();
    assert!(
        msg.to_lowercase().contains("tipo"),
        "error message must mention 'tipo': {msg}"
    );
    assert!(
        msg.contains("base64"),
        "error message must name the invalid field: {msg}"
    );
    assert!(
        msg.contains("esperado boolean") && msg.contains("recibido string"),
        "error message must specify expected and received types: {msg}"
    );
}

#[test]
fn should_reject_wrong_type_for_numeric_property() {
    let paths = Paths::resolve();
    // 'n_inversores' in 'proyecto' is integer; passing a boolean must fail
    let err = call(
        &paths,
        "proyecto",
        &json!({ "accion": "crear", "plantilla": "DEMO", "nombre_nuevo": "NUEVO", "n_inversores": true }),
    )
    .expect_err("should reject boolean for integer property");
    let msg = err.to_string();
    assert!(
        msg.to_lowercase().contains("tipo"),
        "error must mention 'tipo': {msg}"
    );
    assert!(msg.contains("n_inversores"), "error must name field: {msg}");
}

#[test]
fn should_reject_missing_discriminator_when_required() {
    let paths = Paths::resolve();
    // 'proyecto' requires 'accion' (no default)
    let err = call(&paths, "proyecto", &json!({})).expect_err("should require accion");
    let msg = err.to_string();
    assert!(
        msg.contains("Missing required argument 'accion'"),
        "should name the missing discriminator: {msg}"
    );
}

#[test]
fn should_apply_default_discriminator_when_omitted() {
    let paths = Paths::resolve();
    // 'entorno' defaults 'accion' to 'diagnostico'
    let res = call(&paths, "entorno", &json!({})).expect("should succeed with default diagnostico");
    assert!(res.get("entorno").is_some());
}

#[test]
fn should_reject_invalid_discriminator_value() {
    let paths = Paths::resolve();
    let err = call(&paths, "entorno", &json!({ "accion": "inexistente" }))
        .expect_err("should reject invalid accion");
    let msg = err.to_string();
    assert!(
        msg.contains("Valor 'inexistente' no válido"),
        "should reject invalid value: {msg}"
    );
}

#[test]
fn should_reject_wrong_type_for_discriminator() {
    let paths = Paths::resolve();
    let err = call(&paths, "entorno", &json!({ "accion": 12345 }))
        .expect_err("should reject number for accion");
    let msg = err.to_string();
    assert!(
        msg.to_lowercase().contains("tipo"),
        "should mention tipo: {msg}"
    );
    assert!(msg.contains("accion"), "should mention field name: {msg}");
}

#[test]
fn should_accept_compatible_union_types() {
    // 'n_mod_serie' is integer|string
    let h = REGISTRO.iter().find(|h| h.nombre == "proyecto").unwrap();
    let val_int =
        json!({ "accion": "crear", "plantilla": "P", "nombre_nuevo": "N", "n_mod_serie": 20 });
    let val_str =
        json!({ "accion": "crear", "plantilla": "P", "nombre_nuevo": "N", "n_mod_serie": "20" });
    assert!(h.validar(&val_int).is_ok());
    assert!(h.validar(&val_str).is_ok());

    let val_bad =
        json!({ "accion": "crear", "plantilla": "P", "nombre_nuevo": "N", "n_mod_serie": false });
    assert!(h.validar(&val_bad).is_err());
}

#[test]
fn should_expose_all_fifteen_pvsyst_tools_by_default() {
    let paths = Paths::resolve();
    assert_eq!(list(&paths).len(), 15);
}

#[test]
fn should_include_permitir_fuera_del_workspace_property() {
    let paths = Paths::resolve();
    let schemas = list(&paths);
    for tool_name in [
        "leer",
        "listar",
        "editar",
        "escribir_archivo",
        "componente",
        "sitio",
        "resultados",
        "workspace",
    ] {
        let tool_schema = schemas
            .iter()
            .find(|s| s["name"] == tool_name)
            .unwrap_or_else(|| panic!("missing tool {tool_name}"));
        assert!(
            tool_schema["inputSchema"]["properties"]
                .get("permitir_fuera_del_workspace")
                .is_some(),
            "tool {tool_name} must declare permitir_fuera_del_workspace in its properties"
        );
    }
}

#[test]
fn should_annotate_phase_three_schema_constraints() {
    let paths = Paths::resolve();
    let schemas = list(&paths);
    let proyecto = schemas
        .iter()
        .find(|s| s["name"] == "proyecto")
        .expect("proyecto");
    let proyecto_props = &proyecto["inputSchema"]["properties"];
    assert_eq!(
        proyecto_props["estrategia"]["enum"],
        json!(["autoconsumo", "recorte_picos"])
    );

    let resultados = schemas
        .iter()
        .find(|s| s["name"] == "resultados")
        .expect("resultados");
    let medido = &resultados["inputSchema"]["properties"]["medido_kwh"];
    assert_eq!(medido["minItems"], json!(12));
    assert_eq!(medido["maxItems"], json!(12));
    assert_eq!(medido["items"]["type"], "number");

    let disenar = schemas
        .iter()
        .find(|s| s["name"] == "disenar_sistema")
        .expect("disenar_sistema");
    assert_eq!(
        disenar["inputSchema"]["properties"]["tipo_sistema"]["enum"],
        json!(["red", "aislado"])
    );
    assert_eq!(
        disenar["inputSchema"]["properties"]["n_baterias_serie"]["minimum"],
        json!(1)
    );
}

#[test]
fn should_reject_in_house_actions_without_aproximaciones_flag_mentioning_env_var() {
    let mut paths = Paths::resolve();
    paths.aproximaciones = false;

    for (tool, args) in [
        (
            "simular",
            json!({ "accion": "preestimar", "latitud": 40.0, "longitud": -3.0, "kwp": 5.0 }),
        ),
        (
            "sitio",
            json!({ "accion": "geometria_solar", "latitud": 40.0, "longitud": -3.0 }),
        ),
        (
            "resultados",
            json!({ "que": "medido_vs_simulado", "proyecto": "P" }),
        ),
    ] {
        let err = call(&paths, tool, &args)
            .expect_err("in-house action must be rejected without aproximaciones flag");
        assert!(
            err.to_string().contains("PVSYST_MCP_APROXIMACIONES"),
            "error must mention PVSYST_MCP_APROXIMACIONES: {err}"
        );
    }
}

fn enum_de(schemas: &[Value], tool: &str, campo: &str) -> Vec<String> {
    schemas
        .iter()
        .find(|s| s["name"] == tool)
        .and_then(|s| s.pointer(&format!("/inputSchema/properties/{campo}/enum")))
        .and_then(Value::as_array)
        .unwrap_or_else(|| panic!("missing enum for {tool}.{campo}"))
        .iter()
        .filter_map(Value::as_str)
        .map(String::from)
        .collect()
}

#[test]
fn should_hide_in_house_actions_without_the_flag() {
    let mut paths = Paths::resolve();
    paths.aproximaciones = false;
    let schemas = list(&paths);

    assert!(!enum_de(&schemas, "simular", "accion").contains(&"preestimar".to_string()));
    assert!(!enum_de(&schemas, "simular", "accion").contains(&"barrido".to_string()));
    assert!(!enum_de(&schemas, "resultados", "que").contains(&"medido_vs_simulado".to_string()));
    assert!(!enum_de(&schemas, "sitio", "accion").contains(&"geometria_solar".to_string()));
}

#[test]
fn should_keep_native_default_actions_and_hide_the_via_route_without_the_flag() {
    let mut paths = Paths::resolve();
    paths.aproximaciones = false;
    let schemas = list(&paths);

    // These actions keep PVsyst as their default route, so they stay visible.
    assert!(enum_de(&schemas, "componente", "accion").contains(&"crear_desde_ficha".to_string()));
    assert!(enum_de(&schemas, "resultados", "que").contains(&"p50_p90".to_string()));
    assert!(enum_de(&schemas, "sitio", "accion").contains(&"horizonte_pvgis".to_string()));

    for (tool, campo) in [("proyecto", "via"), ("resultados", "via")] {
        let via = enum_de(&schemas, tool, campo);
        assert!(!via.contains(&"aproximacion".to_string()), "{tool}.{campo}");
        assert!(via.contains(&"pvsyst".to_string()), "{tool}.{campo}");
    }
}

#[test]
fn should_show_in_house_actions_and_the_via_route_with_the_flag() {
    let mut paths = Paths::resolve();
    paths.aproximaciones = true;
    let schemas = list(&paths);

    assert!(enum_de(&schemas, "simular", "accion").contains(&"preestimar".to_string()));
    assert!(enum_de(&schemas, "simular", "accion").contains(&"barrido".to_string()));
    assert!(enum_de(&schemas, "resultados", "que").contains(&"medido_vs_simulado".to_string()));
    assert!(enum_de(&schemas, "sitio", "accion").contains(&"geometria_solar".to_string()));
    assert!(enum_de(&schemas, "proyecto", "via").contains(&"aproximacion".to_string()));
}

#[test]
fn should_reject_unknown_via_route() {
    let paths = Paths::resolve();
    let err = call(
        &paths,
        "proyecto",
        &json!({ "accion": "crear", "plantilla": "P", "nombre_nuevo": "N", "via": "otra" }),
    )
    .expect_err("unknown via must be rejected");
    assert!(err.to_string().contains("no válido para via"), "{err}");
}

#[test]
fn should_expose_the_new_gui_field_actions() {
    let paths = Paths::resolve();
    let schemas = list(&paths);
    let tool = |name: &str| {
        schemas
            .iter()
            .find(|s| s["name"] == name)
            .unwrap_or_else(|| panic!("missing tool {name}"))
    };

    let actuar = &tool("gui_actuar")["inputSchema"]["properties"];
    assert_eq!(actuar["accion"]["default"], "click");
    for accion in ["escribir", "seleccionar", "pagina", "esperar_cierre"] {
        assert!(
            actuar["accion"]["enum"]
                .as_array()
                .unwrap()
                .contains(&json!(accion)),
            "missing action {accion}"
        );
    }
    for campo in ["texto", "campo", "indice", "pids"] {
        assert!(actuar.get(campo).is_some(), "missing property {campo}");
    }

    let observar = &tool("gui_observar")["inputSchema"]["properties"];
    assert!(observar["que"]["enum"]
        .as_array()
        .unwrap()
        .contains(&json!("mensajes")));
    assert!(observar.get("lineas").is_some());
}

#[test]
fn should_dispatch_leer_for_currency_mef_and_3d_scenes() {
    let paths = Paths::resolve();

    // Currency.dat
    let cur_res = call(&paths, "leer", &json!({ "archivo": "Currency.dat" }))
        .expect("Lectura de Currency.dat a través de tool leer");
    assert_eq!(cur_res["version_formato"], "CURRENCY_RATES_V2");
    assert_eq!(cur_res["total_divisas"], 33);

    // MEF
    let mef_res = call(&paths, "leer", &json!({ "archivo": "PVsyst_ANE-NEDO.MEF" }))
        .expect("Lectura de MEF a través de tool leer");
    assert_eq!(mef_res["tipo_formato"], "pvMeteoFormat-moderno");
    assert_eq!(mef_res["comentario"], "ANE-NEDO known format");

    // 3D Scene / Building
    let bld_res = call(
        &paths,
        "leer",
        &json!({ "archivo": "DEMO_Geneve_Grid.BLD" }),
    )
    .expect("Lectura de BLD a través de tool leer");
    assert_eq!(bld_res["tipo"], "bld");
    assert_eq!(bld_res["clase_raiz"], "pvShdBuilding");
    assert_eq!(bld_res["total_objetos"], 3);
}

#[test]
fn should_not_demand_an_id_when_a_menu_command_is_named_by_position() {
    let paths = Paths::resolve();
    let args = json!({ "accion": "menu", "ventana": "0x1", "menu": 0, "item": 0 });

    let error = call(&paths, "gui_actuar", &args)
        .err()
        .map(|e| e.to_string())
        .unwrap_or_default();

    assert!(!error.contains("Missing required argument"), "{error}");
}

#[test]
fn should_demand_the_window_when_a_menu_command_names_none() {
    let paths = Paths::resolve();

    let error = call(&paths, "gui_actuar", &json!({ "accion": "menu", "id": 70 }))
        .err()
        .map(|e| e.to_string())
        .unwrap_or_default();

    assert!(
        error.contains("Missing required argument 'ventana'"),
        "{error}"
    );
}

#[test]
fn should_expose_click_by_text_restart_and_menu_listing_in_the_gui_tools() {
    let paths = Paths::resolve();
    let schemas = list(&paths);
    let propiedades = |nombre: &str| {
        schemas
            .iter()
            .find(|s| s["name"] == nombre)
            .map(|s| s["inputSchema"]["properties"].clone())
            .unwrap_or_default()
    };
    let admite = |lista: &Value, valor: &str| {
        lista
            .as_array()
            .is_some_and(|valores| valores.contains(&json!(valor)))
    };

    let actuar = propiedades("gui_actuar");
    let observar = propiedades("gui_observar");

    assert!(admite(&actuar["accion"]["enum"], "clic_texto"));
    assert!(admite(&actuar["accion"]["enum"], "reiniciar"));
    assert!(admite(&observar["que"]["enum"], "menus"));
    for campo in ["dx", "dy", "menu", "item"] {
        assert!(actuar.get(campo).is_some(), "falta la propiedad {campo}");
    }
}
