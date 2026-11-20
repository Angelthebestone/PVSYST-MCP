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
