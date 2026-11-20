use super::leer_acotado;
use crate::error::Error;
use std::io::Cursor;

#[test]
fn should_fail_when_the_body_is_one_byte_over_the_limit() {
    let error = leer_acotado(Cursor::new(b"abcdef".to_vec()), 5).expect_err("excede");

    assert!(matches!(error, Error::Network(_)), "{error}");
}
