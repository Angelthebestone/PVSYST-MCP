//! `workspace exportar`/`importar`: relative paths anchor to the `workspace`
//! argument of the call, and the archive never contains its own temporary.

use super::cliente_mcp::{synthetic_workspace, Client};
use serde_json::json;
use std::fs;

#[test]
fn should_round_trip_a_relative_export_and_import_without_archiving_the_zip_itself() {
    let ws = synthetic_workspace();
    let ws_str = ws.path().to_string_lossy().to_string();
    let mut c = Client::start(None);
    c.initialize();

    let exportado = c.call(
        87,
        "workspace",
        json!({ "accion": "exportar", "destino": "QA_tmp/small.zip", "workspace": ws_str }),
    );
    assert_eq!(
        exportado["result"]["isError"],
        json!(false),
        "resp={exportado}"
    );
    let zip = ws.path().join("QA_tmp").join("small.zip");
    assert!(
        zip.is_file(),
        "el destino relativo se resuelve contra el workspace"
    );
    let bytes = fs::read(&zip).expect("zip");
    assert!(
        !bytes.windows(b".mcp_tmp_".len()).any(|v| v == b".mcp_tmp_"),
        "el zip no archiva su propio temporal"
    );

    let importado = c.call(
        88,
        "workspace",
        json!({
            "accion": "importar", "zip": "QA_tmp/small.zip",
            "destino": "QA_WS_Import", "workspace": ws_str,
        }),
    );
    assert_eq!(
        importado["result"]["isError"],
        json!(false),
        "resp={importado}"
    );
    assert_eq!(
        fs::read_to_string(
            ws.path()
                .join("QA_WS_Import")
                .join("Projects")
                .join("SYNTH_Plant.PRJ")
        )
        .expect("PRJ importado"),
        fs::read_to_string(ws.path().join("Projects").join("SYNTH_Plant.PRJ")).expect("fuente"),
        "la ida y vuelta con rutas relativas deja el mismo contenido"
    );
}
