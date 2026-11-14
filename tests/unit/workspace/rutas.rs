use super::*;
use tempfile::tempdir;

#[test]
fn should_refuse_candidate_names_that_leave_the_base_when_searching() {
    let base = tempdir().expect("base");
    let fuera = tempdir().expect("fuera");
    let ajeno = fuera.path().join("ajeno.txt");
    fs::write(&ajeno, "x").expect("ajeno");
    let nombres = [ajeno.to_string_lossy().into_owned(), "../ajeno.txt".into()];

    assert_eq!(
        primer_existente_en_bases(&[base.path().to_path_buf()], &nombres),
        None
    );

    let res = primer_existente_en_workspace(base.path(), &["../x".into()], Alcance::Workspace);
    assert!(matches!(res, Err(Error::Invalid(_))));
}
