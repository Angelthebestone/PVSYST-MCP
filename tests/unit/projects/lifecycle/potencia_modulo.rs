use super::*;

#[test]
fn should_compute_pnom_from_sizing_dividing_kwp_by_modules() {
    let text = "PVObject_=pvSubArray
  PNomSizing=9.0
  NModSerie=15
  NStringCh=2
End of PVObject pvSubArray
";
    let root = object::parse_first(text).expect("parse");
    assert_eq!(pnom_from_sizing(&root), Some(300.0));
}

#[test]
fn should_extract_pnom_from_name_with_watt_suffix() {
    assert_eq!(pnom_from_name("Generic_Mono_300W.PAN"), Some(300.0));
    assert_eq!(pnom_from_name("CX-M500_2450Wp_1_00_086.PAN"), None);
    assert_eq!(pnom_from_name("Module.PAN"), None);
}

#[test]
fn should_report_unreadable_module_instead_of_inferring_its_power() {
    let ws = tempfile::tempdir().expect("ws");
    let dir = ws.path().join("ComposPV").join("PVModules");
    std::fs::create_dir_all(&dir).expect("dir");
    std::fs::write(dir.join("Roto_300W.PAN"), b"no es un PAN").expect("pan");
    let paths = crate::config::Paths::para_test(ws.path());
    let root = object::parse_first("PVObject_=pvSubArray\n  PNomSizing=9.0\n  NModSerie=15\n  NStringCh=2\nEnd of PVObject pvSubArray\n").expect("root");
    let args = json!({ "workspace": ws.path() });
    let consulta = ConsultaModulo {
        paths: &paths,
        args: &args,
        root: &root,
    };

    let res = pnom_del_modulo(&consulta, "Roto_300W.PAN");

    assert!(
        matches!(res, Err(crate::error::Error::Invalid(_))),
        "{res:?}"
    );
}

#[test]
fn should_infer_power_from_sizing_when_module_is_missing() {
    let ws = tempfile::tempdir().expect("ws");
    let paths = crate::config::Paths::para_test(ws.path());
    let root = object::parse_first("PVObject_=pvSubArray\n  PNomSizing=9.0\n  NModSerie=15\n  NStringCh=2\nEnd of PVObject pvSubArray\n").expect("root");
    let args = json!({ "workspace": ws.path() });
    let consulta = ConsultaModulo {
        paths: &paths,
        args: &args,
        root: &root,
    };

    assert_eq!(
        pnom_del_modulo(&consulta, "NoExiste.PAN").expect("inferido"),
        300.0
    );
}
