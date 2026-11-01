use super::pasos::PasoSitioSalida;
use super::*;

fn paso(ok: bool, sit_file: &str) -> PasoSitioSalida {
    PasoSitioSalida {
        sit_file: sit_file.into(),
        site_name: Some("Planta_Site".into()),
        weather_source: None,
        annual_ghi_kwh_m2: None,
        reutilizado: false,
        ok,
    }
}

#[test]
fn should_stop_design_when_site_step_did_not_create_the_site() {
    for p in [paso(false, "Sites/Planta_Site.SIT"), paso(true, "")] {
        let res = exigir_sitio_creado(&p);
        assert!(matches!(res, Err(Error::Cli(m)) if m.contains("Planta_Site")));
    }
}

#[test]
fn should_continue_design_when_site_exists() {
    assert!(exigir_sitio_creado(&paso(true, "Sites/Planta_Site.SIT")).is_ok());
}

#[test]
fn should_undo_only_the_project_files_when_a_design_run_fails() {
    let ws = tempfile::tempdir().expect("ws");
    let projects = ws.path().join("Projects");
    std::fs::create_dir_all(&projects).expect("Projects");
    std::fs::write(projects.join("Previo.PRJ"), b"antes").expect("previo");
    std::fs::write(projects.join("Mio.PRJ"), b"original").expect("mio");
    let paths = Paths::para_test(ws.path());

    let foto = proyecto::FotoProyecto::tomar(&paths, "Mio").expect("foto");
    std::fs::write(projects.join("Mio.PRJ"), b"a medias").expect("pisado");
    std::fs::write(projects.join("Mio.VC0"), b"x").expect("nuevo");
    std::fs::write(projects.join("Ajeno.PRJ"), b"creado por PVsyst").expect("ajeno");
    foto.deshacer(&paths);

    assert_eq!(
        std::fs::read(projects.join("Previo.PRJ")).expect("previo"),
        b"antes"
    );
    assert_eq!(
        std::fs::read(projects.join("Mio.PRJ")).expect("mio"),
        b"original"
    );
    assert!(!projects.join("Mio.VC0").exists());
    assert!(projects.join("Ajeno.PRJ").exists());
}

#[test]
fn should_take_an_empty_photo_when_projects_folder_does_not_exist() {
    let ws = tempfile::tempdir().expect("ws");
    let paths = Paths::para_test(ws.path());

    let foto = proyecto::FotoProyecto::tomar(&paths, "Mio").expect("foto");
    std::fs::create_dir_all(ws.path().join("Projects")).expect("Projects");
    std::fs::write(ws.path().join("Projects").join("Mio.PRJ"), b"x").expect("nuevo");
    foto.deshacer(&paths);

    assert!(!ws.path().join("Projects").join("Mio.PRJ").exists());
}
