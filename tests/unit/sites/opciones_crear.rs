use super::*;
use crate::error::Error;
use serde_json::json;

#[test]
fn should_resolve_weather_and_output_files_inside_workspace() {
    let ws = tempfile::tempdir().expect("ws");
    std::fs::write(ws.path().join("clima.csv"), "x").expect("csv");
    let args = json!({ "weather_data_file": "clima.csv", "output_sit_file": "Sites/Nuevo.SIT" });

    let opciones = archivos_sitio(ws.path(), &args).expect("opciones");

    assert_eq!(
        opciones,
        [
            format!("-wdf:{}", ws.path().join("clima.csv").display()),
            format!("-osf:{}", ws.path().join("Sites/Nuevo.SIT").display()),
        ]
    );
}

#[test]
fn should_reject_output_site_outside_workspace_when_not_permitted() {
    let ws = tempfile::tempdir().expect("ws");
    let fuera = tempfile::tempdir().expect("fuera");
    std::fs::write(ws.path().join("clima.csv"), "x").expect("csv");
    let sit = fuera.path().join("Robado.SIT");
    let args =
        json!({ "weather_data_file": "clima.csv", "output_sit_file": sit.to_string_lossy() });

    let res = archivos_sitio(ws.path(), &args);

    assert!(matches!(res, Err(Error::Invalid(m)) if m.contains("fuera del workspace")));
}

#[test]
fn should_reject_missing_weather_or_options_file() {
    let ws = tempfile::tempdir().expect("ws");
    std::fs::write(ws.path().join("clima.csv"), "x").expect("csv");
    let casos = [
        json!({}),
        json!({ "weather_data_file": "no_existe.csv" }),
        json!({ "weather_data_file": "clima.csv", "cmd_options_file": "no_existe.cof" }),
    ];
    for args in casos {
        let res = archivos_sitio(ws.path(), &args);
        assert!(
            matches!(res, Err(Error::Invalid(_) | Error::MissingArg { .. })),
            "{args}: {res:?}"
        );
    }
}

#[test]
fn should_emit_only_present_optional_site_flags() {
    let opciones = opciones_sitio(&json!({ "altitud": 1200, "region": "", "timezone": null }));

    assert_eq!(opciones, ["-alt:1200"]);
}
