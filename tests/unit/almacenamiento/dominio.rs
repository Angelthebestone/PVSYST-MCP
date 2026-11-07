use super::*;

fn parametros(valor: &Value) -> Result<ParametrosAlmacenamiento> {
    ParametrosAlmacenamiento::nuevo(EntradaAlmacenamiento::de_args(valor)?)
}

#[test]
fn should_say_the_bank_size_is_a_default_when_the_caller_gave_none() {
    let p = parametros(&json!({"estrategia": "autoconsumo"})).expect("parámetros");
    let criterio = p.criterio_dimensionado();
    assert_eq!(criterio["origen"], json!("valores_por_defecto"));
    assert_eq!(criterio["n_serie"], json!(10));
    assert_eq!(criterio["n_paralelo"], json!(20));
    assert!(
        criterio["nota"]
            .as_str()
            .is_some_and(|n| n.contains("defaults")),
        "explica el criterio: {criterio}"
    );
}

#[test]
fn should_say_the_bank_size_came_from_the_caller_when_both_counts_are_given() {
    let p = parametros(&json!({
        "estrategia": "autoconsumo", "n_serie": 4, "n_paralelo": 6
    }))
    .expect("parámetros");
    let criterio = p.criterio_dimensionado();
    assert_eq!(criterio["origen"], json!("parametros"));
    assert_eq!(criterio["n_serie"], json!(4));
    assert_eq!(criterio["n_paralelo"], json!(6));
    assert!(criterio.get("nota").is_none(), "sin aviso: {criterio}");
}
