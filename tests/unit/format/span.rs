use super::*;
use proptest::prelude::*;

const SAMPLE: &str = "PVObject_=pvProject\r\n  Comment=Hello\r\n  \
PVObject_SitePrj=pvSite\r\n    Site=Geneva\r\n  End of PVObject pvSite\r\n\
End of PVObject pvProject\r\n";

#[test]
fn should_parse_spans_and_match_types() {
    let spans = spans_of(SAMPLE.as_bytes());
    assert_eq!(spans.len(), 2);
    assert_eq!(spans[0].ty, "pvProject");
    assert_eq!(spans[1].ty, "pvSite");
    assert_eq!(spans[1].role.as_deref(), Some("SitePrj"));
}

#[test]
fn should_find_nested_span_by_selector() {
    let spans = spans_of(SAMPLE.as_bytes());
    let idx = find_span(&spans, "pvProject/pvSite").expect("nested span");
    assert_eq!(spans[idx].ty, "pvSite");

    let by_role = find_span(&spans, "role:SitePrj").expect("role span");
    assert_eq!(by_role, idx);
}

#[test]
fn should_find_indexed_span() {
    let text = "PVObject_=pvSubArray\r\n  NMod=10\r\nEnd of PVObject pvSubArray\r\n\
PVObject_=pvSubArray\r\n  NMod=20\r\nEnd of PVObject pvSubArray\r\n";
    let spans = spans_of(text.as_bytes());
    assert_eq!(spans.len(), 2);

    let first = find_span(&spans, "pvSubArray[0]").expect("first");
    let second = find_span(&spans, "pvSubArray[1]").expect("second");
    assert_eq!(first, 0);
    assert_eq!(second, 1);
    assert!(find_span(&spans, "pvSubArray[2]").is_none());
}

#[test]
fn should_return_none_when_selector_is_invalid() {
    let spans = spans_of(SAMPLE.as_bytes());
    assert!(find_span(&spans, "pvSubArray[abc]").is_none());
}

#[test]
fn should_preserve_lines_on_split_and_join() {
    let raw = b"line1\r\nline2\nline3";
    let lines = Lines::split(raw);
    assert_eq!(lines.join(), raw);
}

proptest! {
    #[test]
    fn scanning_arbitrary_bytes_never_panics(
        bytes in prop::collection::vec(any::<u8>(), 0..300)
    ) {
        let _ = spans_of(&bytes);
    }
}
