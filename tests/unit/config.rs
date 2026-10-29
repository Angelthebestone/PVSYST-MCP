use super::*;

#[test]
fn should_be_unset_when_variable_is_absent() {
    assert_eq!(valor_de_entorno_util(None), None);
}

#[test]
fn should_be_unset_when_variable_is_empty() {
    assert_eq!(valor_de_entorno_util(Some(OsString::from(""))), None);
}

#[test]
fn should_be_unset_when_variable_is_only_whitespace() {
    assert_eq!(valor_de_entorno_util(Some(OsString::from("  "))), None);
}

#[test]
fn should_read_version_when_folder_is_a_pvsyst_8_install() {
    assert_eq!(version_de("PVsyst8.1.8"), Some(vec![8, 1, 8]));
}

#[test]
fn should_rank_1_10_above_1_8_when_comparing_versions() {
    assert!(version_de("PVsyst8.1.10") > version_de("PVsyst8.1.8"));
}

#[test]
fn should_ignore_folder_when_it_is_data_or_another_major() {
    assert_eq!(version_de("PVsyst8.1_Data"), None);
    assert_eq!(version_de("PVsystBasic1.0"), None);
    assert_eq!(version_de("PVsyst7.4.0"), None);
}

#[test]
fn should_return_path_when_variable_has_a_value() {
    assert_eq!(
        valor_de_entorno_util(Some(OsString::from(r"C:\PVsyst8.1_Data"))),
        Some(PathBuf::from(r"C:\PVsyst8.1_Data"))
    );
}
