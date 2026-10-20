use super::*;

const ECONOMIA: &str = "PVObject_=pvVCalcul\r\n  EconomicEval=TEcoEvaluation\r\n    \
ProjectLife=20\r\n    InflationRate=0.0150\r\n    DiscountRate=0.0100\r\n    \
DiscountRate=0.0100\r\n  End of TEcoEvaluation\r\nEnd of PVObject pvVCalcul\r\n";

#[test]
fn should_replace_every_duplicate_line_when_field_repeats() {
    let (out, outcomes) = editar_campos_seccion(
        ECONOMIA.as_bytes(),
        "EconomicEval",
        &[("DiscountRate".into(), Some("0.0300".into()))],
    )
    .expect("editar sección");

    let texto = String::from_utf8(out).expect("utf8");
    assert_eq!(texto.matches("DiscountRate=0.0300").count(), 2);
    assert!(!texto.contains("DiscountRate=0.0100"));
    assert_eq!(outcomes.len(), 1);
    assert_eq!(outcomes[0].action, "replaced");
    assert_eq!(outcomes[0].old_value.as_deref(), Some("0.0100"));
}

#[test]
fn should_insert_missing_field_before_the_closing_marker() {
    let (out, outcomes) = editar_campos_seccion(
        ECONOMIA.as_bytes(),
        "EconomicEval",
        &[("SummerPeakSaleTariff".into(), Some("0.02000".into()))],
    )
    .expect("insertar campo");

    let texto = String::from_utf8(out).expect("utf8");
    assert!(texto.contains("    SummerPeakSaleTariff=0.02000\r\n  End of TEcoEvaluation"));
    assert_eq!(outcomes[0].action, "inserted");
}

#[test]
fn should_preserve_crlf_when_editing_section() {
    let (out, _) = editar_campos_seccion(
        ECONOMIA.as_bytes(),
        "EconomicEval",
        &[
            ("ProjectLife".into(), Some("25".into())),
            ("Currency".into(), Some("COP".into())),
        ],
    )
    .expect("editar con CRLF");

    let texto = String::from_utf8(out).expect("utf8");
    assert!(texto.contains("    ProjectLife=25\r\n"));
    assert!(texto.contains("    Currency=COP\r\n"));
    assert!(!texto.contains("ProjectLife=25\n  End"));
}

#[test]
fn should_report_not_found_when_section_is_absent() {
    let error = editar_campos_seccion(
        b"PVObject_=pvVCalcul\nEnd of PVObject pvVCalcul\n",
        "EconomicEval",
        &[("Currency".into(), Some("USD".into()))],
    )
    .expect_err("sección ausente");

    assert!(matches!(error, Error::NotFound(_)));
    assert!(error.to_string().contains("EconomicEval"));
}

#[test]
fn should_accept_the_short_closing_marker_used_by_water_use() {
    let agua = "PVObject_=pvLoad\n  WaterUse=TWaterUse\n    FlowModul=Yearly\n    \
FlowYear=40\n  End of WaterUse\nEnd of PVObject pvLoad\n";

    assert!(existe_seccion(agua.as_bytes(), "WaterUse"));
    assert_eq!(
        campo_seccion(agua.as_bytes(), "WaterUse", "FlowYear").as_deref(),
        Some("40")
    );

    let (out, _) = editar_campos_seccion(
        agua.as_bytes(),
        "WaterUse",
        &[("FlowYear".into(), Some("25.000".into()))],
    )
    .expect("editar WaterUse");
    let texto = String::from_utf8(out).expect("utf8");
    assert!(texto.contains("    FlowYear=25.000\n"));
}

#[test]
fn should_ignore_fields_inside_nested_objects_when_editing_a_section() {
    let con_hijo = "PVObject_=pvVCalcul\n  EconomicEval=TEcoEvaluation\n    \
DiscountRate=0.0100\n    PVObject_=pvNested\n      DiscountRate=0.9900\n    \
End of PVObject pvNested\n  End of TEcoEvaluation\nEnd of PVObject pvVCalcul\n";

    let (out, _) = editar_campos_seccion(
        con_hijo.as_bytes(),
        "EconomicEval",
        &[("DiscountRate".into(), Some("0.0300".into()))],
    )
    .expect("editar sección anidada");
    let texto = String::from_utf8(out).expect("utf8");
    assert!(texto.contains("    DiscountRate=0.0300"));
    assert!(texto.contains("      DiscountRate=0.9900"));
}

#[test]
fn should_delete_every_duplicate_line_when_value_is_none() {
    let (out, outcomes) = editar_campos_seccion(
        ECONOMIA.as_bytes(),
        "EconomicEval",
        &[("DiscountRate".into(), None)],
    )
    .expect("borrar duplicados");

    let texto = String::from_utf8(out).expect("utf8");
    assert!(!texto.contains("DiscountRate"));
    assert_eq!(outcomes[0].action, "deleted");
}

#[test]
fn should_report_not_found_when_deleting_an_absent_field() {
    let (_, outcomes) = editar_campos_seccion(
        ECONOMIA.as_bytes(),
        "EconomicEval",
        &[("Currency".into(), None)],
    )
    .expect("borrar ausente");

    assert_eq!(outcomes[0].action, "not-found");
}

#[test]
fn should_reject_values_with_line_breaks() {
    let error = editar_campos_seccion(
        ECONOMIA.as_bytes(),
        "EconomicEval",
        &[("Currency".into(), Some("US\r\nD".into()))],
    )
    .expect_err("salto de línea");
    assert!(matches!(error, Error::Invalid(_)));
}
