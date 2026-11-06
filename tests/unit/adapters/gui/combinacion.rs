use super::*;

#[test]
fn should_refuse_delete_and_insert_even_with_modifiers() {
    for tecla in [
        "DELETE",
        "del",
        "Supr",
        "INSERT",
        "ins",
        "SHIFT+DELETE",
        "CTRL+INSERT",
    ] {
        let err = parsear_combinacion(tecla).unwrap_err().to_string();
        assert!(err.contains("no se envía"), "{tecla}: {err}");
    }
}
