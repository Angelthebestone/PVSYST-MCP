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

    let paths = Paths::para_test(std::env::temp_dir());
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

    let paths = Paths::para_test(std::env::temp_dir());
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

    let paths = Paths::para_test(std::env::temp_dir());
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

    let paths = Paths::para_test(std::env::temp_dir());
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

/// The rejection of an unknown kind speaks the client's language and lists the known kinds.
#[test]
fn should_word_an_unknown_component_kind_in_the_client_language() {
    let es = tipo_desconocido("NOPE", Idioma::Es).to_string();
    let en = tipo_desconocido("NOPE", Idioma::En).to_string();

    assert!(
        es.contains("Tipo de componente desconocido 'NOPE'") && es.contains("PAN"),
        "{es}"
    );
    assert!(
        en.contains("Unknown component kind 'NOPE'") && en.contains("PAN"),
        "{en}"
    );
}
