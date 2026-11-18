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

/// A `.SIT` whose site object differs from the template's, so the test can tell them apart.
const SIT_VALIDO: &str = "PVObject_=pvSite\r\n  Comment=Requested;Synthetic;\r\n  NomF=SitioPedido\r\n  Country=Nowhere\r\n  Latitude=10.0000\r\n  Longitude=-70.0000\r\n  Altitude=100\r\n  TimeZone=-4.0\r\n  Albedo=0.20\r\nEnd of PVObject pvSite\r\n";

/// One new project per accepted `sitio` form; returns each linked `pvSite` block.
fn crear_proyectos_de_cada_forma(ws: &Path, paths: &Paths, esperado: &str) -> Vec<Vec<u8>> {
    let mut sitios = Vec::new();
    for (i, forma) in ["Lugar", "Lugar.SIT", "Sites/Lugar.SIT", "Sites\\Lugar.SIT"]
        .into_iter()
        .enumerate()
    {
        let nombre = format!("Nuevo{i}");
        let args = json!({
            "plantilla": "SYNTH_Plant",
            "nombre_nuevo": nombre,
            "sitio": forma,
        });
        let res = crear_proyecto(paths, &args).expect(forma);
        assert_eq!(res["sitio"].as_str(), Some(esperado), "{forma}: {res}");
        let prj =
            std::fs::read(ws.join("Projects").join(format!("{nombre}.PRJ"))).expect("new project");
        let bloque = crate::format::insert::extraer_objeto(&prj, "pvSite").expect("pvSite");
        assert!(
            String::from_utf8_lossy(&bloque).contains("NomF=SitioPedido"),
            "{forma}: the project kept another site"
        );
        sitios.push(bloque);
    }
    sitios
}

#[test]
fn should_take_the_same_site_from_every_accepted_site_argument_form() {
    let ws = workspace_con_plantilla(SIT_VALIDO);
    let paths = Paths::para_test(ws.path());
    let esperado = ws.path().join("Sites").join("Lugar.SIT");
    let esperado = esperado.to_string_lossy().into_owned();
    let sitios = crear_proyectos_de_cada_forma(ws.path(), &paths, &esperado);
    assert!(
        sitios.windows(2).all(|par| par.first() == par.get(1)),
        "the argument forms produced different sites"
    );

    let fuera = tempfile::tempdir().expect("fuera");
    let ajeno = fuera.path().join("Ajeno.SIT");
    std::fs::write(&ajeno, SIT_VALIDO).expect("sit");
    let absoluto = ajeno.to_string_lossy().into_owned();
    for (i, forma) in ["../Ajeno.SIT", "Sites/../../Ajeno.SIT", absoluto.as_str()]
        .into_iter()
        .enumerate()
    {
        let args = json!({
            "plantilla": "SYNTH_Plant",
            "nombre_nuevo": format!("Invalido{i}"),
            "sitio": forma,
        });
        let err = crear_proyecto(&paths, &args).expect_err(forma).to_string();
        assert!(err.contains(forma), "{forma}: {err}");
        assert!(err.contains("Sites/X.SIT"), "{forma}: {err}");
    }
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
