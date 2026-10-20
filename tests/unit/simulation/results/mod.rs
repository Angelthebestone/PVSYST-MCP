use super::*;
use std::fs;

fn resultado(proyecto: &str, variante: &str) -> String {
    format!("Project;{proyecto}.PRJ;\nSimulation variant;{proyecto}.{variante};\ndate;E_Grid\n01/01/90 01:00;1\n")
}

#[test]
fn should_read_only_the_csv_whose_metadata_matches_project_and_variant() {
    let ws = tempfile::tempdir().expect("ws");
    let dir = ws.path().join("Results");
    fs::create_dir_all(&dir).expect("Results");
    fs::write(dir.join("a.csv"), resultado("MCP_E2E", "VC0")).expect("a");
    fs::write(dir.join("b.csv"), resultado("MCP_E2E", "VC1")).expect("b");
    fs::write(dir.join("c.csv"), resultado("OTRO", "VC0")).expect("c");

    let args =
        json!({ "workspace": ws.path(), "proyecto": "mcp_e2e", "variante": "vc0", "count": 5 });
    let out = leer_resultados(&Paths::resolve(), &args).expect("read");
    assert_eq!(out["count"], 1);
    assert!(out["results"][0]["path"]
        .as_str()
        .expect("path")
        .ends_with("a.csv"));
}

#[test]
fn should_reject_missing_csv_file_when_reading_results() {
    let ws = tempfile::tempdir().expect("ws");
    let args = json!({ "archivo": "no_existe.csv", "workspace": ws.path() });
    assert!(leer_resultados(&Paths::resolve(), &args).is_err());
}

#[test]
fn should_reject_results_dir_outside_workspace_by_default() {
    let ws = tempfile::tempdir().expect("ws");
    let fuera = tempfile::tempdir().expect("fuera");
    fs::write(fuera.path().join("x.csv"), resultado("P", "VC0")).expect("x");
    let args = json!({ "workspace": ws.path(), "dir": fuera.path() });
    assert!(leer_resultados(&Paths::resolve(), &args).is_err());
}
