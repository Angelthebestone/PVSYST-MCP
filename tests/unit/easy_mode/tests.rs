use super::dimensionado::{dimensionar_serie, ejecutar_paso_dimensionado};
use super::parametros::DisenoParametros;
use super::*;
use serde_json::json;

#[test]
fn should_find_a_layout_inside_the_voltage_window_when_sizing_a_series() {
    // Ns 5..=17 keeps Vmpp >= 200 V and Voc*1.15*Ns <= 1000 V; 5x5x400 W = 10 kW.
    assert_eq!(
        dimensionar_serie(10.0, 400.0, 49.0, 41.0, 200.0, 1000.0),
        Some((5, 5))
    );

    // A VAbsMax of 1e12 V would allow ~1e10 modules per string: the search must stay bounded.
    let (ns, _) = dimensionar_serie(10.0, 400.0, 49.0, 41.0, 200.0, 1.0e12).expect("a layout");
    assert!(ns <= 1000, "{ns}");
}

#[test]
fn should_name_the_missing_module_when_sizing_cannot_read_it() {
    let paths = Paths::para_test(std::env::temp_dir());
    let params = DisenoParametros::parse(&json!({
        "latitud": 10.0,
        "longitud": -70.0,
        "kwp": 10.0,
        "modulo": "ModuloInexistente_12345.PAN"
    }))
    .expect("parámetros");
    let err = ejecutar_paso_dimensionado(&paths, &params).unwrap_err();
    let msg = err.to_string();
    assert!(
        msg.contains("ModuloInexistente_12345.PAN"),
        "expected error to explicitly mention the missing module name, got: {msg}"
    );
}

#[test]
fn should_reject_mcp_sizing_when_aproximaciones_is_disabled() {
    let mut paths = Paths::para_test(std::env::temp_dir());
    paths.aproximaciones = false;
    let args = json!({
        "latitud": 7.07,
        "longitud": -73.84,
        "kwp": 5.0,
        "simular": false,
        "via": "aproximacion",
    });
    let err = super::disenar_sistema(&paths, &args).unwrap_err();
    let msg = err.to_string();
    assert!(msg.contains("PVSYST_MCP_APROXIMACIONES"), "msg={msg}");
    assert!(
        msg.contains("n_mod_serie") && msg.contains("n_strings"),
        "msg={msg}"
    );
}

#[test]
fn should_extract_kpis_when_the_report_pdf_is_a_real_report() {
    let paths = Paths::para_test(std::env::temp_dir());
    let pdf_path = concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/tests/fixtures/results/MCP_E2E.pdf"
    );
    let kpis = super::simulacion::leer_kpis(&paths, &json!({ "report_pdf": pdf_path })).unwrap();
    assert_eq!(kpis["energia_anual_kwh"], 14299.0);
    assert_eq!(kpis["produccion_especifica_kwh_kwp"], 1589.0);
    assert_eq!(kpis["pr_pct"], 83.68);
}

#[test]
fn should_replace_the_template_site_in_project_and_variant_when_linking_a_site() {
    let ws = tempfile::tempdir().unwrap();
    let projects = ws.path().join("Projects");
    std::fs::create_dir_all(&projects).unwrap();
    let plantilla = include_bytes!("../../fixtures/workspace/Projects/SYNTH_Plant.PRJ");
    std::fs::write(projects.join("SYNTH_Plant.PRJ"), plantilla).unwrap();
    let vc0 = include_bytes!("../../fixtures/workspace/Projects/SYNTH_Plant.VC0");
    std::fs::write(projects.join("SYNTH_Plant.VC0"), vc0).unwrap();
    let sit = ws.path().join("Mi.SIT");
    std::fs::write(
        &sit,
        "PVObject_=pvSite\n  Comment=Sitio creado;\nEnd of PVObject pvSite\n",
    )
    .unwrap();
    let paths = Paths::para_test(ws.path().to_path_buf());
    proyecto::vincular_sitio(
        &paths,
        &proyecto::SitioDelDiseno {
            nombre: "SYNTH_Plant",
            variante: "VC0",
            sit_file: &sit.to_string_lossy(),
        },
    )
    .unwrap();
    for archivo in ["SYNTH_Plant.PRJ", "SYNTH_Plant.VC0"] {
        let texto = std::fs::read_to_string(projects.join(archivo)).unwrap();
        assert!(texto.contains("Comment=Sitio creado;"), "{archivo}={texto}");
        assert!(
            !texto.contains("Comment=Synthetic;Nowhere;"),
            "{archivo} keeps the template site: {texto}"
        );
    }
}
#[test]
fn should_resume_the_design_when_the_project_exists_with_the_same_parameters() {
    // Given: a design run that already created the project
    let (_ws, paths) = workspace_con_plantilla();
    let params = diseno(&json!({}));
    proyecto::ejecutar_paso_proyecto(&paths, &params).expect("primer proyecto");

    // When: the same design is retried after a failed step
    let segundo = proyecto::ejecutar_paso_proyecto(&paths, &params).expect("reintento");

    // Then: it continues instead of demanding a rebuild
    assert_eq!(segundo.raw_response["reutilizado"], json!(true));
    assert_eq!(segundo.nombre, "Diseno");
}

