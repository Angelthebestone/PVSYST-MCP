use super::*;

#[test]
fn should_be_unset_when_variable_is_absent() {
    assert_eq!(valor_de_entorno_util(None), None);
}

#[test]
fn should_be_unset_when_variable_is_empty() {
    assert_eq!(valor_de_entorno_util(Some(OsString::from(""))), None);
}

#[test]
fn should_be_unset_when_variable_is_only_whitespace() {
    assert_eq!(valor_de_entorno_util(Some(OsString::from("  "))), None);
}

#[test]
fn should_read_version_when_folder_is_a_pvsyst_8_install() {
    assert_eq!(version_de("PVsyst8.1.8"), Some(vec![8, 1, 8]));
}

#[test]
fn should_rank_1_10_above_1_8_when_comparing_versions() {
    assert!(version_de("PVsyst8.1.10") > version_de("PVsyst8.1.8"));
}

#[test]
fn should_ignore_folder_when_it_is_data_or_a_major_before_8() {
    assert_eq!(version_de("PVsyst8.1_Data"), None);
    assert_eq!(version_de("PVsystBasic1.0"), None);
    assert_eq!(version_de("PVsyst7.4.0"), None);
}

#[test]
fn should_skip_a_later_major_and_rank_minors_when_comparing_versions() {
    assert_eq!(version_de("PVsyst9.0.1"), None);
    assert!(version_de("PVsyst8.2") > version_de("PVsyst8.1.10"));
}

#[test]
fn should_return_path_when_variable_has_a_value() {
    assert_eq!(
        valor_de_entorno_util(Some(OsString::from(r"C:\PVsyst8.1_Data"))),
        Some(PathBuf::from(r"C:\PVsyst8.1_Data"))
    );
}

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
fn should_list_nothing_when_the_install_folder_does_not_exist() {
    let ausente = tempfile::tempdir()
        .expect("carpeta temporal")
        .path()
        .join("no_existe");

    assert!(instalaciones_en(&[ausente]).is_empty());
}

#[test]
fn should_scan_the_install_folders_once_when_asked_repeatedly() {
    assert!(std::ptr::eq(instalaciones(), instalaciones()));
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
