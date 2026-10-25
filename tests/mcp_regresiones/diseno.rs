use super::cliente_mcp::{error_text, synthetic_workspace, tool_text, Client};
use serde_json::{json, Value};
use std::fs;

#[test]
fn should_reject_invalid_easy_design_inputs() {
    let ws = synthetic_workspace();
    let ws_str = ws.path().to_string_lossy().to_string();
    let mut c = Client::start(Some(&ws_str));
    c.initialize();

    let base = json!({ "latitud": 4.71, "longitud": -74.07, "kwp": 5.0 });
    let casos: [(Value, &str); 5] = [
        (json!({ "n_mod_serie": 4_294_967_297u64 }), "n_mod_serie"),
        (json!({ "n_strings": -1 }), "n_strings"),
        (json!({ "tilt": 91.0 }), "tilt"),
        (json!({ "tilt": "NaN" }), "tilt"),
        (json!({ "azimut": 181.0 }), "azimut"),
    ];
    for (i, (extra, clave)) in casos.iter().enumerate() {
        let mut args = base.clone();
        for (k, v) in extra.as_object().expect("objeto") {
            args[k.as_str()] = v.clone();
        }
        let resp = c.call(10 + i as u64, "disenar_sistema", args);
        let msg = error_text(&resp).to_lowercase();
        assert!(msg.contains(clave), "esperaba '{clave}' en: {msg}");
    }
}

#[test]
fn should_resolve_a_module_inside_the_requested_workspace() {
    let ws = synthetic_workspace(); // trae ComposPV/PVModules/Synth_400W.PAN
    let ws_str = ws.path().to_string_lossy().to_string();
    let mut c = Client::start(None);
    c.initialize();

    let resp = c.call(
        20,
        "leer",
        json!({ "archivo": "Synth_400W.PAN", "workspace": ws_str }),
    );
    assert_eq!(resp["result"]["isError"], json!(false), "resp={resp}");
    let data = tool_text(&resp);
    assert_eq!(data["pnom"], json!(400.0), "data={data}");
    let file = data["file"].as_str().expect("file");
    assert!(file.starts_with(&ws_str), "file={file}");
}

#[test]
fn should_copy_a_binary_component_without_field_changes() {
    let ws = synthetic_workspace();
    let ws_str = ws.path().to_string_lossy().to_string();
    let carpeta = ws.path().join("ComposPV").join("PVModules");
    let legacy: [u8; 5] = [0x00, 0x01, 0x02, 0xFF, 0x00];
    fs::write(carpeta.join("Legacy.PAN"), legacy).expect("legacy");

    let mut c = Client::start(Some(&ws_str));
    c.initialize();
    let resp = c.call(
        21,
        "componente",
        json!({
            "accion": "crear",
            "plantilla": carpeta.join("Legacy.PAN").to_string_lossy(),
            "nombre_nuevo": "Copia",
            "tipo": "PAN",
            "cambios": {},
        }),
    );
    assert_eq!(resp["result"]["isError"], json!(false), "resp={resp}");
    let creado = tool_text(&resp)["created"]
        .as_str()
        .expect("created")
        .to_string();
    assert!(creado.ends_with("Copia.PAN"), "created={creado}");
    assert_eq!(fs::read(carpeta.join("Copia.PAN")).expect("copia"), legacy);
}

#[test]
fn should_not_double_the_kind_extension_when_creating_from_a_datasheet() {
    let ws = synthetic_workspace(); // trae un .PAN de texto como plantilla
    let ws_str = ws.path().to_string_lossy().to_string();
    let mut c = Client::start_env(Some(&ws_str), &[("PVSYST_MCP_APROXIMACIONES", "1")]);
    c.initialize();

    let resp = c.call(
        22,
        "componente",
        json!({
            "accion": "crear_desde_ficha",
            "tipo": "PAN",
            "nombre_nuevo": "Ficha.PAN",
            "datos": { "Manufacturer": "MCP", "Model": "Ficha" },
            "via": "aproximacion",
        }),
    );
    assert_eq!(resp["result"]["isError"], json!(false), "resp={resp}");
    let data = tool_text(&resp);
    assert_eq!(data["origen"], "aproximacion_mcp");
    let archivo = data["archivo"].as_str().expect("archivo").to_string();
    assert!(archivo.ends_with("Ficha.PAN"), "archivo={archivo}");
    assert!(ws
        .path()
        .join("ComposPV")
        .join("PVModules")
        .join("Ficha.PAN")
        .is_file());
}

#[test]
fn should_bound_the_module_power_at_the_sizing_boundary() {
    let ws = synthetic_workspace(); // SYNTH_Plant.VC0 con NModSerie y PNomSizing
    let ws_str = ws.path().to_string_lossy().to_string();
    let mut c = Client::start_env(None, &[("PVSYST_MCP_APROXIMACIONES", "1")]);
    c.initialize();

    let fuera = c.call(
        30,
        "proyecto",
        json!({
            "accion": "dimensionar", "proyecto": "SYNTH_Plant", "variante": "VC0",
            "kwp": 10, "pnom_modulo": 4000, "workspace": ws_str, "via": "aproximacion",
        }),
    );
    let msg = error_text(&fuera);
    assert!(msg.contains("(0, 3000]"), "{msg}");
    assert!(!msg.contains("pase 'pnom_modulo'"), "{msg}");

    let valido = c.call(
        31,
        "proyecto",
        json!({
            "accion": "dimensionar", "proyecto": "SYNTH_Plant", "variante": "VC0",
            "kwp": 10, "pnom_modulo": 2450, "workspace": ws_str, "via": "aproximacion",
        }),
    );
    assert_eq!(valido["result"]["isError"], json!(false), "resp={valido}");
    assert_eq!(tool_text(&valido)["module_pnom_w"], json!(2450.0));
}

