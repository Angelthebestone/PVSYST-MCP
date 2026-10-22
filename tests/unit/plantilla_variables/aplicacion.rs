use super::*;
use tempfile::TempDir;

fn workspace_temporal() -> (TempDir, Paths) {
    let tmp = TempDir::new().expect("temp dir");
    let mut paths = Paths::resolve();
    paths.workspace = tmp.path().to_path_buf();
    (tmp, paths)
}

#[test]
fn should_return_catalogue_when_operacion_catalogo_requested() {
    let (_tmp, paths) = workspace_temporal();
    let args = json!({
        "operacion": "catalogo"
    });
    let res = gestionar_rvt(&paths, &args).expect("catalogo");
    assert_eq!(res["ok"], true);
    assert_eq!(res["total"], 122);
}

#[test]
fn should_create_and_list_rvt_file() {
    let (_tmp, paths) = workspace_temporal();
    let rvt_nombre = "TestVariables.RVT";
    let args_crear = json!({
        "archivo": rvt_nombre,
        "operacion": "crear",
        "variables": ["HSol", "AzSol", "E_Grid"]
    });
    let res_crear = gestionar_rvt(&paths, &args_crear).expect("crear");
    assert_eq!(res_crear["ok"], true);
    assert_eq!(res_crear["total"], 3);

    let args_listar = json!({
        "archivo": rvt_nombre,
        "operacion": "listar"
    });
    let res_listar = gestionar_rvt(&paths, &args_listar).expect("listar");
    assert_eq!(res_listar["total"], 3);
    assert_eq!(res_listar["variables"], json!(["HSol", "AzSol", "E_Grid"]));
}

#[test]
fn should_add_variables_with_history() {
    let (_tmp, paths) = workspace_temporal();
    let rvt_nombre = "TestAdd.RVT";
    let args_crear = json!({
        "archivo": rvt_nombre,
        "operacion": "crear",
        "variables": ["HSol", "AzSol"]
    });
    gestionar_rvt(&paths, &args_crear).expect("crear");

    let args_anadir = json!({
        "archivo": rvt_nombre,
        "operacion": "anadir",
        "variables": ["E_Grid", "PR"]
    });
    let res_anadir = gestionar_rvt(&paths, &args_anadir).expect("anadir");
    assert_eq!(res_anadir["total"], 4);
    assert!(res_anadir.get("historial_id").is_some());
}

#[test]
fn should_remove_variables_with_history() {
    let (_tmp, paths) = workspace_temporal();
    let rvt_nombre = "TestRemove.RVT";
    let args_crear = json!({
        "archivo": rvt_nombre,
        "operacion": "crear",
        "variables": ["HSol", "AzSol", "E_Grid", "PR"]
    });
    gestionar_rvt(&paths, &args_crear).expect("crear");

    let args_eliminar = json!({
        "archivo": rvt_nombre,
        "operacion": "eliminar",
        "variables": ["AzSol"]
    });
    let res = gestionar_rvt(&paths, &args_eliminar).expect("eliminar");
    assert_eq!(res["total"], 3);
    assert_eq!(res["variables"], json!(["HSol", "E_Grid", "PR"]));
}

#[test]
fn should_reorder_variables_with_history() {
    let (_tmp, paths) = workspace_temporal();
    let rvt_nombre = "TestReorder.RVT";
    let args_crear = json!({
        "archivo": rvt_nombre,
        "operacion": "crear",
        "variables": ["HSol", "AzSol", "PR"]
    });
    gestionar_rvt(&paths, &args_crear).expect("crear");

    let args_reordenar = json!({
        "archivo": rvt_nombre,
        "operacion": "reordenar",
        "variables": ["PR", "HSol", "AzSol"]
    });
    let res = gestionar_rvt(&paths, &args_reordenar).expect("reordenar");
    assert_eq!(res["variables"], json!(["PR", "HSol", "AzSol"]));
}

fn invocar_tool(server: &crate::Server, id: u64, name: &str, args: &Value) -> Value {
    let req = json!({
        "jsonrpc": "2.0",
        "id": id,
        "method": "tools/call",
        "params": { "name": name, "arguments": args }
    });
    let out = server.handle_line(&req.to_string()).expect("respuesta");
    let resp: Value = serde_json::from_str(&out).expect("json");
    assert!(
        !resp["result"]["isError"].as_bool().unwrap_or(false),
        "error en {name}: {resp}"
    );
    resp
}

#[test]
fn should_dispatch_via_simular_preparar() {
    let (_tmp, paths) = workspace_temporal();
    let server = crate::Server::new(paths);
    let args = json!({
        "accion": "preparar",
        "batch_rvt_file": "BatchVars.RVT",
        "operacion": "crear",
        "variables": ["HSol", "E_Grid"]
    });
    let resp = invocar_tool(&server, 1, "simular", &args);
    let text = resp["result"]["content"][0]["text"].as_str().expect("text");
    let parsed: Value = serde_json::from_str(text).expect("parsed");
    assert_eq!(parsed["ok"], true);
    assert_eq!(parsed["total"], 2);
}

#[test]
fn should_dispatch_via_editar_rvt() {
    let (_tmp, paths) = workspace_temporal();
    let server = crate::Server::new(paths);
    let args = json!({
        "accion": "rvt",
        "archivo": "EditVars.RVT",
        "operacion": "crear",
        "variables": ["GlobInc", "PR"]
    });
    let resp = invocar_tool(&server, 2, "editar", &args);
    let text = resp["result"]["content"][0]["text"].as_str().expect("text");
    let parsed: Value = serde_json::from_str(text).expect("parsed");
    assert_eq!(parsed["ok"], true);
    assert_eq!(parsed["total"], 2);
}
