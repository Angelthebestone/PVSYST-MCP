//! An agent that only follows `vista.siguiente[0]` goes from creating a project
//! to simulating it, without reading free text or inventing an argument.

use super::cliente_mcp::{synthetic_workspace, tool_text, Client};
use serde_json::{json, Value};

const FAKE: &str = env!("CARGO_BIN_EXE_fake_pvsystcli");

fn fake_env() -> Vec<(&'static str, &'static str)> {
    vec![
        ("PVSYST_CLI", FAKE),
        (
            "FAKE_PVSYST_FIXTURES",
            concat!(env!("CARGO_MANIFEST_DIR"), "/tests/fixtures/cli"),
        ),
    ]
}

/// The `vista` of an answer: inside the JSON text of a success, or as the
/// second text item of a failure for clients without structured results.
fn vista_de(respuesta: &Value) -> Value {
    let texto = respuesta
        .pointer("/result/content")
        .and_then(Value::as_array)
        .and_then(|items| items.last())
        .and_then(|item| item.get("text"))
        .and_then(Value::as_str)
        .unwrap();
    let cuerpo: Value = serde_json::from_str(texto).unwrap();
    cuerpo.get("vista").cloned().unwrap()
}

fn texto_en(valor: &Value, ruta: &str) -> String {
    valor
        .pointer(ruta)
        .and_then(Value::as_str)
        .unwrap()
        .to_string()
}

/// Calls the first suggestion of each answer `saltos` times; returns tool and state of every call.
fn seguir_primera_sugerencia(
    client: &mut Client,
    primera: Value,
    saltos: u64,
) -> Vec<(String, String)> {
    let mut paso = primera;
    let mut recorrido = Vec::new();
    for id in 0..saltos {
        let tool = texto_en(&paso, "/tool");
        let argumentos = paso.get("argumentos").cloned().unwrap();
        let vista = vista_de(&client.call(10 + id, &tool, argumentos));
        recorrido.push((tool, texto_en(&vista, "/estado")));
        paso = vista.pointer("/siguiente/0").cloned().unwrap();
    }
    recorrido
}

#[test]
fn should_follow_the_first_next_step_from_creating_a_project_to_reading_results() {
    let ws = synthetic_workspace();
    std::fs::create_dir_all(ws.path().join("Results")).unwrap();
    let mut client = Client::start_env(Some(&ws.path().to_string_lossy()), &fake_env());
    client.initialize();

    let creada = tool_text(&client.call(
        2,
        "proyecto",
        json!({ "accion": "crear", "plantilla": "SYNTH_Plant", "nombre_nuevo": "Copia" }),
    ));
    let primera = creada.pointer("/vista/siguiente/0").cloned().unwrap();
    assert_eq!(texto_en(&creada, "/vista/estado"), "ok", "{creada}");

    let recorrido = seguir_primera_sugerencia(&mut client, primera, 3);

    let esperado = [
        ("proyecto", "ok"),
        ("simular", "ok"),
        ("resultados", "error"),
    ]
    .map(|(tool, estado)| (tool.to_string(), estado.to_string()));
    assert_eq!(
        recorrido, esperado,
        "the last call fails: the fake CLI writes no PDF"
    );
}
