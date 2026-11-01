use super::*;

#[test]
fn should_reject_site_name_that_leaves_sites_folder() {
    let ws = tempfile::tempdir().expect("ws");
    let fuera = tempfile::tempdir().expect("fuera");
    std::fs::write(fuera.path().join("Ajeno.SIT"), "x").expect("sit");
    let ajeno = fuera.path().join("Ajeno").to_string_lossy().into_owned();

    for sitio in ["..\\..\\Ajeno", "../Ajeno", ajeno.as_str()] {
        let res = sit_pedido(ws.path(), &json!({ "sitio": sitio }));
        assert!(matches!(res, Err(Error::Invalid(_))), "{sitio}: {res:?}");
    }
}

#[test]
fn should_return_site_path_when_it_exists_in_sites() {
    let ws = tempfile::tempdir().expect("ws");
    let dir = ws.path().join("Sites");
    std::fs::create_dir_all(&dir).expect("Sites");
    std::fs::write(dir.join("Lugar.SIT"), "x").expect("sit");

    let sit = sit_pedido(ws.path(), &json!({ "sitio": "Lugar" })).expect("sitio");

    assert_eq!(sit, Some(dir.join("Lugar.SIT")));
}

#[test]
fn should_report_not_found_when_site_is_missing() {
    let ws = tempfile::tempdir().expect("ws");

    let res = sit_pedido(ws.path(), &json!({ "sitio": "Nada" }));

    assert!(matches!(res, Err(Error::NotFound(_))), "{res:?}");
}

#[test]
fn should_not_look_up_demo_template_when_name_is_rooted() {
    assert_eq!(plantilla_demo_ro("\\Windows\\win"), None);
    assert_eq!(plantilla_demo_ro("C:x"), None);
}
