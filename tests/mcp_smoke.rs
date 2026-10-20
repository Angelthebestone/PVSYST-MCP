//! End-to-end smoke tests: drive the compiled `pvsyst-mcp` server over stdio.
//! Tests needing a real PVsyst install skip (early return) when it is absent.

// Test code: failing loudly on unexpected state is the point.
#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::too_many_lines,
    clippy::needless_pass_by_value,
    unreachable_pub
)]

mod common;

use common::{synthetic_workspace, tool_text, Client, DEMO_WS};
use serde_json::{json, Value};

const DEMO_PAN: &str = r"C:\Program Files\PVsyst8.1.7\DataRO\PVsyst8.1_Data\ComposPV\PVModules\CX-M500_2450Wp_1_00_086.PAN";
const EXPERT: (&str, &str) = ("PVSYST_MCP_EXPERTO", "1");

fn tool_names(c: &mut Client, id: u64) -> Vec<String> {
    let list = c.request(id, "tools/list", json!({}));
    list["result"]["tools"]
        .as_array()
        .unwrap()
        .iter()
        .map(|t| t["name"].as_str().unwrap().to_string())
        .collect()
}

/// Copy one demo project (all its files) into an isolated temp workspace.
fn stage_project(tmp: &std::path::Path, name: &str) -> std::path::PathBuf {
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

fn window_classes(c: &mut Client) -> Vec<String> {
    // Only visible windows: VCL hides (not destroys) a closed form, so an
    // all-windows list would still contain it and mask "a dialog appeared".
    let resp = c.call(
        24,
        "gui_observar",
        json!({ "que": "ventanas", "visibles": true }),
    );
    tool_text(&resp)["windows"]
        .as_array()
        .unwrap()
        .iter()
        .map(|w| w["class"].as_str().unwrap().to_string())
        .collect()
}

#[test]
fn handshake_and_tool_list() {
    let mut c = Client::start(None);
    let init = c.initialize();
    assert_eq!(init["result"]["serverInfo"]["name"], "pvsyst-mcp");

    let names = tool_names(&mut c, 2);
    assert_eq!(names.len(), 12, "default surface: {names:?}");
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
    ] {
        assert!(
            names.contains(&expected.to_string()),
            "missing tool {expected}"
        );
    }
}

#[test]
fn expert_tools_are_hidden_by_default_and_exposed_with_flag() {
    let mut c = Client::start(None);
    c.initialize();
    let names = tool_names(&mut c, 2);
    for hidden in ["pvsyst_cli", "gui_observar", "gui_actuar"] {
        assert!(!names.contains(&hidden.to_string()), "{hidden} visible");
    }
    let resp = c.call(3, "gui_observar", json!({ "que": "ventanas" }));
    assert_eq!(resp["result"]["isError"], true, "resp={resp}");

    let mut e = Client::start_env(None, &[EXPERT]);
    e.initialize();
    let names = tool_names(&mut e, 2);
    assert_eq!(names.len(), 15, "expert surface: {names:?}");
    for expert in ["pvsyst_cli", "gui_observar", "gui_actuar"] {
        assert!(names.contains(&expert.to_string()), "missing {expert}");
    }
}

