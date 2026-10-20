//! Measured vs simulated using the non-PVsyst PDF/CSV fixture.

#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

mod common;

use common::{fixture, synthetic_workspace, tool_text, Client};
use serde_json::{json, Value};
use std::fs;

fn preparar() -> (tempfile::TempDir, Client) {
    let tmp = synthetic_workspace();
    let resultados = tmp.path().join("Results");
    fs::create_dir_all(&resultados).expect("Results");
    fs::copy(
        fixture("results/MCP_E2E.csv"),
        resultados.join("MCP_E2E.csv"),
    )
    .expect("csv");
    fs::copy(
        fixture("results/MCP_E2E.pdf"),
        resultados.join("MCP_E2E.pdf"),
    )
    .expect("pdf");
    let mut client = Client::start(Some(tmp.path().to_str().expect("workspace utf8")));
    client.initialize();
    (tmp, client)
}

fn simulada_kwh(mensual: &Value) -> Vec<f64> {
    let columnas = mensual["columnas"].as_array().expect("columnas");
    let indice = columnas
        .iter()
        .position(|columna| {
            columna
                .as_str()
                .is_some_and(|nombre| nombre.eq_ignore_ascii_case("E_Grid"))
        })
        .expect("columna E_Grid");
    let clave = columnas[indice].as_str().expect("nombre de columna");
    let unidad = mensual["unidades"][indice]
        .as_str()
        .expect("unidad de columna");
    let factor = if unidad.eq_ignore_ascii_case("MWh") {
        1000.0
    } else {
        1.0
    };
    mensual["meses"]
        .as_array()
        .expect("meses")
        .iter()
        .map(|mes| mes["valores"][clave].as_f64().expect("E_Grid mensual") * factor)
        .collect()
}

#[test]
fn should_compare_the_fixture_with_a_measured_series_from_an_array() {
    let (_tmp, mut client) = preparar();
    let mensual = client.call(
        1,
        "resultados",
        json!({
            "que": "mensual",
            "proyecto": "SYNTH_Plant",
            "archivo": "Results/MCP_E2E.csv"
        }),
    );
    let simulado = simulada_kwh(&tool_text(&mensual));
    let medido: Vec<f64> = simulado.iter().map(|valor| valor * 1.2).collect();

    let respuesta = client.call(
        2,
        "resultados",
        json!({
            "que": "medido_vs_simulado",
            "proyecto": "SYNTH_Plant",
            "archivo": "Results/MCP_E2E.csv",
            "medido_kwh": medido
        }),
    );
    let valor = tool_text(&respuesta);
    let meses = valor["meses"].as_array().expect("meses");
    assert_eq!(meses.len(), 12);
    for mes in meses {
        assert_eq!(mes["desviacion_pct"], 20.0, "{mes}");
    }
    assert_eq!(valor["anual"]["desviacion_pct"], 20.0);
    assert_eq!(valor["fuera_de_umbral"].as_array().map(Vec::len), Some(12));
}

#[test]
fn should_compare_the_fixture_with_a_measured_series_from_csv() {
    let (tmp, mut client) = preparar();
    let mensual = client.call(
        3,
        "resultados",
        json!({
            "que": "mensual",
            "proyecto": "SYNTH_Plant",
            "archivo": "Results/MCP_E2E.csv"
        }),
    );
    let simulado = simulada_kwh(&tool_text(&mensual));
    let filas: Vec<String> = simulado
        .iter()
        .enumerate()
        .map(|(i, valor)| format!("{};{:.3}", i + 1, valor * 1.2))
        .collect();
    fs::write(tmp.path().join("medido.csv"), filas.join("\n")).expect("csv medido");

    let respuesta = client.call(
        4,
        "resultados",
        json!({
            "que": "medido_vs_simulado",
            "proyecto": "SYNTH_Plant",
            "archivo": "Results/MCP_E2E.csv",
            "medido_csv": "medido.csv",
            "umbral_pct": 15
        }),
    );
    let valor = tool_text(&respuesta);
    assert_eq!(valor["fuente"], "medido_csv");
    assert_eq!(valor["umbral_pct"], 15.0);
    assert_eq!(valor["fuera_de_umbral"].as_array().map(Vec::len), Some(12));
}
