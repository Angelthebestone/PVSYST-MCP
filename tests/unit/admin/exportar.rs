//! Exports never replace an existing file unless the caller asks for it.

use super::componentes::exportar_componente;
use super::proyecto::exportar_proyecto;
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

#[test]
fn should_replace_existing_file_when_exporting_component_with_sobrescribir() {
    let (tmp, paths) = workspace_con_componente();
    let destino = tmp.path().join("copia.OND");
    fs::write(&destino, "previo").expect("previo");
    let args = json!({ "componente": "Mio.OND", "tipo": "OND", "destino": "copia.OND", "sobrescribir": true });

    exportar_componente(&paths, &args).expect("exportado");

    assert!(fs::read_to_string(&destino)
        .expect("destino")
        .contains("pvInverter"));
}

#[test]
fn should_keep_existing_zip_when_exporting_project_without_sobrescribir() {
    let tmp = tempfile::tempdir().expect("ws");
    let proyectos = tmp.path().join("Projects");
    fs::create_dir_all(&proyectos).expect("Projects");
    fs::write(proyectos.join("Demo.PRJ"), "PVObject_=pvProject\n").expect("prj");
    fs::write(proyectos.join("Demo.zip"), "previo").expect("zip");
    let paths = Paths::para_test(tmp.path());

    let res = exportar_proyecto(&paths, &json!({ "proyecto": "Demo" }));

    assert!(matches!(res, Err(Error::Invalid(m)) if m.contains("sobrescribir")));
    assert_eq!(
        fs::read_to_string(proyectos.join("Demo.zip")).expect("zip"),
        "previo"
    );
}

#[test]
fn should_refuse_import_when_destination_cannot_be_read() {
    let tmp = tempfile::tempdir().expect("ws");
    fs::write(tmp.path().join("copia.zip"), "zip").expect("zip");
    fs::write(tmp.path().join("destino"), "soy un archivo").expect("destino");
    let paths = Paths::para_test(tmp.path());
    let args = json!({ "zip": "copia.zip", "destino": "destino" });

    let res = super::importacion::importar_workspace(&paths, &args);

    assert!(
        matches!(&res, Err(Error::Io { context, .. }) if context.contains("destino")),
        "{res:?}"
    );
}

#[test]
fn should_list_every_entry_of_a_folder() {
    let tmp = tempfile::tempdir().expect("dir");
    fs::write(tmp.path().join("a.PRJ"), "a").expect("a");
    fs::create_dir(tmp.path().join("sub")).expect("sub");

    let mut entradas = crate::workspace::entradas_de(tmp.path()).expect("entradas");
    entradas.sort();

    assert_eq!(entradas, [tmp.path().join("a.PRJ"), tmp.path().join("sub")]);
    assert!(matches!(
        crate::workspace::entradas_de(&tmp.path().join("no_existe")),
        Err(crate::error::Error::Io { .. })
    ));
}
