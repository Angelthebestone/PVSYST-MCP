use super::*;
use crate::format::object;

fn modulo() -> PvNode {
    object::parse_first("PVObject_=pvModule\n  PNom=400.0\n  Isc=10.20\n  Technol=mtSiMono\nEnd of PVObject pvModule\n")
        .expect("módulo")
}

#[test]
fn should_find_no_difference_when_saved_values_match_within_rounding() {
    let enviados = json!({ "PNom": 400, "Isc": "10.2", "Technol": "mtSiMono", "NoLeida": 1 });

    let d = diferencias(
        &modulo(),
        &["PNom", "Isc", "Technol"],
        enviados.as_object().expect("mapa"),
    );

    assert!(d.is_empty(), "{d:?}");
}

#[test]
fn should_report_value_that_was_saved_differently_or_not_saved() {
    let enviados = json!({ "PNom": 410, "Voc": 49.5 });

    let d = diferencias(
        &modulo(),
        &["PNom", "Voc"],
        enviados.as_object().expect("mapa"),
    );

    assert_eq!(d.len(), 2);
    assert_eq!(d[0]["clave"], "PNom");
    assert_eq!(d[0]["guardado"], "400.0");
    assert_eq!(d[1]["guardado"], Value::Null);
}
