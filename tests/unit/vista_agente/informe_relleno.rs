use super::*;

#[test]
fn should_keep_the_typed_fields_when_one_fill_in_step_fails() {
    let mut salida = json!({ "resultados": [
        { "clave": "a", "estado": "ok" },
        paso_fallido("b", &Error::Gui("PVsyst no respondió".into())),
    ]});

    resumir_relleno(&mut salida);

    assert_eq!(salida["completo"], false);
    assert_eq!(salida["errores"], json!(["PVsyst no respondió"]));
    assert_eq!(salida["resultados"][0]["estado"], "ok");
}
