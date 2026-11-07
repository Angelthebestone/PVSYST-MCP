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
    assert_eq!(outcomes[0].action, AccionCambio::Replaced);
    assert_eq!(outcomes[0].old_value.as_deref(), Some("0.0100"));
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
fn should_reject_values_with_line_breaks_when_editing_a_section() {
    let error = editar_campos_seccion(
        ECONOMIA.as_bytes(),
        "EconomicEval",
        &[("Currency".into(), Some("US\r\nD".into()))],
    )
    .expect_err("salto de línea");
    assert!(matches!(error, Error::Invalid(_)));
}
