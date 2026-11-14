use super::*;

#[test]
fn should_write_nothing_when_creating_a_site_from_coordinates_as_a_dry_run() {
    // Given: an empty workspace and the coordinates of a site to create
    let ws = tempfile::tempdir().expect("ws");
    let paths = Paths::para_test(ws.path().to_path_buf());
    let args = json!({
        "latitud": 7.07,
        "longitud": -73.84,
        "nombre_sitio": "Z_Site",
        "dry_run": true,
    });

    // When
    let out = crear_sitio_coordenadas(&paths, &args).expect("dry run");

    // Then: the command line is reported and neither the monthly CSV nor the site exist
    assert_eq!(out["dry_run"], json!(true), "out={out}");
    assert!(
        out["command_line"]
            .as_str()
            .is_some_and(|c| c.contains("Z_Site_Monthly.csv") && c.contains("create-site")),
        "out={out}"
    );
    assert!(
        !ws.path().join("UserData").exists(),
        "dry_run wrote UserData"
    );
    assert!(!ws.path().join("Sites").exists(), "dry_run wrote Sites");
}
