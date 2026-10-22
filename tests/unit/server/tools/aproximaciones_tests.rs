use super::*;

#[test]
fn should_reject_in_house_actions_without_aproximaciones_flag_mentioning_env_var() {
    let mut paths = Paths::resolve();
    paths.aproximaciones = false;

    let err = call(&paths, "simular", &json!({ "accion": "preestimar" }))
        .expect_err("preestimar requires the flag");
    let msg = err.to_string();
    assert!(
        msg.contains("PVSYST_MCP_APROXIMACIONES=1"),
        "error must name the required env var: {msg}"
    );

    let err = call(&paths, "simular", &json!({ "accion": "barrido" }))
        .expect_err("barrido requires the flag");
    assert!(
        err.to_string().contains("PVSYST_MCP_APROXIMACIONES=1"),
        "{err}"
    );

    let err = call(
        &paths,
        "sitio",
        &json!({ "accion": "geometria_solar", "latitud": 0.0, "longitud": 0.0 }),
    )
    .expect_err("geometria_solar requires the flag");
    assert!(
        err.to_string().contains("PVSYST_MCP_APROXIMACIONES=1"),
        "{err}"
    );

    let err = call(
        &paths,
        "resultados",
        &json!({ "que": "medido_vs_simulado", "proyecto": "Demo" }),
    )
    .expect_err("medido_vs_simulado requires the flag");
    assert!(
        err.to_string().contains("PVSYST_MCP_APROXIMACIONES=1"),
        "{err}"
    );
}

#[test]
fn should_hide_in_house_actions_without_the_flag() {
    let mut paths = Paths::resolve();
    paths.aproximaciones = false;
    let schemas = list(&paths);
    let simular = schemas
        .iter()
        .find(|t| t["name"] == "simular")
        .expect("simular");
    let acciones = simular["inputSchema"]["properties"]["accion"]["enum"]
        .as_array()
        .expect("enum");
    assert!(!acciones.iter().any(|v| v == "preestimar"));
    assert!(!acciones.iter().any(|v| v == "barrido"));
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
    let simular = schemas
        .iter()
        .find(|t| t["name"] == "simular")
        .expect("simular");
    let acciones = simular["inputSchema"]["properties"]["accion"]["enum"]
        .as_array()
        .expect("enum");
    assert!(acciones.iter().any(|v| v == "preestimar"));
    assert!(acciones.iter().any(|v| v == "barrido"));

    let sitio = schemas
        .iter()
        .find(|t| t["name"] == "sitio")
        .expect("sitio");
    let via_enum = &sitio["inputSchema"]["properties"]["via"]["enum"];
    assert_eq!(
        via_enum,
        &json!(["pvsyst", "aproximacion"]),
        "via must expose 'aproximacion' when flag is on"
    );
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
