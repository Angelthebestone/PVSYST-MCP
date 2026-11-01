use super::preparacion::{con_salidas_por_defecto, push_options};
use super::*;

#[test]
fn typed_options_are_emitted() {
    let mut v = Vec::new();
    let args = json!({
        "input_rvt_file": "in.RVT",
        "cmd_options_file": "opts.cof",
        "recalcular_sombreado": false,
        "paginas_reporte": ["cover", "summary"],
        "idioma_reporte": "es",
    });
    push_options(&args, &mut v);
    assert!(v.contains(&"-irf:in.RVT".to_string()));
    assert!(v.contains(&"-cof:opts.cof".to_string()));
    assert!(v.contains(&"-rst:0".to_string()));
    assert!(v.contains(&"-rp:cover,summary".to_string()));
    assert!(v.contains(&"-rl:es".to_string()));
}

#[test]
fn timeout_default_for_simulation_is_900() {
    assert_eq!(
        cli::timeout_de(&json!({}), cli::TIMEOUT_SIMULACION_S),
        std::time::Duration::from_secs(900)
    );
}

#[test]
fn should_default_outputs_into_results_and_respect_overrides_when_inside_workspace() {
    let dir = tempfile::tempdir().expect("ws");
    let ws = dir.path();
    let v = con_salidas_por_defecto(&json!({}), ws, "Planta", "VC0").expect("por defecto");
    let csv = Path::new(v["output_csv"].as_str().expect("csv"));
    assert!(csv.ends_with("Results/Planta_VC0.csv"));
    assert!(v["report_pdf"]
        .as_str()
        .expect("pdf")
        .ends_with("Planta_VC0.pdf"));

    let explicito = con_salidas_por_defecto(
        &json!({"csv": "x.csv", "sin_pdf": true}),
        ws,
        "Planta",
        "VC0",
    )
    .expect("explícito");
    let csv_explicito = Path::new(explicito["output_csv"].as_str().expect("csv"));
    assert_eq!(csv_explicito, ws.join("x.csv"));
    assert!(explicito.get("report_pdf").is_none());
}

#[test]
fn should_reject_output_outside_workspace_when_not_permitted() {
    let ws = tempfile::tempdir().expect("ws");
    let fuera = tempfile::tempdir().expect("fuera");
    let csv = fuera.path().join("robado.csv");
    for clave in ["output_csv", "report_pdf"] {
        let res = con_salidas_por_defecto(
            &json!({ clave: csv.to_string_lossy() }),
            ws.path(),
            "Planta",
            "VC0",
        );
        assert!(matches!(res, Err(Error::Invalid(_))), "{clave}");
    }
    let res = con_salidas_por_defecto(&json!({ "csv": "../x.csv" }), ws.path(), "P", "VC0");
    assert!(matches!(res, Err(Error::Invalid(_))));
}

#[test]
fn should_keep_output_outside_workspace_when_explicitly_permitted() {
    let ws = tempfile::tempdir().expect("ws");
    let fuera = tempfile::tempdir().expect("fuera");
    let csv = fuera.path().join("fuera.csv");
    let args = json!({ "output_csv": csv.to_string_lossy(), "permitir_fuera_del_workspace": true });

    let v = con_salidas_por_defecto(&args, ws.path(), "P", "VC0").expect("permitido");

    assert_eq!(Path::new(v["output_csv"].as_str().expect("csv")), csv);
}

#[test]
fn should_reject_extra_option_that_redefines_a_managed_flag() {
    for opcion in [
        "-w:C:\\otro",
        "-p:Otro.PRJ",
        "-V:VC9",
        "-ocf:x.csv",
        "-bpf:p.dat",
        "-s:x.SIT",
    ] {
        let res = super::preparacion::validar_extra_options(&json!({ "extra_options": [opcion] }));
        assert!(
            matches!(res, Err(Error::Invalid(m)) if m.contains("redefinir")),
            "{opcion}"
        );
    }
}

#[test]
fn should_reject_non_text_extra_option() {
    let res = super::preparacion::validar_extra_options(&json!({ "extra_options": [3] }));
    assert!(matches!(res, Err(Error::Invalid(_))));
}

#[test]
fn should_accept_unmanaged_extra_option_or_none() {
    let p = super::preparacion::validar_extra_options;
    assert!(p(&json!({ "extra_options": ["-hd:1"] })).is_ok());
    assert!(p(&json!({})).is_ok());
}

#[test]
fn modo_gui_enum_matches_de_bool() {
    assert!(ModoGui::de_bool(true).debe_abrir());
    assert!(!ModoGui::de_bool(false).debe_abrir());
}

#[test]
fn should_reject_simulation_variant_that_is_not_an_identifier() {
    let ws = tempfile::tempdir().expect("ws");
    let dir = ws.path().join("Projects");
    std::fs::create_dir_all(&dir).expect("Projects");
    std::fs::write(dir.join("Planta.PRJ"), "x").expect("prj");
    let paths = crate::config::Paths::para_test(ws.path());
    let args = json!({ "proyecto": "Planta", "variante": "VC0\\..\\..\\x", "dry_run": true });

    let res = lanzar_simulacion(&paths, &args);

    assert!(matches!(res, Err(Error::Invalid(m)) if m.contains("Variante inválida")));
}
