use super::*;
use crate::idioma::Idioma;

#[test]
fn should_reject_conflicting_alias_when_the_values_differ() {
    let tool_leer = REGISTRO.iter().find(|h| h.nombre == "leer").unwrap();
    let conflict = json!({ "archivo": "a.txt", "file": "b.txt" });
    let err = tool_leer
        .preparar_args(&conflict, Idioma::En)
        .expect_err("conflicting alias and canonical must fail");
    let msg = err.to_string();
    assert!(
        msg.contains("Argument conflict") && msg.contains("archivo") && msg.contains("file"),
        "error must explain the conflict: {msg}"
    );

    let same_value = json!({ "archivo": "same.txt", "file": "same.txt" });
    assert!(
        tool_leer.preparar_args(&same_value, Idioma::En).is_ok(),
        "identical values for alias and canonical should succeed"
    );
}

#[test]
fn should_keep_the_declared_source_property_when_normalizing_sitio_aliases() {
    let tool_sitio = REGISTRO.iter().find(|h| h.nombre == "sitio").unwrap();
    let args = json!({ "accion": "crear", "source": "NASA SSE" });
    let preparados = tool_sitio
        .preparar_args(&args, Idioma::En)
        .expect("preparar_args");
    assert_eq!(preparados.get("source"), Some(&json!("NASA SSE")));
    assert_eq!(preparados.get("origen"), None);
}
