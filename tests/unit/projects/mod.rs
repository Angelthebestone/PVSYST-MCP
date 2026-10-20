use super::*;

#[test]
fn should_identify_variant_kind_when_stem_matches() {
    let p = Path::new("Projects/DEMO.VC0");
    assert_eq!(variant_kind(p, "DEMO"), Some("VC0".into()));
}

#[test]
fn should_reject_variant_kind_when_stem_differs() {
    let p = Path::new("Projects/OTHER.VC0");
    assert_eq!(variant_kind(p, "DEMO"), None);
}

#[test]
fn should_reject_escape_when_confinement_is_workspace() {
    let ws = std::env::temp_dir();
    let res = resolve_project_file_con_alcance(&ws, "../../secret.PRJ", Alcance::Workspace);
    assert!(res.is_err());
}
