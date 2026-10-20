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
fn should_return_path_when_variable_has_a_value() {
    assert_eq!(
        valor_de_entorno_util(Some(OsString::from(r"C:\PVsyst8.1_Data"))),
        Some(PathBuf::from(r"C:\PVsyst8.1_Data"))
    );
}
