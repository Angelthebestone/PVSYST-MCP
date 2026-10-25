use super::cliente_mcp::{tool_text, Client};
use serde_json::json;

fn window_classes(c: &mut Client) -> Vec<String> {
    let resp = c.call(
        24,
        "gui_observar",
        json!({ "que": "ventanas", "visibles": true }),
    );
    assert_eq!(resp["result"]["isError"], false, "resp={resp}");
    tool_text(&resp)["windows"]
        .as_array()
        .unwrap()
        .iter()
        .map(|w| w["class"].as_str().unwrap().to_string())
        .collect()
}

#[test]
#[ignore = "needs the PVsyst GUI to be closed: run with --ignored"]
fn gui_tools_report_when_pvsyst_not_running() {
    let mut c = Client::start_instalacion_real(None);
    let resp = c.call(21, "gui_observar", json!({ "que": "ventanas" }));
    assert_eq!(resp["result"]["isError"], true);
    let msg = resp["result"]["content"][0]["text"].as_str().unwrap();
    assert!(
        msg.contains("PVsyst no está en ejecución"),
        "unexpected error message: {msg}"
    );
}

#[test]
#[ignore = "needs a running PVsyst GUI: run with --ignored"]
fn gui_automation_drives_running_pvsyst() {
    let mut c = Client::start_instalacion_real(None);

    let classes = window_classes(&mut c);
    assert!(
        classes
            .iter()
            .any(|c| c == "THomeForm" || c.starts_with('T')),
        "expected VCL windows, got {classes:?}"
    );

    let resp = c.call(
        23,
        "gui_observar",
        json!({ "que": "controles", "ventana": "THomeForm" }),
    );
    assert_eq!(resp["result"]["isError"], false, "resp={resp}");
    let ctrls = tool_text(&resp);
    let n = ctrls["count"].as_u64().unwrap();
    assert!(n >= 5, "expected several controls, got {n}");
    let labels: Vec<String> = ctrls["controls"]
        .as_array()
        .unwrap()
        .iter()
        .map(|w| w["title"].as_str().unwrap_or("").to_string())
        .collect();
    assert!(
        labels.iter().any(|l| !l.is_empty()),
        "expected labelled controls"
    );
}

#[test]
#[ignore = "needs a running PVsyst GUI: run with --ignored"]
fn gui_invokes_button_and_opens_window() {
    let mut c = Client::start_instalacion_real(None);
    for dlg in ["TDEMOForm", "TToolsForm"] {
        let _ = c.call(
            27,
            "gui_actuar",
            json!({ "accion": "cerrar", "ventana": dlg }),
        );
    }
    std::thread::sleep(std::time::Duration::from_millis(500));
    let before = window_classes(&mut c);

    let resp = c.call(
        25,
        "gui_actuar",
        json!({ "accion": "click", "ventana": "Herramientas" }),
    );
    assert_eq!(resp["result"]["isError"], false, "resp={resp}");

    let mut opened = false;
    for _ in 0..10 {
        std::thread::sleep(std::time::Duration::from_millis(300));
        let after = window_classes(&mut c);
        if after.iter().any(|cls| !before.contains(cls)) {
            opened = true;
            break;
        }
    }
    assert!(opened, "expected a new window after clicking Herramientas");

    let after = window_classes(&mut c);
    for class in after {
        if !before.contains(&class) {
            let _ = c.call(
                26,
                "gui_actuar",
                json!({ "accion": "cerrar", "ventana": class }),
            );
        }
    }
}
