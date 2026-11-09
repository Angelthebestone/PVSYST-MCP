//! `sitio crear` from a CSV reports the `.SIT` it expects, as the coordinates path does.

use super::cliente_mcp::{synthetic_workspace, tool_text, Client};
use serde_json::json;
use std::fs;

#[test]
fn should_report_the_expected_sit_when_creating_a_site_from_csv() {
    let ws = synthetic_workspace();
    fs::create_dir_all(ws.path().join("UserData")).expect("UserData");
    fs::write(
        ws.path().join("UserData").join("Site_Monthly.csv"),
        "Month;GHI;T\n1;100;20\n",
    )
    .expect("csv mensual");
    let ws_str = ws.path().to_string_lossy().to_string();
    let fake = env!("CARGO_BIN_EXE_fake_pvsystcli");
    let fixtures = concat!(env!("CARGO_MANIFEST_DIR"), "/tests/fixtures/cli");
    let mut c = Client::start_env(
        Some(&ws_str),
        &[
            ("PVSYST_CLI", fake),
            ("FAKE_PVSYST_FIXTURES", fixtures),
        ],
    );
    c.initialize();

    let resp = c.call(
        77,
        "sitio",
        json!({
            "accion": "crear", "origen": "csv", "nombre_sitio": "QA_SiteCSV",
            "latitud": 7.07, "longitud": -73.84,
            "weather_data_file": "UserData/Site_Monthly.csv",
        }),
    );
    let out = tool_text(&resp);
    let esperado = out["expected_sit"].as_str().unwrap_or_default();
    assert!(
        esperado.ends_with("Sites/QA_SiteCSV.SIT") || esperado.ends_with(r"Sites\QA_SiteCSV.SIT"),
        "informa el .SIT esperado: {out}"
    );
}