#[test]
fn should_reject_the_design_when_the_existing_project_holds_other_parameters() {
    // Given: a project created with 10 modules in series and 2 strings
    let (_ws, paths) = workspace_con_plantilla();
    proyecto::ejecutar_paso_proyecto(&paths, &diseno(&json!({}))).expect("primer proyecto");

    // When: the retry asks for a different layout
    let err = proyecto::ejecutar_paso_proyecto(&paths, &diseno(&json!({ "n_strings": 4 })))
        .expect_err("parámetros distintos");

    // Then: the difference and the two ways out are named
    let msg = err.to_string();
    assert!(msg.contains("NStringCh"), "msg={msg}");
    assert!(msg.contains("sobrescribir"), "msg={msg}");
}

#[test]
fn should_rebuild_the_project_when_sobrescribir_is_asked_for() {
    let (_ws, paths) = workspace_con_plantilla();
    proyecto::ejecutar_paso_proyecto(&paths, &diseno(&json!({}))).expect("primer proyecto");
    let otro = proyecto::ejecutar_paso_proyecto(
        &paths,
        &diseno(&json!({ "n_strings": 4, "sobrescribir": true })),
    )
    .expect("sobrescritura");
    assert!(otro.raw_response.get("reutilizado").is_none(), "{otro:?}");
}

/// A workspace with the synthetic project as a template to copy from.
fn workspace_con_plantilla() -> (tempfile::TempDir, Paths) {
    let ws = tempfile::tempdir().unwrap();
    let projects = ws.path().join("Projects");
    std::fs::create_dir_all(&projects).unwrap();
    let prj = include_bytes!("../../fixtures/workspace/Projects/SYNTH_Plant.PRJ");
    let vc0 = include_bytes!("../../fixtures/workspace/Projects/SYNTH_Plant.VC0");
    std::fs::write(projects.join("SYNTH_Plant.PRJ"), prj).unwrap();
    std::fs::write(projects.join("SYNTH_Plant.VC0"), vc0).unwrap();
    let paths = Paths {
        workspace: ws.path().to_path_buf(),
        ..Paths::resolve()
    };
    (ws, paths)
}

fn diseno(extra: &serde_json::Value) -> DisenoParametros {
    let mut args = json!({
        "latitud": 10.0,
        "longitud": -70.0,
        "kwp": 8.0,
        "plantilla": "SYNTH_Plant",
        "nombre": "Diseno",
        "modulo": "Synth_400W.PAN",
        "inversor": "Synth_10kW.OND",
        "n_mod_serie": 10,
        "n_strings": 2,
        "simular": false,
    });
    for (k, v) in extra.as_object().expect("objeto") {
        args[k.as_str()] = v.clone();
    }
    DisenoParametros::parse(&args).expect("parámetros")
}

#[test]
fn should_skip_modules_without_electrical_data_when_substituting_the_template_module() {
    // Given: a template whose module is missing, and two candidates, the first one
    // alphabetically without Vmpp
    let (ws, paths) = workspace_con_plantilla();
    let modulos = ws.path().join("ComposPV").join("PVModules");
    std::fs::create_dir_all(&modulos).unwrap();
    std::fs::write(
        modulos.join("A_Sin_Vmpp.PAN"),
        "PVObject_=pvModule\n  PNom=300\n  Voc=44.5\n  Isc=9.0\nEnd of PVObject pvModule\n",
    )
    .unwrap();
    std::fs::copy(
        concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/tests/fixtures/workspace/ComposPV/PVModules/Synth_400W.PAN"
        ),
        modulos.join("B_Con_Datos.PAN"),
    )
    .unwrap();
    let params = diseno(&json!({ "modulo": null }));
    proyecto::ejecutar_paso_proyecto(&paths, &params).expect("proyecto");

    // When
    let paso = ejecutar_paso_dimensionado(&paths, &params).expect("dimensionado");

    // Then: the module that can be sized is the substitute, and it is the one reported
    let aviso = paso
        .avisos
        .iter()
        .find(|a| a.contains("no existe"))
        .unwrap_or_else(|| panic!("sin aviso de sustitución: {:?}", paso.avisos));
    assert!(aviso.contains("B_Con_Datos.PAN"), "aviso={aviso}");
    assert!(!aviso.contains("A_Sin_Vmpp.PAN"), "aviso={aviso}");
    assert_eq!(paso.kwp_realizado, Some(8.0));
}
