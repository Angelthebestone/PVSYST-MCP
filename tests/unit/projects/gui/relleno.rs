use super::tomado;
use serde_json::json;

#[test]
fn should_accept_a_value_that_reads_back_exactly() {
    assert!(tomado(
        &json!({ "estado": "ok", "pedido": "30", "despues": "30.0" })
    ));
}

#[test]
fn should_accept_a_number_that_pvsyst_shows_with_fewer_decimals() {
    assert!(tomado(
        &json!({ "estado": "distinto", "pedido": "12.345", "despues": "12.3" })
    ));
}

#[test]
fn should_reject_a_number_that_reads_back_far_from_the_request() {
    assert!(!tomado(
        &json!({ "estado": "distinto", "pedido": "12.345", "despues": "9.0" })
    ));
}

#[test]
fn should_reject_a_text_value_that_differs_from_the_request() {
    assert!(!tomado(
        &json!({ "estado": "distinto", "pedido": "VC1", "despues": "VC0" })
    ));
}
