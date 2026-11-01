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

/// Workspace with the synthetic template project and a `Sites/Lugar.SIT` holding `sit`.
fn workspace_con_plantilla(sit: &str) -> tempfile::TempDir {
    let ws = tempfile::tempdir().expect("ws");
    let proyectos = ws.path().join("Projects");
    let sitios = ws.path().join("Sites");
    std::fs::create_dir_all(&proyectos).expect("Projects");
    std::fs::create_dir_all(&sitios).expect("Sites");
    let prj = include_bytes!("../../../fixtures/workspace/Projects/SYNTH_Plant.PRJ");
    let vc0 = include_bytes!("../../../fixtures/workspace/Projects/SYNTH_Plant.VC0");
    std::fs::write(proyectos.join("SYNTH_Plant.PRJ"), prj).expect("prj");
    std::fs::write(proyectos.join("SYNTH_Plant.VC0"), vc0).expect("vc0");
    std::fs::write(sitios.join("Lugar.SIT"), sit).expect("sit");
    ws
}

#[test]
fn should_leave_no_project_when_the_site_cannot_be_linked() {
    let ws = workspace_con_plantilla("no es un sitio");
    let paths = Paths::para_test(ws.path());
    let args = json!({ "plantilla": "SYNTH_Plant", "nombre_nuevo": "Nuevo", "sitio": "Lugar" });

    let res = crear_proyecto(&paths, &args);

    assert!(matches!(res, Err(crate::error::Error::NotFound(_))));
    assert!(!ws.path().join("Projects").join("Nuevo.PRJ").exists());
    assert!(!ws.path().join("Projects").join("Nuevo.VC0").exists());
}

#[test]
fn should_link_the_site_in_the_requested_workspace() {
    let ws = workspace_con_plantilla(
        "PVObject_=pvSite\n  Comment=Sitio creado;\nEnd of PVObject pvSite\n",
    );
    let otro = tempfile::tempdir().expect("workspace por defecto");
    let paths = Paths::para_test(otro.path());
    let args = json!({
        "workspace": ws.path(),
        "plantilla": "SYNTH_Plant",
        "nombre_nuevo": "Nuevo",
        "sitio": "Lugar",
    });

    crear_proyecto(&paths, &args).expect("creado");

    let prj = std::fs::read_to_string(ws.path().join("Projects").join("Nuevo.PRJ")).expect("prj");
    assert!(prj.contains("Comment=Sitio creado;"), "{prj}");
}
