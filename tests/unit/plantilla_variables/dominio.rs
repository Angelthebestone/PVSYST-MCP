use super::*;

#[test]
fn should_recognize_valid_variables_when_canonical_or_case_different() {
    assert_eq!(buscar_variable_valida("E_Grid"), Some("E_Grid"));
    assert_eq!(buscar_variable_valida("e_grid"), Some("E_Grid"));
    assert_eq!(buscar_variable_valida("  GlobInc "), Some("GlobInc"));
    assert_eq!(buscar_variable_valida("PR"), Some("PR"));
    assert_eq!(buscar_variable_valida("HSol"), Some("HSol"));
    assert_eq!(buscar_variable_valida("IL_Pmax"), Some("IL_Pmax"));
}

#[test]
fn should_reject_unknown_variable_name() {
    assert_eq!(buscar_variable_valida("VariableInexistente"), None);
    assert_eq!(buscar_variable_valida(""), None);
}

#[test]
fn should_parse_rvt_when_standard_line_supplied() {
    let raw = "  ResultVars=HSol;AzSol;AngInc;E_Grid;\n";
    let vars = parse_rvt(raw).expect("parseable");
    assert_eq!(vars, vec!["HSol", "AzSol", "AngInc", "E_Grid"]);
}

#[test]
fn should_add_valid_variable_when_not_already_present() {
    let actuales = vec!["HSol".to_string(), "AzSol".to_string()];
    let resultado = anadir_variables(&actuales, &["E_Grid", "PR"]).expect("anadido");
    assert_eq!(resultado, vec!["HSol", "AzSol", "E_Grid", "PR"]);
}

#[test]
fn should_fail_to_add_when_variable_name_is_invalid() {
    let actuales = vec!["HSol".to_string()];
    let err = anadir_variables(&actuales, &["NoExiste"]).expect_err("debe fallar");
    assert!(err.to_string().contains("NoExiste"));
}

#[test]
fn should_remove_variable_when_present() {
    let actuales = vec!["HSol".to_string(), "AzSol".to_string(), "PR".to_string()];
    let resultado = eliminar_variables(&actuales, &["AzSol"]).expect("eliminado");
    assert_eq!(resultado, vec!["HSol", "PR"]);
}

#[test]
fn should_fail_to_remove_when_variable_not_present() {
    let actuales = vec!["HSol".to_string(), "PR".to_string()];
    let err = eliminar_variables(&actuales, &["AzSol"]).expect_err("no encontrada");
    assert!(err.to_string().contains("AzSol"));
}

#[test]
fn should_reorder_variables_when_all_match() {
    let actuales = vec!["HSol".to_string(), "AzSol".to_string(), "PR".to_string()];
    let nuevo_orden = &["PR", "HSol", "AzSol"];
    let resultado = reordenar_variables(&actuales, nuevo_orden).expect("reordenado");
    assert_eq!(resultado, vec!["PR", "HSol", "AzSol"]);
}

#[test]
fn should_fail_to_reorder_when_length_or_elements_mismatch() {
    let actuales = vec!["HSol".to_string(), "PR".to_string()];
    let err = reordenar_variables(&actuales, &["PR"]).expect_err("longitud distinta");
    assert!(err.to_string().contains("no coincide"));
}

#[test]
fn should_roundtrip_serialize_and_parse() {
    let vars = vec!["HSol".to_string(), "E_Grid".to_string(), "PR".to_string()];
    let serializado = serializar_rvt(&vars);
    let parsed = parse_rvt(&serializado).expect("roundtrip parse");
    assert_eq!(parsed, vars);
}
