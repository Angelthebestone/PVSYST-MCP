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
        assert!(decode_base64(malo).is_err(), "aceptó {malo:?}");
    }
}
