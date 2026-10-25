use super::cliente_mcp::{synthetic_workspace, tool_text, Client, DEMO_WS};
use serde_json::json;
use std::path::{Path, PathBuf};

const DEMO_PAN: &str = r"C:\Program Files\PVsyst8.1.7\DataRO\PVsyst8.1_Data\ComposPV\PVModules\CX-M500_2450Wp_1_00_086.PAN";

fn stage_project(tmp: &Path, name: &str) -> PathBuf {
    let projects = tmp.join("Projects");
    std::fs::create_dir_all(&projects).unwrap();
    for entry in std::fs::read_dir(format!(r"{DEMO_WS}\Projects"))
        .unwrap()
        .flatten()
    {
        let fname = entry.file_name().to_string_lossy().to_string();
        if fname.starts_with(name) {
            std::fs::copy(entry.path(), projects.join(&fname)).unwrap();
        }
    }
    projects
}

#[test]
#[ignore = "needs the PVsyst 8.1.7 demo data: run with --ignored"]
fn leer_modulo_decodes_legacy_pan() {
    let mut c = Client::start(None);
    let resp = c.call(3, "leer", json!({ "archivo": DEMO_PAN }));
    assert_eq!(resp["result"]["isError"], false);
    let data = tool_text(&resp);
    assert_eq!(data["format"], "legacy-real48");
    assert_eq!(data["vendor"], "Soitec");
    let pnom = data["pnom"].as_f64().unwrap();
    let voc = data["voc"].as_f64().unwrap();
    let isc = data["isc"].as_f64().unwrap();
    assert!((pnom - 2450.0).abs() < 0.5, "pnom was {pnom}");
    assert!((voc - 695.7).abs() < 1.0, "voc was {voc}");
    assert!((isc - 3.861).abs() < 0.01, "isc was {isc}");
}

#[test]
fn leer_decodes_the_synthetic_legacy_module() {
    let ws = synthetic_workspace();
    let mut c = Client::start(Some(ws.path().to_str().unwrap()));
    let resp = c.call(3, "leer", json!({ "archivo": "Synth_Legacy_300W.PAN" }));
    assert_eq!(resp["result"]["isError"], false, "resp={resp}");
    let data = tool_text(&resp);
    assert_eq!(data["format"], "legacy-real48");
    assert_eq!(data["vendor"], "SynthSolar");
    let pnom = data["pnom"].as_f64().unwrap();
    let voc = data["voc"].as_f64().unwrap();
    assert!((pnom - 300.0).abs() < 0.01, "pnom was {pnom}");
    assert!((voc - 40.0).abs() < 0.01, "voc was {voc}");
}

#[test]
#[ignore = "needs the PVsyst 8.1.7 demo data: run with --ignored"]
fn listar_proyectos_finds_demos() {
    let mut c = Client::start(Some(DEMO_WS));
    let resp = c.call(4, "listar", json!({ "que": "proyectos" }));
    assert_eq!(resp["result"]["isError"], false);
    let data = tool_text(&resp);
    let count = data["count"].as_u64().unwrap();
    assert!(count >= 1, "expected at least one demo project");
    let names: Vec<String> = data["projects"]
        .as_array()
        .unwrap()
        .iter()
        .map(|p| p["name"].as_str().unwrap().to_string())
        .collect();
    assert!(
        names.iter().any(|n| n.starts_with("_DEMO_")),
        "names: {names:?}"
    );
}

#[test]
#[ignore = "needs the PVsyst 8.1.7 demo data: run with --ignored"]
fn simular_builds_unquoted_arguments() {
    let mut c = Client::start(Some(DEMO_WS));
    let resp = c.call(
        6,
        "simular",
        json!({ "proyecto": "_DEMO_Residential", "variante": "VC0", "dry_run": true }),
    );
    assert_eq!(resp["result"]["isError"], false, "resp={resp}");
    let data = tool_text(&resp);
    assert_eq!(data["dry_run"], true);
    let cmd = data["command_line"].as_str().unwrap();
    assert!(cmd.contains("-w:"), "cmd was {cmd}");
    assert!(
        !cmd.contains("-w:\""),
        "workspace path must not be quoted: {cmd}"
    );
    assert!(cmd.contains("-p:_DEMO_Residential.PRJ") || cmd.contains("-p:_DEMO_Residential.PRJ "));
}

