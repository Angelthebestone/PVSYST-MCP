use super::{buscar_existente, coincide, copiar_con_nombre, exigir_nombre_libre, Busqueda};
use std::fs;

const SIT: &str = "\u{feff}Source=NASA SSE grid cell 7/-74\r\nLatitude=7.0708\r\nLongitude=-73.8385\r\nAltitude=100\r\n";

#[test]
fn should_match_when_coordinates_and_nasa_source_match() {
    assert!(coincide(SIT, (7.0708, -73.8385), None));
}

#[test]
fn should_not_match_when_latitude_differs() {
    assert!(!coincide(SIT, (7.0, -73.8385), None));
}

#[test]
fn should_only_compare_altitude_when_provided() {
    assert!(!coincide(SIT, (7.0708, -73.8385), Some(250.0)));
    assert!(coincide(SIT, (7.0708, -73.8385), Some(100.0)));
}

#[test]
fn should_not_reuse_sites_from_other_sources() {
    let otro = SIT.replace("NASA SSE grid cell", "Otro origen");
    assert!(!coincide(&otro, (7.0708, -73.8385), None));
}

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
fn should_accept_an_explicit_site_name_that_is_free() {
    let ws = workspace_con_sitio("Mi");
    assert!(exigir_nombre_libre(ws.path(), "Libre").is_ok());
}

#[test]
fn should_only_reuse_the_named_site_when_a_name_is_given() {
    let ws = workspace_con_sitio("Otro");
    let coordenadas = (7.0708, -73.8385);
    let por_nombre = Busqueda {
        coordenadas,
        altitud: None,
        nombre: Some("Mi"),
    };
    assert!(buscar_existente(ws.path(), por_nombre).is_none());
    let sin_nombre = Busqueda {
        coordenadas,
        altitud: None,
        nombre: None,
    };
    assert!(buscar_existente(ws.path(), sin_nombre).is_some());
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
