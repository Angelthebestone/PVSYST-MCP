use super::*;

const OBJETO: &[u8] =
    b"PVObject_=pvModule\r\n  PNom=300\r\n  Width=1.0\r\nEnd of PVObject pvModule\r\n";

const FIXTURE_LEGACY_PAN: &[u8] = include_bytes!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/tests/fixtures/workspace/ComposPV/PVModules/Synth_Legacy_300W.PAN"
));

fn cambio(k: &str, v: &str) -> Vec<(String, Option<String>)> {
    vec![(k.to_string(), Some(v.to_string()))]
}

#[test]
fn should_read_back_edited_field_when_object_is_found() {
    let verificado =
        verificar_span(OBJETO, "pvModule", &cambio("PNom", "300")).expect("verificado");

    assert_eq!(verificado["PNom"], "300");
}

#[test]
fn should_refuse_edit_when_edited_object_cannot_be_found_again() {
    let res = verificar_span(OBJETO, "pvInverter", &cambio("PNom", "300"));

    assert!(matches!(res, Err(Error::Format(m)) if m.contains("releer")));
}

#[test]
fn should_refuse_edit_when_field_reads_back_differently() {
    let res = verificar_span(OBJETO, "pvModule", &cambio("PNom", "310"));

    assert!(matches!(res, Err(Error::Format(m)) if m.contains("PNom")));
}

#[test]
fn should_fail_when_file_on_disk_differs_from_what_was_sent() {
    let dir = tempfile::tempdir().expect("dir");
    let ruta = dir.path().join("x.PAN");
    std::fs::write(&ruta, b"otro").expect("escribir");

    assert!(matches!(
        releer_escrito(&ruta, OBJETO),
        Err(Error::Format(_))
    ));
    std::fs::write(&ruta, OBJETO).expect("escribir");
    assert_eq!(releer_escrito(&ruta, OBJETO).expect("igual"), OBJETO);
}

#[test]
fn should_report_value_decoded_from_disk_when_legacy_slot_was_written() {
    let changes = cambio("pnom_w", "310");
    let (escrito, outcomes) =
        crate::components::pan::parchear_campos_legacy(FIXTURE_LEGACY_PAN, &changes)
            .expect("parche");

    let verificado = verificar_legacy(&escrito, &changes, &outcomes).expect("verificado");

    assert_eq!(verificado.len(), 1);
    let leido: f64 = verificado
        .values()
        .next()
        .and_then(Value::as_str)
        .and_then(|t| t.parse().ok())
        .expect("número");
    assert!((leido - 310.0).abs() < 1e-6);
}

#[test]
fn should_refuse_legacy_edit_when_disk_still_holds_old_value() {
    let changes = cambio("pnom_w", "310");
    let (_, outcomes) =
        crate::components::pan::parchear_campos_legacy(FIXTURE_LEGACY_PAN, &changes)
            .expect("parche");

    let res = verificar_legacy(FIXTURE_LEGACY_PAN, &changes, &outcomes);

    assert!(matches!(res, Err(Error::Format(_))));
}
