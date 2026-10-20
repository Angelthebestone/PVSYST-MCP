use super::*;

#[test]
fn solo_reconoce_el_prefijo_temporal() {
    assert!(es_temporal("mcp_tmp_demo.VC0"));
    assert!(es_temporal("MCP_TMP_demo.PRJ"));
    assert!(!es_temporal("MCP_E2E.VC0"));
    assert!(!es_temporal("mcp_tm.PRJ"));
    assert!(!es_temporal("Ciudad_é.SIT"));
}

#[test]
fn should_recognize_scratch_file_when_prefix_matches() {
    assert!(es_temporal("mcp_tmp_test.VC0"));
    assert!(!es_temporal("production.PRJ"));
}
