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

#[test]
fn should_edit_legacy_component_in_place_with_history() {
    let temp = tempdir().expect("tempdir");
    let mods_dir = temp.path().join("ComposPV").join("PVModules");
    fs::create_dir_all(&mods_dir).expect("create dir");
    let pan_path = mods_dir.join("Synth_Legacy_300W.PAN");
    let fixture = include_bytes!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/tests/fixtures/workspace/ComposPV/PVModules/Synth_Legacy_300W.PAN"
    ));
    fs::write(&pan_path, fixture).expect("write fixture");

    let paths = Paths::resolve();
    let args = json!({
        "workspace": temp.path().to_string_lossy(),
        "componente": "Synth_Legacy_300W.PAN",
        "tipo": "PAN",
        "cambios": { "pnom_w": 320.0 }
    });

    let res = editar_componente(&paths, &args).expect("editar_componente");
    assert!(res["historial_id"].is_string());

    let modified_bytes = fs::read(&pan_path).expect("read modified");
    let parsed = pan::parse_pan(&modified_bytes).expect("parse modified");
    assert_eq!(parsed["pnom"], 320.0);
    assert_eq!(parsed["voc"], 40.0);
}

#[test]
fn should_create_legacy_component_from_template_with_applied_changes() {
    let temp = tempdir().expect("tempdir");
    let mods_dir = temp.path().join("ComposPV").join("PVModules");
    fs::create_dir_all(&mods_dir).expect("create dir");
    let pan_path = mods_dir.join("Synth_Legacy_300W.PAN");
    let fixture = include_bytes!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/tests/fixtures/workspace/ComposPV/PVModules/Synth_Legacy_300W.PAN"
    ));
    fs::write(&pan_path, fixture).expect("write fixture");

    let paths = Paths::resolve();
    let args = json!({
        "workspace": temp.path().to_string_lossy(),
        "plantilla": "Synth_Legacy_300W.PAN",
        "nombre_nuevo": "Cloned_Legacy_325W",
        "tipo": "PAN",
        "cambios": { "PNom": 325.0, "Voc": 42.0 }
    });

    let res = crear_componente(&paths, &args).expect("crear_componente");
    assert!(res["created"]
        .as_str()
        .unwrap()
        .ends_with("Cloned_Legacy_325W.PAN"));

    let cloned_path = mods_dir.join("Cloned_Legacy_325W.PAN");
    let cloned_bytes = fs::read(&cloned_path).expect("read cloned");
    let parsed_cloned = pan::parse_pan(&cloned_bytes).expect("parse cloned");
    assert_eq!(parsed_cloned["pnom"], 325.0);
    assert_eq!(parsed_cloned["voc"], 42.0);

    let orig_bytes = fs::read(&pan_path).expect("read orig");
    let parsed_orig = pan::parse_pan(&orig_bytes).expect("parse orig");
    assert_eq!(parsed_orig["pnom"], 300.0);
}

#[test]
fn should_reject_editing_legacy_component_with_unknown_field() {
    let temp = tempdir().expect("tempdir");
    let mods_dir = temp.path().join("ComposPV").join("PVModules");
    fs::create_dir_all(&mods_dir).expect("create dir");
    let pan_path = mods_dir.join("Synth_Legacy_300W.PAN");
    let fixture = include_bytes!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/tests/fixtures/workspace/ComposPV/PVModules/Synth_Legacy_300W.PAN"
    ));
    fs::write(&pan_path, fixture).expect("write fixture");

    let paths = Paths::resolve();
    let args = json!({
        "workspace": temp.path().to_string_lossy(),
        "componente": "Synth_Legacy_300W.PAN",
        "tipo": "PAN",
        "cambios": { "unknown_legacy_field": 123.4 }
    });

    let err = editar_componente(&paths, &args).unwrap_err();
    assert!(
        matches!(err, Error::Invalid(msg) if msg.contains("No comprobado") || msg.contains("desconocido"))
    );
}

#[test]
fn should_reject_editing_legacy_component_in_read_only_database() {
    let temp = tempdir().expect("tempdir");
    let ro_pan = crate::config::compos_ro()
        .join("PVModules")
        .join("Concentrix_CPV_Gen_II.PAN");
    if !ro_pan.exists() {
        return;
    }
    let paths = Paths::resolve();
    let args = json!({
        "workspace": temp.path().to_string_lossy(),
        "componente": "Concentrix_CPV_Gen_II.PAN",
        "tipo": "PAN",
        "cambios": { "pnom_w": 90.0 }
    });
    let err = editar_componente(&paths, &args).unwrap_err();
    assert!(matches!(err, Error::Invalid(msg) if msg.contains("solo lectura")));
}

#[test]
fn should_resolve_component_type_aliases_to_their_pvsyst_codes() {
    assert_eq!(tipo_canonico("inverters"), Some(("OND", "Inverters")));
    assert_eq!(tipo_canonico("INVERSOR"), Some(("OND", "Inverters")));
    assert_eq!(tipo_canonico("modules"), Some(("PAN", "PVModules")));
    assert_eq!(tipo_canonico("pan"), Some(("PAN", "PVModules")));
    assert_eq!(tipo_canonico("desconocido"), None);
}
