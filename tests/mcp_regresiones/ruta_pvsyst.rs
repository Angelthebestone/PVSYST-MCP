use super::cliente_mcp::{error_text, synthetic_workspace, Client};
use serde_json::json;

#[test]
fn should_reject_workspace_override_on_p50_p90_pvsyst_route() {
    let configurado = synthetic_workspace();
    let otro = synthetic_workspace();
    let configurado_str = configurado.path().to_string_lossy().to_string();
    let otro_str = otro.path().to_string_lossy().to_string();
    let mut c = Client::start(Some(&configurado_str));
    c.initialize();

    let resp = c.call(
        30,
        "resultados",
        json!({
            "que": "p50_p90",
            "proyecto": "SYNTH_Plant",
            "variante": "VC0",
            "workspace": otro_str,
        }),
    );
    let msg = error_text(&resp);
    assert!(msg.contains("no admite 'workspace'"), "msg={msg}");
}

#[test]
fn should_reject_workspace_override_on_pvgis_horizon_pvsyst_route() {
    let configurado = synthetic_workspace();
    let otro = synthetic_workspace();
    let configurado_str = configurado.path().to_string_lossy().to_string();
    let otro_str = otro.path().to_string_lossy().to_string();
    let mut c = Client::start(Some(&configurado_str));
    c.initialize();

    let resp = c.call(
        31,
        "sitio",
        json!({
            "accion": "horizonte_pvgis",
            "proyecto": "SYNTH_Plant",
            "workspace": otro_str,
        }),
    );
    let msg = error_text(&resp);
    assert!(msg.contains("no admite 'workspace'"), "msg={msg}");
}