#[test]
fn should_report_preference_sections_as_counts() {
    let mut c = Client::start(None);
    c.initialize();
    let resp = c.call(50, "entorno", json!({ "accion": "preferencias" }));
    if resp["result"]["isError"] == json!(true) {
        return; // sin instalación de PVsyst no hay PVsyst.ini que leer
    }
    let data = tool_text(&resp);
    let secciones = data["secciones"].as_object().expect("secciones");
    assert!(!secciones.is_empty());
    for (nombre, valor) in secciones {
        assert!(valor.is_u64(), "sección {nombre} debe venir como conteo");
    }
    assert!(data["siguiente"]
        .as_str()
        .is_some_and(|s| s.contains("seccion")));
}

#[test]
fn should_keep_cli_logs_compact() {
    let mut c = Client::start(None);
    c.initialize();
    let resp = c.call(84, "entorno", json!({ "accion": "logs_cli", "n": 2 }));
    if resp["result"]["isError"] == json!(true) {
        return; // sin instalación de PVsyst no hay logs que resumir
    }
    let data = tool_text(&resp);
    for log in data["logs"].as_array().expect("logs") {
        let avisos = log["lineas_no_norm"].as_array().expect("lineas_no_norm");
        assert!(avisos.len() <= 6, "avisos: {avisos:?}");
        if let Some(total) = log["lineas_no_norm_total"].as_u64() {
            assert!(total >= avisos.len() as u64);
        }
        let ultimas = log["ultimas"].as_array().expect("ultimas");
        assert!(ultimas.len() <= 8, "cola larga: {}", ultimas.len());
        for linea in avisos.iter().chain(ultimas) {
            let texto = linea.as_str().expect("línea");
            assert!(texto.chars().count() <= 221, "línea sin recortar: {texto}");
        }
    }
}

// El SIT debe nacer en el workspace pedido; el TMY puede fallar por red y no
// importa para esta regresión.
#[test]
#[ignore = "needs the PVsyst CLI and NASA grids (consumes a trial execution): run with --ignored"]
fn should_create_the_meteo_site_inside_the_requested_workspace() {
    let ws = synthetic_workspace();
    let ws_str = ws.path().to_string_lossy().to_string();
    let mut c = Client::start(None);
    c.initialize();

    let resp = c.call(
        60,
        "sitio",
        json!({
            "accion": "meteo_pvgis", "latitud": 46.2044, "longitud": 6.1432,
            "nombre_sitio": "MCP_Meteo", "workspace": ws_str,
        }),
    );
    let sit = ws.path().join("Sites").join("MCP_Meteo.SIT");
    assert!(
        sit.is_file(),
        "el SIT debe nacer en el workspace pedido: {}; resp={resp}",
        sit.display()
    );
}

#[test]
#[ignore = "needs network (PVGIS) and the PVsyst demo data: run with --ignored"]
fn should_fill_the_horizon_of_a_project_in_a_foreign_workspace() {
    let ws = synthetic_workspace(); // SYNTH_Plant.VC0 sin pvHorizon
    let ws_str = ws.path().to_string_lossy().to_string();
    let mut c = Client::start_env(None, &[("PVSYST_MCP_APROXIMACIONES", "1")]);
    c.initialize();

    let resp = c.call(
        89,
        "sitio",
        json!({
            "accion": "horizonte_pvgis", "proyecto": "SYNTH_Plant", "workspace": ws_str,
        }),
    );
    assert_eq!(resp["result"]["isError"], json!(false), "resp={resp}");
    let data = tool_text(&resp);
    assert_eq!(data["origen"], "aproximacion_mcp");
    assert!(data["historial_id"].as_str().is_some_and(|s| !s.is_empty()));
    let var = data["variante"].as_str().expect("variante").to_string();
    assert!(var.starts_with(&ws_str), "variante={var}");
    let bytes = fs::read(&var).expect("variante editada");
    assert!(String::from_utf8_lossy(&bytes).contains("pvHorizon"));
}

#[test]
#[ignore = "needs network (PVGIS) and the PVsyst CLI (consumes a trial execution): run with --ignored"]
fn should_complete_meteo_with_pvgis_rounding_artifacts() {
    let ws = synthetic_workspace();
    let ws_str = ws.path().to_string_lossy().to_string();
    let mut c = Client::start(None);
    c.initialize();

    // 4.71/-74.07 reproduce el artefacto WS10m≈-0.16 que antes abortaba el flujo.
    let resp = c.call(
        90,
        "sitio",
        json!({
            "accion": "meteo_pvgis", "latitud": 4.71, "longitud": -74.07,
            "nombre_sitio": "MCP_Redondeo", "workspace": ws_str,
        }),
    );
    assert_eq!(resp["result"]["isError"], json!(false), "resp={resp}");
    let met = ws.path().join("Meteo").join("MCP_Redondeo_PVGIS.MET");
    assert!(met.is_file(), "MET publicado en el workspace pedido");
}

#[test]
fn default_client_rejects_launching_gui_with_missing_executable() {
    let mut c = Client::start(None);
    c.initialize();

    let resp = c.call(105, "gui_actuar", json!({ "accion": "lanzar" }));
    assert_eq!(resp["result"]["isError"], true, "resp={resp}");
    let msg = error_text(&resp);
    assert!(
        msg.contains("PVsyst GUI not found") || msg.contains("nonexistent_pvsyst_gui.exe"),
        "expected missing GUI error, got: {msg}"
    );
}
