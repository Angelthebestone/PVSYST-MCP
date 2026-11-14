use super::{copiar_con_nombre, exigir_nombre_libre};
use std::fs;

const SIT: &str = "\u{feff}Source=NASA SSE grid cell 7/-74\r\nLatitude=7.0708\r\nLongitude=-73.8385\r\nAltitude=100\r\n";

fn workspace_con_sitio(nombre: &str) -> tempfile::TempDir {
    let ws = tempfile::tempdir().expect("tempdir");
    let sites = ws.path().join("Sites");
    fs::create_dir_all(&sites).expect("Sites");
    fs::write(sites.join(format!("{nombre}.SIT")), SIT).expect("sit");
    ws
}

#[test]
fn should_refuse_an_explicit_site_name_that_already_exists() {
    let ws = workspace_con_sitio("Mi");
    let msg = exigir_nombre_libre(ws.path(), "Mi")
        .expect_err("name in use")
        .to_string();
    assert!(
        msg.contains("nombre_sitio") && msg.contains("forzar"),
        "msg={msg}"
    );
}

#[test]
fn should_copy_a_cached_site_under_the_requested_name() {
    let ws = workspace_con_sitio("Otro");
    let origen = ws.path().join("Sites").join("Otro.SIT");
    let existente = serde_json::json!({ "site": "Otro", "sit_file": origen.display().to_string() });
    let copia = copiar_con_nombre(ws.path(), &existente, "Mi").expect("copia");
    assert_eq!(copia["site"], "Mi");
    let escrito = fs::read_to_string(ws.path().join("Sites").join("Mi.SIT")).expect("Mi.SIT");
    assert_eq!(escrito, SIT);
}
