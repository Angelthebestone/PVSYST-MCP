use super::*;
use crate::error::Error;

fn modulo(pnom: f64, vendor: &str) -> Result<Value> {
    Ok(json!({"pnom": pnom, "vendor": vendor, "model": "X1"}))
}

#[test]
fn numero_accepts_decimal_comma_strings() {
    assert_eq!(numero(&json!("3,5")), Some(3.5));
    assert_eq!(numero(&json!(410)), Some(410.0));
    assert_eq!(numero(&json!("n/a")), None);
}

#[test]
fn rango_de_potencia_excluye_modulos_fuera() {
    let c = Criterios::desde(&json!({"pnom_min": 350}));
    assert!(c.admite("a.pan", &modulo(300.0, "Acme")).is_none());
    assert!(c.admite("b.pan", &modulo(400.0, "Acme")).is_some());
}

#[test]
fn texto_busca_en_fabricante_sin_distinguir_mayusculas() {
    let c = Criterios::desde(&json!({"texto": "ACME"}));
    let hit = c
        .admite("a.pan", &modulo(300.0, "Acme Solar"))
        .unwrap_or(Value::Null);
    assert_eq!(hit["vendor"], json!("Acme Solar"));
    assert!(c.admite("a.pan", &modulo(300.0, "Otro")).is_none());
}

#[test]
fn modulo_ilegible_solo_pasa_sin_filtro_de_potencia() {
    let error: Result<Value> = Err(Error::Format("corrupto".into()));
    assert!(Criterios::desde(&json!({}))
        .admite("x.pan", &error)
        .is_some());
    assert!(Criterios::desde(&json!({"pnom_max": 500}))
        .admite("x.pan", &error)
        .is_none());
}
