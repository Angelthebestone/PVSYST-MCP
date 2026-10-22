use super::cliente_mcp::{tool_text, Client};
use serde_json::{json, Value};

fn tool_names(c: &mut Client, id: u64) -> Vec<String> {
    let list = c.request(id, "tools/list", json!({}));
    list["result"]["tools"]
        .as_array()
        .unwrap()
        .iter()
        .map(|t| t["name"].as_str().unwrap().to_string())
        .collect()
}

/// Valid `accion`/`que` values per tool, from the `tools/list` schemas.
fn discriminators(c: &mut Client, id: u64) -> std::collections::HashMap<String, Vec<String>> {
    let list = c.request(id, "tools/list", json!({}));
    let mut out = std::collections::HashMap::new();
    for t in list["result"]["tools"].as_array().unwrap() {
        let props = &t["inputSchema"]["properties"];
        for key in ["accion", "que"] {
            if let Some(values) = props[key]["enum"].as_array() {
                let names = values.iter().map(|v| v.as_str().unwrap().to_string());
                out.insert(t["name"].as_str().unwrap().to_string(), names.collect());
            }
        }
    }
    out
}

#[test]
fn handshake_and_tool_list() {
    let mut c = Client::start(None);
    let init = c.initialize();
    assert_eq!(init["result"]["serverInfo"]["name"], "pvsyst-mcp");

    let names = tool_names(&mut c, 2);
    assert_eq!(names.len(), 15, "default surface: {names:?}");
    for expected in [
        "entorno",
        "disenar_sistema",
        "listar",
        "leer",
        "editar",
        "escribir_archivo",
        "proyecto",
        "componente",
        "sitio",
        "simular",
        "resultados",
        "workspace",
        "pvsyst_cli",
        "gui_observar",
        "gui_actuar",
    ] {
        assert!(
            names.contains(&expected.to_string()),
            "missing tool {expected}"
        );
    }
}

#[test]
fn gui_and_cli_tools_are_visible_by_default() {
    let mut c = Client::start(None);
    c.initialize();
    let names = tool_names(&mut c, 2);
    for visible in ["pvsyst_cli", "gui_observar", "gui_actuar"] {
        assert!(names.contains(&visible.to_string()), "{visible} hidden");
    }
}

#[test]
fn resources_list_and_read() {
    let mut c = Client::start(None);
    c.initialize();
    let list = c.request(2, "resources/list", json!({}));
    assert_eq!(list["result"]["resources"].as_array().unwrap().len(), 5);

    let cli = c.request(3, "resources/read", json!({ "uri": "pvsyst://cli" }));
    let text = cli["result"]["contents"][0]["text"].as_str().unwrap();
    assert!(text.contains("PVsystCLI"), "cli resource empty");

    let bad = c.request(4, "resources/read", json!({ "uri": "pvsyst://nope" }));
    assert_eq!(bad["error"]["code"], -32602, "resp={bad}");
    assert!(bad["error"]["message"]
        .as_str()
        .unwrap()
        .contains("Recurso desconocido"));
}

#[test]
fn cobertura_resource_lists_areas() {
    let mut c = Client::start(None);
    c.initialize();
    let implemented = tool_names(&mut c, 20);

    let resp = c.request(7, "resources/read", json!({ "uri": "pvsyst://cobertura" }));
    let text = resp["result"]["contents"][0]["text"].as_str().unwrap();
    let data: Value = serde_json::from_str(text).unwrap();
    let areas = data["areas"].as_array().unwrap();
    assert!(areas.len() >= 12, "areas: {}", areas.len());

    // Every tool (and discriminator) referenced by the coverage map must really exist.
    let valid = discriminators(&mut c, 21);
    let mut referenced = 0;
    for area in areas {
        for t in area["tools"].as_array().unwrap() {
            let entry = t.as_str().unwrap();
            let mut parts = entry.split_whitespace();
            let name = parts.next().unwrap().to_string();
            referenced += 1;
            assert!(
                implemented.contains(&name),
                "coverage map references non-existent tool '{name}'"
            );
            if let Some(disc) = parts.next() {
                let value = disc.rsplit(':').next().unwrap();
                assert!(
                    valid
                        .get(&name)
                        .is_some_and(|v| v.iter().any(|x| x == value)),
                    "coverage map entry '{entry}' is not a valid tool/discriminator"
                );
            }
        }
    }
    assert!(referenced >= 25, "coverage map too thin: {referenced}");

    let joined = serde_json::to_string(areas).unwrap();
    assert!(joined.contains("Simulation"));
    assert!(joined.contains("Component database"));
    assert!(joined.contains("GUI-only"), "GUI-only area must be covered");
    assert!(joined.contains("gui_actuar"), "GUI tools must be mapped");
}

#[test]
fn entorno_licencia_activar_returns_template_only() {
    let mut c = Client::start(None);
    let resp = c.call(48, "entorno", json!({ "accion": "licencia_activar" }));
    assert_eq!(resp["result"]["isError"], false, "resp={resp}");
    let cmd = tool_text(&resp)["comando"].as_str().unwrap().to_string();
    assert!(cmd.contains("<TU_CLAVE>"), "cmd={cmd}");
}
