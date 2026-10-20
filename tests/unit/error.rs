use super::*;

#[test]
fn missing_arg_text_includes_the_hint_only_when_given() {
    let sin = Error::MissingArg {
        key: "archivo".into(),
        hint: String::new(),
    };
    let con = Error::MissingArg {
        key: "archivo".into(),
        hint: "ruta".into(),
    };
    assert_eq!(sin.to_string(), "Missing required argument 'archivo'");
    assert_eq!(
        con.to_string(),
        "Missing required argument 'archivo' (ruta)"
    );
}

#[test]
fn io_error_keeps_its_source_and_context() {
    let e = Error::io("Cannot read x", io::Error::from(io::ErrorKind::NotFound));
    assert!(e.to_string().starts_with("Cannot read x: "));
    assert!(std::error::Error::source(&e).is_some());
    assert_eq!(e.kind(), "io");
}