#[test]
fn resources_list_and_read() {
    let mut c = Client::start(None);
    c.initialize();
    let list = c.request(2, "resources/list", json!({}));
    assert_eq!(list["result"]["resources"].as_array().unwrap().len(), 4);

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
    // The workspace value must NOT be wrapped in literal quotes (that would be
    // passed verbatim to the child process and corrupt the path).
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
    // Work on a private copy of the workspace Projects folder.
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
fn cobertura_resource_lists_areas() {
    let mut c = Client::start_env(None, &[EXPERT]);
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
#[ignore = "needs the PVsyst GUI to be closed: run with --ignored"]
fn gui_tools_report_when_pvsyst_not_running() {
    // Without the GUI running, gui_observar must return a clear, non-panicking
    // error (proves the tool is wired and the process probe works).
    let mut c = Client::start_env(None, &[EXPERT]);
    let resp = c.call(21, "gui_observar", json!({ "que": "ventanas" }));
    assert_eq!(resp["result"]["isError"], true);
    let msg = resp["result"]["content"][0]["text"].as_str().unwrap();
    assert!(msg.contains("not running"), "msg={msg}");
}

#[test]
#[ignore = "needs a running PVsyst GUI: run with --ignored"]
fn gui_automation_drives_running_pvsyst() {
    let mut c = Client::start_env(None, &[EXPERT]);

    // Enumerate the desktop app's windows.
    let resp = c.call(
        22,
        "gui_observar",
        json!({ "que": "ventanas", "visibles": true }),
    );
    assert_eq!(resp["result"]["isError"], false, "resp={resp}");
    let wins = tool_text(&resp);
    let classes: Vec<String> = wins["windows"]
        .as_array()
        .unwrap()
        .iter()
        .map(|w| w["class"].as_str().unwrap().to_string())
        .collect();
    assert!(
        classes
            .iter()
            .any(|c| c == "THomeForm" || c.starts_with('T')),
        "expected VCL windows, got {classes:?}"
    );

    // Enumerate the home form's controls (real buttons).
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
    let mut c = Client::start_env(None, &[EXPERT]);
    // Make the test idempotent: close any modal/dialog a previous run or the
    // launch left open (TDEMOForm is the trial/demo dialog shown at startup).
    for dlg in ["TDEMOForm", "TToolsForm"] {
        let _ = c.call(
            27,
            "gui_actuar",
            json!({ "accion": "cerrar", "ventana": dlg }),
        );
    }
    std::thread::sleep(std::time::Duration::from_millis(500));
    let before = window_classes(&mut c);

    // Invoke the 'Herramientas' (Tools) button on the home form. This is a
    // GUI-only function with no CLI/file equivalent.
    let resp = c.call(
        25,
        "gui_actuar",
        json!({ "accion": "click", "ventana": "Herramientas" }),
    );
    assert_eq!(resp["result"]["isError"], false, "resp={resp}");
    std::thread::sleep(std::time::Duration::from_secs(3));

    let after = window_classes(&mut c);
    let opened: Vec<String> = after
        .iter()
        .filter(|k| !before.contains(k))
        .cloned()
        .collect();
    assert!(
        !opened.is_empty(),
        "invoking a GUI button produced no new window (before={before:?} after={after:?})"
    );

    // Close whatever new dialog opened, to leave the app clean.
    for class in &opened {
        if class.starts_with('T') {
            let _ = c.call(
                26,
                "gui_actuar",
                json!({ "accion": "cerrar", "ventana": class }),
            );
        }
    }
    std::thread::sleep(std::time::Duration::from_millis(600));
}

/// Export a bundled inverter and import it back into the temp workspace.
fn component_roundtrip(c: &mut Client, tmp: &std::path::Path, ws_dir: &std::path::Path) {
    let src = common::fixture("workspace/ComposPV/Inverters/Synth_10kW.OND");
    let src = src.to_string_lossy();
    let outp = tmp.join("exported.OND");
    let r = c.call(
        44,
        "componente",
        json!({
            "accion": "exportar",
            "componente": src, "tipo": "OND", "destino": outp.to_string_lossy(),
            "permitir_fuera_del_workspace": true
        }),
    );
    assert_eq!(r["result"]["isError"], false, "resp={r}");
    assert!(outp.exists());

    let r = c.call(
        45,
        "componente",
        json!({
            "accion": "importar",
            "origen": outp.to_string_lossy(), "tipo": "OND", "nombre_nuevo": "R.OND",
            "permitir_fuera_del_workspace": true
        }),
    );
    assert_eq!(r["result"]["isError"], false, "resp={r}");
    assert!(ws_dir
        .join("ComposPV")
        .join("Inverters")
        .join("R.OND")
        .exists());
}

/// Without `permitir_fuera_del_workspace`, a destination outside the workspace is refused.
fn destination_outside_workspace_is_rejected(c: &mut Client, tmp: &std::path::Path) {
    let fuera = tmp.join("Param_Modif.dat");
    let r = c.call(
        41,
        "simular",
        json!({
            "accion": "preparar",
            "parametros": { "pi_TimeShiftMax": 40 },
            "destino": fuera.to_string_lossy()
        }),
    );
    assert_eq!(r["result"]["isError"], true, "resp={r}");
    assert!(!fuera.exists());
}

#[test]
fn admin_tools_generate_files_and_roundtrip_component() {
    let tmp = std::env::temp_dir().join(format!("pvsyst-mcp-admin-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&tmp);
    std::fs::create_dir_all(&tmp).unwrap();

    // The client's workspace IS the one we create, so imports land in it.
    let ws_dir = tmp.join("PVsyst8.1_Data");
    let ws_str = ws_dir.to_string_lossy().to_string();
    let mut c = Client::start(Some(ws_str.as_str()));

    // create workspace
    let r = c.call(
        40,
        "workspace",
        json!({ "accion": "crear", "ruta": ws_str }),
    );
    assert_eq!(r["result"]["isError"], false, "resp={r}");
    assert!(ws_dir.join("Projects").is_dir());
    assert!(ws_dir.join("ComposPV").join("Inverters").is_dir());

    destination_outside_workspace_is_rejected(&mut c, &tmp);

    // Param_Modif.dat via simular preparar
    let pm = ws_dir.join("Param_Modif.dat");
    let r = c.call(
        41,
        "simular",
        json!({
            "accion": "preparar",
            "parametros": { "pi_TimeShiftMax": 40 },
            "unidades": { "pi_TimeShiftMax": "min" },
            "destino": pm.to_string_lossy()
        }),
    );
    assert_eq!(r["result"]["isError"], false, "resp={r}");
    let txt = std::fs::read_to_string(&pm).unwrap();
    assert!(txt.starts_with("Hidden parameters: User's modifications;"));
    assert!(txt.contains("pi_TimeShiftMax;0.0000;40;min"));

    // batch file (the base variant must exist; stub files are enough)
    let projects = ws_dir.join("Projects");
    std::fs::create_dir_all(&projects).unwrap();
    std::fs::write(
        projects.join("_DEMO_Residential.PRJ"),
        "PVObject_=pvProject\n",
    )
    .unwrap();
    std::fs::write(
        projects.join("_DEMO_Residential.VC0"),
        "PVObject_=pvVCalcul\n",
    )
    .unwrap();
    let missing = c.call(
        42,
        "simular",
        json!({
            "accion": "preparar",
            "proyecto": "_DEMO_Residential", "variante": "VC7", "filas": [ { "tilt": 25 } ]
        }),
    );
    assert_eq!(missing["result"]["isError"], true, "resp={missing}");
    let bc = ws_dir.join("Batch.csv");
    let r = c.call(43, "simular", json!({
        "accion": "preparar",
        "proyecto": "_DEMO_Residential", "variante": "VC0",
        "filas": [ { "tilt": 25, "azim": 20, "pan": "X.PAN", "n_mod_serie": 15, "n_strings": 2 },
                   { "tilt": 30, "azim": 20, "pan": "X.PAN", "n_mod_serie": 15, "n_strings": 2 } ],
        "variables": ["E_Grid", "PR"],
        "destino": bc.to_string_lossy()
    }));
    assert_eq!(r["result"]["isError"], false, "resp={r}");
    let btxt = std::fs::read_to_string(&bc).unwrap();
    assert!(btxt.contains("SIM_1"));
    assert!(btxt.contains("SIM_2"));
    assert!(btxt.contains("Project;;;_DEMO_Residential.PRJ"));

    component_roundtrip(&mut c, &tmp, &ws_dir);

    // CLI-backed admin: dry runs are instant and hermetic
    let r = c.call(
        46,
        "entorno",
        json!({ "accion": "licencia", "dry_run": true }),
    );
    // The CLI path depends on the machine; only the command itself is hermetic.
    let cmd = tool_text(&r)["command_line"]
        .as_str()
        .unwrap_or("")
        .to_string();
    assert!(cmd.ends_with(" lic-info"), "cmd={cmd}");
    let r = c.call(
        47,
        "entorno",
        json!({ "accion": "exportar_logs", "dry_run": true }),
    );
    assert_eq!(tool_text(&r)["dry_run"], true);

    let _ = std::fs::remove_dir_all(&tmp);
}

#[test]
fn entorno_licencia_activar_returns_template_only() {
    let mut c = Client::start(None);
    let resp = c.call(48, "entorno", json!({ "accion": "licencia_activar" }));
    assert_eq!(resp["result"]["isError"], false, "resp={resp}");
    let cmd = tool_text(&resp)["comando"].as_str().unwrap().to_string();
    assert!(cmd.contains("<TU_CLAVE>"), "cmd={cmd}");
}

#[test]
fn pvsyst_cli_dry_run_builds_command() {
    let mut c = Client::start_env(None, &[EXPERT]);
    let resp = c.call(
        8,
        "pvsyst_cli",
        json!({ "comando": "run-simulation",
                "opciones": ["-w:C:\\ws", "-p:X.PRJ", "-v:VC0", "-rpf:C:\\out.pdf"],
                "dry_run": true }),
    );
    assert_eq!(resp["result"]["isError"], false, "resp={resp}");
    let data = tool_text(&resp);
    let cmd = data["command_line"].as_str().unwrap();
    assert!(cmd.contains("run-simulation"));
    assert!(cmd.contains("-rpf:C:\\out.pdf"));
}

#[test]
fn sitio_crear_dry_run_builds_command() {
    let mut c = Client::start(None);
    let resp = c.call(
        9,
        "sitio",
        json!({ "accion": "crear", "origen": "csv", "nombre_sitio": "S",
                "latitud": 46.2, "longitud": 6.1,
                "weather_data_file": "w.csv", "dry_run": true }),
    );
    assert_eq!(resp["result"]["isError"], false, "resp={resp}");
    let data = tool_text(&resp);
    let cmd = data["command_line"].as_str().unwrap();
    assert!(cmd.contains("create-site"));
    assert!(cmd.contains("-lat:46.2"), "cmd={cmd}"); // numeric arg accepted
}

#[test]
fn workspace_historial_and_restore_honor_the_workspace_argument() {
    let base = synthetic_workspace();
    let target = tempfile::tempdir().expect("temp dir");

    let mut c = Client::start(Some(base.path().to_str().unwrap()));
    // The second write snapshots the first one inside the target workspace.
    let mut id = String::new();
    for contenido in ["uno", "dos"] {
        let resp = c.call(
            14,
            "escribir_archivo",
            json!({ "archivo": "Notas/nota.txt", "contenido": contenido,
                    "workspace": target.path().to_str().unwrap() }),
        );
        assert_eq!(resp["result"]["isError"], false, "resp={resp}");
        if let Some(v) = tool_text(&resp)["historial_id"].as_str() {
            id = v.to_string();
        }
    }
    assert!(!id.is_empty(), "la segunda escritura deja snapshot");

    let resp = c.call(
        15,
        "workspace",
        json!({ "accion": "historial", "n": 5, "workspace": target.path().to_str().unwrap() }),
    );
    assert_eq!(resp["result"]["isError"], false, "resp={resp}");
    let lista = tool_text(&resp);
    assert_eq!(lista["total"].as_u64(), Some(1), "lista={lista}");
    assert_eq!(
        lista["snapshots"][0]["id"].as_str(),
        Some(id.as_str()),
        "lista={lista}"
    );

    let resp = c.call(
        16,
        "workspace",
        json!({ "accion": "restaurar", "id": id, "workspace": target.path().to_str().unwrap() }),
    );
    assert_eq!(resp["result"]["isError"], false, "resp={resp}");
    let nota = std::fs::read_to_string(target.path().join("Notas").join("nota.txt")).expect("nota");
    assert_eq!(nota, "uno");
}

#[test]
#[ignore = "needs the NASA grids from a PVsyst install: run with --ignored"]
fn sitio_crear_coordenadas_honors_the_workspace_argument() {
    let target = tempfile::tempdir().expect("temp dir");
    let mut c = Client::start(None);
    let resp = c.call(
        17,
        "sitio",
        json!({ "accion": "crear", "origen": "coordenadas",
                "latitud": 46.2, "longitud": 6.1, "nombre_sitio": "MCP_Tmp_Site",
                "dry_run": true, "workspace": target.path().to_str().unwrap() }),
    );
    assert_eq!(resp["result"]["isError"], false, "resp={resp}");
    let csv = target
        .path()
        .join("UserData")
        .join("MCP_Tmp_Site_Monthly.csv");
    assert!(
        csv.exists(),
        "el CSV mensual debe quedar en el workspace pedido"
    );
    assert!(tool_text(&resp)["create_site"]["command_line"]
        .as_str()
        .unwrap()
        .contains("create-site"));
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

    let mut c = Client::start(Some(tmp.to_str().unwrap()));
    let resp = c.call(
        11,
        "proyecto",
        json!({ "accion": "dimensionar", "proyecto": "_DEMO_Residential", "kwp": 300,
                "pnom_modulo": 300, "n_mod_serie": 15 }),
    );
    assert_eq!(resp["result"]["isError"], false, "resp={resp}");
    let data = tool_text(&resp);
    // 300 kWp / 300 W = 1000 modules; /15 series = 67 strings; 15*67*300 = 301.5 kWp
    assert_eq!(data["n_strings"].as_u64().unwrap(), 67);
    let vc = std::fs::read_to_string(tmp.join("Projects").join("_DEMO_Residential.VC0")).unwrap();
    assert!(vc.contains("NStringCh=67"), "sizing not written");
    assert!(vc.contains("NModSerie=15"));

    let _ = std::fs::remove_dir_all(&tmp);
}
