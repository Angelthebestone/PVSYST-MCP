use super::cliente_mcp::{error_text, synthetic_workspace, Client};
use serde_json::json;

#[test]
fn should_reject_parent_segments_in_workspace_create_path() {
    let ws = synthetic_workspace();
    let ws_str = ws.path().to_string_lossy().to_string();
    let mut c = Client::start(Some(&ws_str));
    c.initialize();

    let resp = c.call(
        30,
        "workspace",
        json!({ "accion": "crear", "ruta": "../x_nope", "workspace": ws_str }),
    );
    let msg = error_text(&resp);
    assert!(msg.contains("'ruta' no admite '..'"), "{msg}");
}

#[test]
fn should_reject_default_export_zip_outside_workspace_without_permission() {
    let ws = synthetic_workspace();
    let ws_str = ws.path().to_string_lossy().to_string();
    let mut c = Client::start(Some(&ws_str));
    c.initialize();

    let resp = c.call(
        31,
        "workspace",
        json!({ "accion": "exportar", "workspace": ws_str }),
    );
    let msg = error_text(&resp);
    assert!(msg.contains("permitir_fuera_del_workspace"), "{msg}");
}

#[test]
fn should_keep_existing_export_zip_unless_sobrescribir_is_set() {
    let ws = synthetic_workspace();
    let ws_str = ws.path().to_string_lossy().to_string();
    let mut c = Client::start(Some(&ws_str));
    c.initialize();

    let primero = c.call(
        32,
        "workspace",
        json!({ "accion": "exportar", "destino": "copia.zip", "workspace": ws_str }),
    );
    assert_eq!(primero["result"]["isError"], json!(false), "resp={primero}");

    let segundo = c.call(
        33,
        "workspace",
        json!({ "accion": "exportar", "destino": "copia.zip", "workspace": ws_str }),
    );
    let msg = error_text(&segundo);
    assert!(msg.contains("sobrescribir"), "{msg}");
}

#[test]
fn should_not_archive_its_previous_copy_when_reexporting_inside_workspace() {
    let ws = synthetic_workspace();
    let ws_str = ws.path().to_string_lossy().to_string();
    let mut c = Client::start(Some(&ws_str));
    c.initialize();

    for id in [34, 35] {
        let resp = c.call(
            id,
            "workspace",
            json!({
                "accion": "exportar", "destino": "origen.zip",
                "sobrescribir": true, "workspace": ws_str,
            }),
        );
        assert_eq!(resp["result"]["isError"], json!(false), "resp={resp}");
    }

    let zip = std::fs::read(ws.path().join("origen.zip")).expect("leer zip");
    let contiene_a_si_mismo = zip.windows(b"origen.zip".len()).any(|w| w == b"origen.zip");
    assert!(!contiene_a_si_mismo, "el zip se incluyó a sí mismo");
}
