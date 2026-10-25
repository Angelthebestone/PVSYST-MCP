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
    let desc = simular["inputSchema"]["properties"]["accion"]["description"]
        .as_str()
        .expect("description");
    assert!(
        !desc.contains("preestimar"),
        "description must not mention preestimar: {desc}"
    );
    assert!(
        !desc.contains("preestimate"),
        "description must not mention preestimate: {desc}"
    );
    assert!(
        !desc.contains("barrido"),
        "description must not mention barrido: {desc}"
    );
    assert!(
        !desc.contains("sweep"),
        "description must not mention sweep: {desc}"
    );
}

#[test]
fn should_filter_approximated_synonyms_from_coverage_resource_when_flag_is_off() {
    let cob_con = crate::server::resources::read("pvsyst://cobertura", true).unwrap();
    let text_con = cob_con["contents"][0]["text"].as_str().unwrap();
    let data_con: serde_json::Value = serde_json::from_str(text_con).unwrap();
    let sim_vals_con = &data_con["sinonimos"]["simular"]["valores"];
    assert!(sim_vals_con.get("preestimate").is_some());

    let cob_sin = crate::server::resources::read("pvsyst://cobertura", false).unwrap();
    let text_sin = cob_sin["contents"][0]["text"].as_str().unwrap();
    let data_sin: serde_json::Value = serde_json::from_str(text_sin).unwrap();
    let sim_vals_sin = &data_sin["sinonimos"]["simular"]["valores"];
    assert!(
        sim_vals_sin.get("preestimate").is_none(),
        "preestimate must be filtered from cobertura: {sim_vals_sin:?}"
    );
    assert!(
        sim_vals_sin.get("sweep").is_none(),
        "sweep must be filtered from cobertura: {sim_vals_sin:?}"
    );
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

#[test]
fn should_not_name_in_house_actions_anywhere_in_the_catalogue_without_the_flag() {
    let mut paths = Paths::resolve();
    paths.aproximaciones = false;
    let catalogo = serde_json::to_string(&list(&paths)).expect("catalogue serializes");
    for palabra in [
        "preestimar",
        "barrido",
        "medido_vs_simulado",
        "geometria_solar",
        "pre-estimate",
        "sweep",
        "measured vs",
        "solar geometry",
        "aproximacion",
    ] {
        assert!(
            !catalogo.contains(palabra),
            "'{palabra}' leaks into tools/list"
        );
    }
}
