use super::*;

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
    let sitio = schemas
        .iter()
        .find(|t| t["name"] == "sitio")
        .expect("sitio");
    let acciones = sitio["inputSchema"]["properties"]["accion"]["enum"]
        .as_array()
        .expect("enum");
    assert!(
        acciones.iter().any(|v| v == "horizonte_pvgis"),
        "horizonte_pvgis defaults to PVsyst and must remain exposed"
    );

    let via_enum = &sitio["inputSchema"]["properties"]["via"]["enum"];
    assert_eq!(
        via_enum,
        &json!(["pvsyst"]),
        "via must hide 'aproximacion' when flag is off"
    );
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
        "sitio",
        &json!({ "accion": "horizonte_pvgis", "proyecto": "Demo", "via": "desconocida" }),
    )
    .expect_err("unknown via must be rejected");
    assert!(err.to_string().contains("no válido para via"), "{err}");
}
