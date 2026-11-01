use super::*;

#[test]
fn should_round_trip_every_padding_length() {
    for data in [&b""[..], b"a", b"ab", b"abc", b"\x00\xff\x10"] {
        assert_eq!(decode_base64(&base64(data)).expect("valid"), data);
    }
    assert_eq!(decode_base64("YW\r\nJj").expect("wrapped"), b"abc");
}

#[test]
fn should_reject_non_canonical_base64() {
    for malo in ["YWJ", "YQ=", "Y=Q=", "YQ===", "YR==", "YW Jj", "YWJj=AAA"] {
        assert!(
            matches!(decode_base64(malo), Err(crate::error::Error::Invalid(_))),
            "aceptó {malo:?}"
        );
    }
}

#[test]
fn should_encode_with_padding_per_rfc_4648() {
    assert_eq!(base64(b"Man"), "TWFu");
    assert_eq!(base64(b"Ma"), "TWE=");
    assert_eq!(base64(b"M"), "TQ==");
    assert_eq!(base64(b""), "");
}
