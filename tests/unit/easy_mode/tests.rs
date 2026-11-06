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
fn should_replace_the_template_site_when_linking_a_site() {
    let ws = tempfile::tempdir().unwrap();
    let projects = ws.path().join("Projects");
    std::fs::create_dir_all(&projects).unwrap();
    let plantilla = include_bytes!("../../fixtures/workspace/Projects/SYNTH_Plant.PRJ");
    std::fs::write(projects.join("SYNTH_Plant.PRJ"), plantilla).unwrap();
    let sit = ws.path().join("Mi.SIT");
    std::fs::write(
        &sit,
        "PVObject_=pvSite\n  Comment=Sitio creado;\nEnd of PVObject pvSite\n",
    )
    .unwrap();
    let paths = Paths::para_test(ws.path().to_path_buf());
    proyecto::vincular_sitio(&paths, "SYNTH_Plant", &sit.to_string_lossy()).unwrap();
    let prj = std::fs::read_to_string(projects.join("SYNTH_Plant.PRJ")).unwrap();
    assert!(prj.contains("Comment=Sitio creado;"), "prj={prj}");
    assert!(!prj.contains("Comment=Synthetic;Nowhere;"), "prj={prj}");
}
