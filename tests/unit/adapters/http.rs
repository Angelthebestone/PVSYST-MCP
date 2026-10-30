use super::leer_acotado;
use crate::error::Error;
use std::io::Cursor;

#[test]
fn should_return_the_whole_body_when_it_fits_the_limit() {
    let cuerpo = leer_acotado(Cursor::new(b"abcde".to_vec()), 5).expect("cabe");

    assert_eq!(cuerpo, b"abcde");
}

#[test]
fn should_fail_when_the_body_is_one_byte_over_the_limit() {
    let error = leer_acotado(Cursor::new(b"abcdef".to_vec()), 5).expect_err("excede");

    assert!(matches!(error, Error::Network(_)), "{error}");
}

#[test]
fn should_read_only_past_the_limit_when_the_source_never_ends() {
    let infinito = std::io::repeat(b'x');

    assert!(leer_acotado(infinito, 1024).is_err());
}
