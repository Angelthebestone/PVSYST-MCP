//! Exports never replace an existing file unless the caller asks for it.

use super::componentes::exportar_componente;
use crate::config::Paths;
use crate::error::Error;
use serde_json::json;
use std::fs;

fn workspace_con_componente() -> (tempfile::TempDir, Paths) {
    let tmp = tempfile::tempdir().expect("ws");
    let dir = tmp.path().join("ComposPV").join("Inverters");
    fs::create_dir_all(&dir).expect("dir");
    fs::write(
        dir.join("Mio.OND"),
        "PVObject_=pvInverter\nEnd of PVObject pvInverter\n",
    )
    .expect("ond");
    let paths = Paths::para_test(tmp.path());
    (tmp, paths)
}

#[test]
fn should_keep_existing_file_when_exporting_component_without_sobrescribir() {
    let (tmp, paths) = workspace_con_componente();
    let destino = tmp.path().join("copia.OND");
    fs::write(&destino, "previo").expect("previo");
    let args = json!({ "componente": "Mio.OND", "tipo": "OND", "destino": "copia.OND" });

    let res = exportar_componente(&paths, &args);

    assert!(matches!(res, Err(Error::Invalid(m)) if m.contains("sobrescribir")));
    assert_eq!(fs::read_to_string(&destino).expect("destino"), "previo");
}
