use super::*;

#[test]
fn should_reject_in_house_actions_when_the_approximations_flag_is_off() {
    let mut paths = Paths::para_test(std::env::temp_dir());
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
fn should_not_name_in_house_actions_anywhere_when_the_flag_is_off() {
    let mut paths = Paths::para_test(std::env::temp_dir());
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

#[test]
fn should_name_the_environment_variable_when_a_gated_action_is_rejected() {
    // Given: a server without the approximation flag
    let mut paths = Paths::resolve();
    paths.aproximaciones = false;

    // When: the caller asks for an action that only exists behind it
    let err = call(
        &paths,
        "sitio",
        &json!({ "accion": "geometria_solar", "latitud": 4.7, "longitud": -74.0 }),
    )
    .expect_err("geometria_solar is gated");

    // Then: the rejection says the action exists and what unlocks it
    let msg = err.to_string();
    assert!(msg.contains("geometria_solar"), "{msg}");
    assert!(msg.contains("PVSYST_MCP_APROXIMACIONES=1"), "{msg}");
}

#[test]
fn should_not_mention_the_environment_variable_for_an_unknown_action() {
    let mut paths = Paths::resolve();
    paths.aproximaciones = false;
    let err = call(&paths, "sitio", &json!({ "accion": "inventada" })).expect_err("unknown action");
    let msg = err.to_string();
    assert!(msg.contains("sitio") && msg.contains("accion"), "{msg}");
    assert!(!msg.contains("PVSYST_MCP_APROXIMACIONES"), "{msg}");
}
