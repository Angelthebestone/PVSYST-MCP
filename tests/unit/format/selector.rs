use super::*;

#[test]
fn should_parse_simple_name_with_default_index_zero() {
    let segs = parse_selector("pvProject").expect("valid selector");
    assert_eq!(
        segs,
        vec![Seg {
            name: "pvProject".into(),
            index: 0
        }]
    );
}

#[test]
fn should_parse_indexed_segment() {
    let segs = parse_selector("pvSubArray[1]").expect("valid indexed selector");
    assert_eq!(
        segs,
        vec![Seg {
            name: "pvSubArray".into(),
            index: 1
        }]
    );
}

#[test]
fn should_parse_role_prefix() {
    let segs = parse_selector("role:SitePrj").expect("valid role selector");
    assert_eq!(
        segs,
        vec![Seg {
            name: "SitePrj".into(),
            index: 0
        }]
    );
}

#[test]
fn should_parse_multi_segment_path() {
    let segs = parse_selector("pvVCalcul/pvSubArray[2]").expect("valid multi-segment");
    assert_eq!(
        segs,
        vec![
            Seg {
                name: "pvVCalcul".into(),
                index: 0
            },
            Seg {
                name: "pvSubArray".into(),
                index: 2
            },
        ]
    );
}

#[test]
fn should_reject_empty_selector() {
    let err = parse_selector("").unwrap_err();
    assert!(matches!(err, Error::Invalid(_)));
    assert!(err.to_string().contains("''"));
}

#[test]
fn should_reject_empty_segment_when_slashes_are_consecutive() {
    let err = parse_selector("a//b").unwrap_err();
    assert!(matches!(err, Error::Invalid(_)));
    assert!(err.to_string().contains("'a//b'"));
}

#[test]
fn should_reject_leading_or_trailing_slash() {
    assert!(parse_selector("/pvProject").is_err());
    assert!(parse_selector("pvProject/").is_err());
}

#[test]
fn should_reject_empty_name() {
    let err = parse_selector("[0]").unwrap_err();
    assert!(matches!(err, Error::Invalid(_)));
    assert!(err.to_string().contains("'[0]'"));

    let err_role = parse_selector("role:[0]").unwrap_err();
    assert!(matches!(err_role, Error::Invalid(_)));
    assert!(err_role.to_string().contains("'role:[0]'"));
}

#[test]
fn should_reject_missing_closing_bracket() {
    let err = parse_selector("pvSubArray[1").unwrap_err();
    assert!(matches!(err, Error::Invalid(_)));
    assert!(err.to_string().contains("'pvSubArray[1'"));
}

#[test]
fn should_reject_text_after_closing_bracket() {
    let err = parse_selector("pvSubArray[1]abc").unwrap_err();
    assert!(matches!(err, Error::Invalid(_)));
    assert!(err.to_string().contains("'pvSubArray[1]abc'"));
}

#[test]
fn should_reject_non_integer_index() {
    let err = parse_selector("pvSubArray[abc]").unwrap_err();
    assert!(matches!(err, Error::Invalid(_)));
    assert!(err.to_string().contains("'pvSubArray[abc]'"));

    let err_neg = parse_selector("pvSubArray[-1]").unwrap_err();
    assert!(matches!(err_neg, Error::Invalid(_)));
    assert!(err_neg.to_string().contains("'pvSubArray[-1]'"));
}

#[test]
fn should_reject_empty_index() {
    let err = parse_selector("pvSubArray[]").unwrap_err();
    assert!(matches!(err, Error::Invalid(_)));
    assert!(err.to_string().contains("'pvSubArray[]'"));
}

#[test]
fn should_reject_unopened_closing_bracket() {
    let err = parse_selector("pvSubArray]").unwrap_err();
    assert!(matches!(err, Error::Invalid(_)));
    assert!(err.to_string().contains("'pvSubArray]'"));
}

#[test]
fn should_reject_nested_brackets() {
    let err = parse_selector("pvSubArray[1][2]").unwrap_err();
    assert!(matches!(err, Error::Invalid(_)));
    assert!(err.to_string().contains("'pvSubArray[1][2]'"));
}
