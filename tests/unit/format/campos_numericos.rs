use super::super::edit::edit_fields;
use super::validar_valores;

fn cambio(campo: &str, valor: &str) -> Vec<(String, Option<String>)> {
    vec![(campo.to_string(), Some(valor.to_string()))]
}

#[test]
fn should_reject_text_in_a_numeric_field() {
    let err = validar_valores(&cambio("PNom", "abc")).expect_err("PNom es numérico");
    assert!(err.to_string().contains("PNom"), "{err}");
    assert!(err.to_string().contains("no numérico"), "{err}");
}

#[test]
fn should_accept_numbers_and_decimal_commas_in_a_numeric_field() {
    validar_valores(&cambio("PNom", "400")).expect("entero");
    validar_valores(&cambio("pnom", "400.5")).expect("decimal con punto");
    validar_valores(&cambio("PNom", "400,5")).expect("decimal con coma");
    validar_valores(&cambio("PNom", "-1.5e2")).expect("notación científica");
}

#[test]
fn should_leave_free_text_fields_alone() {
    validar_valores(&cambio("Comment", "2026")).expect("un comentario puede ser un año");
    validar_valores(&cambio("Comment", "Prueba")).expect("un comentario es texto");
}

#[test]
fn should_refuse_the_edit_and_keep_the_bytes_when_a_numeric_field_gets_text() {
    let original = b"PVObject_=pvModule\r\n  PNom=400.0\r\nEnd of PVObject pvModule\r\n";

    let err = edit_fields(original, "pvModule", &cambio("PNom", "abc"))
        .expect_err("no debe escribirse texto en PNom");
    assert!(err.to_string().contains("PNom"), "{err}");
}
