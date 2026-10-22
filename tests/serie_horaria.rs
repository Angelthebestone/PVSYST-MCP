//! Integration tests for hourly series reading and writing via MCP tools.
#![expect(
    clippy::indexing_slicing,
    clippy::too_many_arguments,
    reason = "deuda medida; se limpia en el paso 2 de docs/handoff_limpieza_optimizacion.md"
)]
#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    unreachable_pub
)]

mod cliente_mcp;

use cliente_mcp::tool_text;
use pvsyst_mcp::Server;
use serde_json::json;
use std::fs;
use tempfile::tempdir;

const DEMO_MET: &str =
    r"C:\Program Files\PVsyst8.1.7\DataRO\PVsyst8.1_Data\Meteo\Abu Dhabi DEMO Project_MN82_SYN.MET";
const DEMO_DAM: &str = r"C:\Program Files\PVsyst8.1.7\DataRO\PVsyst8.1_Data\DataMeas\N13GlobH.DAM";

fn server_con_workspace(ws: &std::path::Path) -> Server {
    let mut paths = pvsyst_mcp::Paths::resolve();
    paths.workspace = ws.to_path_buf();
    paths.aproximaciones = false;
    Server::new(paths)
}

fn call_tool(server: &Server, id: u64, name: &str, args: &serde_json::Value) -> serde_json::Value {
    let req = json!({
        "jsonrpc": "2.0",
        "id": id,
        "method": "tools/call",
        "params": {
            "name": name,
            "arguments": args
        }
    });
    let out = server.handle_line(&req.to_string()).expect("response");
    let resp: serde_json::Value = serde_json::from_str(&out).expect("valid json");
    tool_text(&resp)
}

#[test]
fn should_read_hourly_series_via_mcp_leer_when_requested() {
    if !std::path::Path::new(DEMO_MET).exists() {
        return;
    }
    let tmp = tempdir().expect("tempdir");
    let server = server_con_workspace(tmp.path());

    let res = call_tool(
        &server,
        1,
        "leer",
        &json!({
            "archivo": "Abu Dhabi DEMO Project_MN82_SYN.MET",
            "serie": true,
            "offset": 0,
            "limite": 24
        }),
    );

    assert_eq!(res["total_horas"], 8760);
    assert_eq!(res["offset"], 0);
    assert_eq!(res["limite"], 24);
    assert!(res["variables"]
        .as_array()
        .expect("vars")
        .contains(&json!("GlobHor")));
    assert_eq!(res["unidades"]["GlobHor"], "W/m²");
    assert_eq!(res["unidades"]["T_Amb"], "°C");

    let horas = res["horas"].as_array().expect("horas array");
    assert_eq!(horas.len(), 24);
    assert_eq!(horas[0]["hora"], 0);
    assert_eq!(horas[0]["dia"], "0101");
}

#[test]
fn should_preserve_default_leer_summary_when_serie_is_not_requested() {
    if !std::path::Path::new(DEMO_MET).exists() {
        return;
    }
    let tmp = tempdir().expect("tempdir");
    let meteo_dir = tmp.path().join("Meteo");
    fs::create_dir_all(&meteo_dir).expect("create Meteo");
    fs::copy(
        DEMO_MET,
        meteo_dir.join("Abu Dhabi DEMO Project_MN82_SYN.MET"),
    )
    .expect("copy");
    let server = server_con_workspace(tmp.path());
    let res = call_tool(
        &server,
        2,
        "leer",
        &json!({
            "archivo": "Abu Dhabi DEMO Project_MN82_SYN.MET"
        }),
    );

    assert!(res.get("monthly").is_some());
    assert!(res.get("SiteM").is_some());
    assert!(res.get("horas").is_none());
}

#[test]
fn should_read_dam_measured_series_via_mcp_leer() {
    if !std::path::Path::new(DEMO_DAM).exists() {
        return;
    }
    let tmp = tempdir().expect("tempdir");
    let server = server_con_workspace(tmp.path());

    let res = call_tool(
        &server,
        3,
        "leer",
        &json!({
            "archivo": "N13GlobH.DAM",
            "serie": true,
            "offset": 0,
            "limite": 12
        }),
    );

    assert_eq!(res["tipo"], "pvDataMeas");
    assert_eq!(res["total_horas"], 8760);
    let vars = res["variables"].as_array().expect("vars");
    assert!(vars.contains(&json!("T_Amb")));
    assert!(vars.contains(&json!("DiffHor")));
}

#[test]
fn should_write_hourly_series_via_sitio_escribir_serie_and_verify_with_history() {
    if !std::path::Path::new(DEMO_MET).exists() {
        return;
    }
    let tmp = tempdir().expect("tempdir");
    let meteo_dir = tmp.path().join("Meteo");
    fs::create_dir_all(&meteo_dir).expect("create Meteo dir");

    let dest_file = meteo_dir.join("Test_Custom.MET");
    fs::copy(DEMO_MET, &dest_file).expect("copy template");
    let server = server_con_workspace(tmp.path());

    let (custom_gh, custom_ta) = generar_datos_sinteticos();
    let write_res = call_tool(
        &server,
        4,
        "sitio",
        &json!({
            "accion": "escribir_serie",
            "archivo": "Test_Custom.MET",
            "variables": ["GlobHor", "T_Amb"],
            "valores": { "GlobHor": custom_gh, "T_Amb": custom_ta }
        }),
    );
    assert_eq!(write_res["total_horas"], 8760);
    assert!(write_res["historial_id"].is_string());

    let read_res = call_tool(
        &server,
        5,
        "leer",
        &json!({ "archivo": "Test_Custom.MET", "serie": true, "offset": 10, "limite": 5 }),
    );
    verificar_horas_leidas(read_res["horas"].as_array().expect("horas"));
}

fn generar_datos_sinteticos() -> (Vec<f64>, Vec<f64>) {
    let gh = (0..8760)
        .map(|h| {
            if (7..17).contains(&(h % 24)) {
                600.0
            } else {
                0.0
            }
        })
        .collect();
    let ta = (0..8760).map(|_| 22.5).collect();
    (gh, ta)
}

fn verificar_horas_leidas(horas: &[serde_json::Value]) {
    assert_eq!(horas.len(), 5);
    for h in horas {
        let gh = h["valores"]["GlobHor"].as_f64().expect("gh");
        let ta = h["valores"]["T_Amb"].as_f64().expect("ta");
        assert!((gh - 600.0).abs() <= 1.0, "gh was {gh}");
        assert!((ta - 22.5).abs() <= 0.1, "ta was {ta}");
    }
}
