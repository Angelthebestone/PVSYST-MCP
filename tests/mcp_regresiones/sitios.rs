//! `sitio crear` reports the `.SIT` it expects, and a creation PVsystCLI could not
//! confirm never reaches the agent as a success.

use super::cliente_mcp::{synthetic_workspace, tool_text, Client};
use serde_json::json;
use std::fs;

const FAKE: &str = env!("CARGO_BIN_EXE_fake_pvsystcli");

/// Fixtures of the fake CLI, with the `create-site` answer replaced by `salida`.
fn cli_que_responde_crear_site(salida: &str) -> tempfile::TempDir {
    let dir = tempfile::tempdir().expect("fixtures CLI");
    for entrada in fs::read_dir(concat!(env!("CARGO_MANIFEST_DIR"), "/tests/fixtures/cli"))
        .expect("fixtures")
        .flatten()
    {
        fs::copy(entrada.path(), dir.path().join(entrada.file_name())).expect("fixture");
    }
    fs::write(dir.path().join("create-site.txt"), salida).expect("salida de create-site");
    dir
}

/// A `DataRO` with just the three NASA grids of the cell (10, -70).
fn data_ro() -> tempfile::TempDir {
    let dir = tempfile::tempdir().expect("DataRO");
    let grid = "10 -70 5.0 5.2 5.4 5.6 5.8 5.5 5.3 5.1 4.9 4.7 4.8 5.0\n";
    for archivo in ["Global_World.txt", "Diffuse_World.TXT", "Tamb10m_World.txt"] {
        fs::write(dir.path().join(archivo), grid).expect("grid");
    }
    dir
}

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
        &[("PVSYST_CLI", fake), ("FAKE_PVSYST_FIXTURES", fixtures)],
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

#[test]
fn should_not_report_ok_when_the_cli_could_not_confirm_the_site_creation() {
    // Given: a create-site that prints neither a confirmation nor an error marker, so
    // PVsystCLI's result stays ambiguous and no `.SIT` is promised
    let ws = synthetic_workspace();
    let data_ro = data_ro();
    let cli = cli_que_responde_crear_site("Checking license...\n");
    let mut c = Client::start_env(
        Some(&ws.path().to_string_lossy()),
        &[
            ("PVSYST_CLI", FAKE),
            ("FAKE_PVSYST_FIXTURES", &cli.path().to_string_lossy()),
            ("PVSYST_DATARO", &data_ro.path().to_string_lossy()),
        ],
    );
    c.initialize();

    // When: the site is created from coordinates
    let out = tool_text(&c.call(
        78,
        "sitio",
        json!({ "accion": "crear", "origen": "coordenadas", "nombre_sitio": "QA_SiteAmbigua",
                "latitud": 10.0, "longitud": -70.0 }),
    ));

    // Then: the vista reports the ambiguity, with the reason the CLI gave
    assert_eq!(
        out["create_site"]["clasificacion"],
        json!("ambiguo"),
        "out={out}"
    );
    assert_eq!(
        out["create_site"]["confirmed_by_output"],
        json!(false),
        "{out}"
    );
    assert_eq!(out["vista"]["estado"], json!("error"), "out={out}");
    let motivo = out["vista"]["error"]["motivo"].as_str().unwrap_or_default();
    assert!(motivo.contains("ambiguo"), "motivo={motivo}");
}