#[test]
#[ignore = "needs the PVsyst 8.1.7 demo data: run with --ignored"]
fn proyecto_crear_sets_module_reference() {
    let tmp = std::env::temp_dir().join(format!("pvsyst-mcp-test-{}", std::process::id()));
    let projects = tmp.join("Projects");
    std::fs::create_dir_all(&projects).unwrap();
    for entry in std::fs::read_dir(format!(r"{DEMO_WS}\Projects"))
        .unwrap()
        .flatten()
    {
        let name = entry.file_name().to_string_lossy().to_string();
        if name.starts_with("_DEMO_Commercial") {
            std::fs::copy(entry.path(), projects.join(&name)).unwrap();
        }
    }

    let mut c = Client::start(Some(tmp.to_str().unwrap()));
    let resp = c.call(
        5,
        "proyecto",
        json!({
            "accion": "crear",
            "plantilla": "_DEMO_Commercial",
            "nombre_nuevo": "mcp_tmp_variant",
            "modulo": "CX-M500_2450Wp_1_00_086.PAN",
            "sobrescribir": true
        }),
    );
    assert_eq!(resp["result"]["isError"], false, "resp={resp}");

    let vc = std::fs::read_to_string(projects.join("mcp_tmp_variant.VC0")).unwrap();
    assert!(vc.contains("PVModule=CX-M500_2450Wp_1_00_086.PAN"));
    assert!(!vc.contains("PVModule=Generic_Mono_440W_Half.PAN"));
    assert!(projects.join("mcp_tmp_variant.PRJ").exists());

    let limpia = c.call(6, "proyecto", json!({ "accion": "limpiar_temporales" }));
    assert_eq!(limpia["result"]["isError"], false, "resp={limpia}");
    let _ = std::fs::remove_dir_all(&tmp);
}

#[test]
#[ignore = "needs the PVsyst 8.1.7 demo data: run with --ignored"]
fn editar_objeto_preserves_bom_and_snapshots_history() {
    let tmp = std::env::temp_dir().join(format!("pvsyst-mcp-edit-{}", std::process::id()));
    let projects = stage_project(&tmp, "_DEMO_Residential");
    let prj = projects.join("_DEMO_Residential.PRJ");
    let before = std::fs::read(&prj).unwrap();
    assert_eq!(&before[..3], &[0xEF, 0xBB, 0xBF], "fixture has a UTF-8 BOM");

    let mut c = Client::start(Some(tmp.to_str().unwrap()));
    let resp = c.call(
        10,
        "editar",
        json!({ "accion": "campos",
                "archivo": "Projects\\_DEMO_Residential.PRJ",
                "objeto": "pvProject",
                "campos": { "Comment": "EditedByMCP" } }),
    );
    assert_eq!(resp["result"]["isError"], false, "resp={resp}");

    let after = std::fs::read(&prj).unwrap();
    assert_eq!(&after[..3], &[0xEF, 0xBB, 0xBF], "BOM preserved");
    let text = String::from_utf8_lossy(&after);
    assert!(text.contains("Comment=EditedByMCP"), "text not edited");

    let id = tool_text(&resp)["historial_id"]
        .as_str()
        .expect("historial_id in response")
        .to_string();
    let manifest = tmp
        .join("UserData")
        .join("mcp_history")
        .join(&id)
        .join("manifest.json");
    assert!(manifest.exists(), "missing snapshot manifest {manifest:?}");

    let _ = std::fs::remove_dir_all(&tmp);
}

#[test]
#[ignore = "needs the PVsyst 8.1.7 demo data: run with --ignored"]
fn proyecto_dimensionar_computes_capacity() {
    let tmp = std::env::temp_dir().join(format!("pvsyst-mcp-size-{}", std::process::id()));
    stage_project(&tmp, "_DEMO_Residential");

    let mut c = Client::start_env(
        Some(tmp.to_str().unwrap()),
        &[("PVSYST_MCP_APROXIMACIONES", "1")],
    );
    let resp = c.call(
        11,
        "proyecto",
        json!({ "accion": "dimensionar", "proyecto": "_DEMO_Residential", "kwp": 300,
                "pnom_modulo": 300, "n_mod_serie": 15 }),
    );
    assert_eq!(resp["result"]["isError"], false, "resp={resp}");
    let data = tool_text(&resp);
    assert_eq!(data["n_strings"].as_u64().unwrap(), 67);
    let vc = std::fs::read_to_string(tmp.join("Projects").join("_DEMO_Residential.VC0")).unwrap();
    assert!(vc.contains("NStringCh=67"), "sizing not written");
    assert!(vc.contains("NModSerie=15"));

    let _ = std::fs::remove_dir_all(&tmp);
}
