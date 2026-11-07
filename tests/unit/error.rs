use super::*;

#[test]
fn io_error_keeps_its_source_and_context() {
    let e = Error::io("Cannot read x", io::Error::from(io::ErrorKind::NotFound));
    assert!(e.to_string().starts_with("Cannot read x: "));
    assert!(std::error::Error::source(&e).is_some());
    assert_eq!(e.kind(), "io");
}
