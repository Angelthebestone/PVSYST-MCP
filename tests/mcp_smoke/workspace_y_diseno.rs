use super::cliente_mcp::{synthetic_workspace, tool_text, Client};
use serde_json::json;

#[test]
fn should_restore_previous_content_when_historial_targets_the_requested_workspace() {
    let base = synthetic_workspace();
    let target = tempfile::tempdir().expect("temp dir");

    let mut c = Client::start(Some(base.path().to_str().unwrap()));
    let mut id = String::new();
    for contenido in ["uno", "dos"] {
        let resp = c.call(
            15,
            "escribir_archivo",
            json!({
                "archivo": "Notas/nota.txt",
                "contenido": contenido,
                "sobrescribir": true,
                "workspace": target.path().to_str().unwrap(),
            }),
        );
        assert_eq!(resp["result"]["isError"], false, "resp={resp}");
        let data = tool_text(&resp);
        if let Some(hist) = data.get("historial_id").and_then(|v| v.as_str()) {
            id = hist.to_string();
        }
    }
    assert!(
        !id.is_empty(),
        "segunda escritura debió snapshotear la primera"
    );

    let resp = c.call(
        16,
        "workspace",
        json!({ "accion": "historial", "workspace": target.path().to_str().unwrap() }),
    );
    assert_eq!(resp["result"]["isError"], false, "resp={resp}");
    let snapshots = tool_text(&resp)["snapshots"]
        .as_array()
        .cloned()
        .unwrap_or_default();
    assert_eq!(
        snapshots.len(),
        2,
        "la creación y la sobrescritura dejan un snapshot cada una: {snapshots:?}"
    );

    let resp = c.call(
        17,
        "workspace",
        json!({ "accion": "restaurar", "id": id, "workspace": target.path().to_str().unwrap() }),
    );
    assert_eq!(resp["result"]["isError"], false, "resp={resp}");
    let nota = std::fs::read_to_string(target.path().join("Notas").join("nota.txt")).expect("nota");
    assert_eq!(nota, "uno");
}
