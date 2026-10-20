//! Tests for component resolution and workspace confinement.

use super::*;
use serde_json::json;
use std::fs;
use tempfile::tempdir;

#[test]
fn should_reject_reading_component_with_absolute_path_outside_workspace() {
    let temp = tempdir().expect("tempdir");
    let outside_dir = tempdir().expect("tempdir outside");
    let outside_file = outside_dir.path().join("external.OND");
    fs::write(
        &outside_file,
        b"PVObject_=pvGInverter\nEnd of PVObject pvGInverter\n",
    )
    .expect("write");

    let paths = Paths::resolve();
    let args = json!({
        "workspace": temp.path().to_string_lossy(),
        "componente": outside_file.to_string_lossy(),
        "tipo": "OND"
    });

    let res = leer_componente(&paths, &args);
    assert!(
        res.is_err(),
        "debe rechazar ruta fuera del workspace por defecto"
    );
}

#[test]
fn should_reject_deleting_component_with_absolute_path_outside_workspace() {
    let temp = tempdir().expect("tempdir");
    let outside_dir = tempdir().expect("tempdir outside");
    let outside_file = outside_dir.path().join("external.OND");
    fs::write(&outside_file, b"test").expect("write");

    let paths = Paths::resolve();
    let args = json!({
        "workspace": temp.path().to_string_lossy(),
        "componente": outside_file.to_string_lossy(),
        "tipo": "OND"
    });

    let res = borrar_componente(&paths, &args);
    assert!(
        res.is_err(),
        "debe rechazar borrar fuera del workspace sin opt-in"
    );
    assert!(outside_file.exists(), "no debe borrar el archivo exterior");
}

#[test]
fn should_reject_deleting_component_from_read_only_database() {
    let temp = tempdir().expect("tempdir");
    let ro_ond = crate::config::compos_ro()
        .join("Inverters")
        .join("Generic_50kW.OND");
    if !ro_ond.exists() {
        return;
    }

    let paths = Paths::resolve();
    let args = json!({
        "workspace": temp.path().to_string_lossy(),
        "componente": ro_ond.to_string_lossy(),
        "tipo": "OND",
        "permitir_fuera_del_workspace": true
    });

    let res = borrar_componente(&paths, &args);
    assert!(res.is_err(), "nunca debe permitir borrar de la base RO");
    let err_str = res.unwrap_err().to_string();
    assert!(
        err_str.contains("solo lectura"),
        "error debe mencionar solo lectura: {err_str}"
    );
}

#[test]
fn should_reject_component_file_larger_than_any_real_component() {
    let temp = tempdir().expect("tempdir");
    let carpeta = temp.path().join("ComposPV").join("Inverters");
    fs::create_dir_all(&carpeta).expect("carpeta");
    // Sparse file: the size check must fire before any byte is read.
    let file = fs::File::create(carpeta.join("huge.OND")).expect("create");
    file.set_len(64 * 1024 * 1024).expect("set_len");

    let args = json!({ "workspace": temp.path(), "componente": "huge.OND", "tipo": "OND" });
    let err = leer_componente(&Paths::resolve(), &args).expect_err("demasiado grande");
    assert!(err.to_string().contains("como máximo"), "err={err}");
}

#[test]
fn should_allow_reading_component_outside_when_explicitly_permitted() {
    let temp = tempdir().expect("tempdir");
    let outside_dir = tempdir().expect("tempdir outside");
    let outside_file = outside_dir.path().join("external_allowed.OND");
    fs::write(
        &outside_file,
        b"PVObject_=pvGInverter\n  Manufacturer=Test\nEnd of PVObject pvGInverter\n",
    )
    .expect("write");

    let paths = Paths::resolve();
    let args = json!({
        "workspace": temp.path().to_string_lossy(),
        "componente": outside_file.to_string_lossy(),
        "tipo": "OND",
        "permitir_fuera_del_workspace": true
    });

    let res = leer_componente(&paths, &args).expect("debe permitir con permiso explícito");
    assert_eq!(res["is_text"], true);
}
#[test]
fn should_resolve_relative_component_path_inside_the_workspace() {
    let temp = tempdir().expect("tempdir");
    fs::create_dir_all(temp.path().join("propios")).expect("dir");
    fs::write(
        temp.path().join("propios").join("mio.OND"),
        b"PVObject_=pvGInverter\nEnd of PVObject pvGInverter\n",
    )
    .expect("write");
    let args = json!({ "workspace": temp.path(), "componente": "propios/mio.OND", "tipo": "OND" });
    let res = leer_componente(&Paths::resolve(), &args).expect("ruta relativa al workspace");
    assert_eq!(res["is_text"], true);
}
