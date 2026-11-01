use super::*;

#[test]
fn should_count_fields_that_read_differently_when_summarising() {
    let mut salida = json!({ "resultados": [
        { "clave": "a", "estado": "ok" },
        { "clave": "b", "estado": "distinto" },
        { "clave": "c", "estado": "distinto" },
    ]});

    resumir_relleno(&mut salida);

    assert_eq!(salida["campos_que_difieren"], 2);
    assert!(salida.get("completo").is_none());
}

#[test]
fn should_mark_fill_incomplete_and_keep_typed_fields_when_one_fails() {
    let mut salida = json!({ "resultados": [
        { "clave": "a", "estado": "ok" },
        paso_fallido("b", &Error::Gui("PVsyst no respondió".into())),
    ]});

    resumir_relleno(&mut salida);

    assert_eq!(salida["completo"], false);
    assert_eq!(salida["errores"], json!(["PVsyst no respondió"]));
    assert_eq!(salida["resultados"][0]["estado"], "ok");
}

#[test]
fn should_report_failed_tab_as_failure() {
    let fallo = pagina_fallida(2, &Error::Gui("pestaña".into()));

    assert!(es_fallido(&fallo));
    assert_eq!(fallo["pagina"], 2);
    assert!(!es_fallido(&json!({ "estado": "ok" })));
}

#[test]
fn should_report_zero_differences_when_there_are_no_results() {
    let mut salida = json!({});

    resumir_relleno(&mut salida);

    assert_eq!(salida["campos_que_difieren"], 0);
}
