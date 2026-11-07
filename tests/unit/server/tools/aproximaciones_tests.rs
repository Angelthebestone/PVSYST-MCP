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
