use super::*;

#[test]
fn should_list_installs_newest_first_when_the_folders_hold_several_versions() {
    let raiz = tempfile::tempdir().expect("carpeta temporal");
    for nombre in [
        "PVsyst8.1.8",
        "PVsyst8.1.10",
        "PVsyst8.1_Data",
        "PVsyst9.0.1",
        "Otro",
    ] {
        std::fs::create_dir(raiz.path().join(nombre)).expect("carpeta de prueba");
    }

    let halladas = instalaciones_en(&[raiz.path().to_path_buf()]);

    let nombres: Vec<_> = halladas
        .iter()
        .filter_map(|p| p.file_name()?.to_str())
        .collect();
    assert_eq!(nombres, ["PVsyst8.1.10", "PVsyst8.1.8"]);
}

#[test]
fn should_document_the_dataro_override_in_the_client_example() {
    let ejemplo = include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/mcp.example.json"));
    let json: serde_json::Value = serde_json::from_str(ejemplo).expect("ejemplo JSON válido");
    let env = &json["mcpServers"]["pvsyst"]["env"];
    assert!(
        env.get("PVSYST_DATARO").is_some(),
        "sin PVSYST_DATARO, leer DataRO falla: {env}"
    );
}
