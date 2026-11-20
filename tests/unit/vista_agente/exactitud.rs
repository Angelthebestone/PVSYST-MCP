use super::diferencias;
use serde_json::json;

#[test]
fn should_list_every_value_pvsyst_did_not_take_when_some_differ() {
    let respuesta = json!({ "resultados": [
        { "clave": "a", "pedido": "100", "despues": "100.9", "estado": "distinto" },
        { "clave": "b", "pedido": "5", "despues": "5", "estado": "ok" },
        { "clave": "c", "pedido": "7", "despues": null, "estado": "simulado" },
    ]});

    let lista = diferencias(&respuesta);

    assert_eq!(lista.len(), 2);
    assert!(
        lista[0].contains("'a'") && lista[0].contains("100.9"),
        "{lista:?}"
    );
    assert!(lista[1].contains("'c'"), "{lista:?}");
}
