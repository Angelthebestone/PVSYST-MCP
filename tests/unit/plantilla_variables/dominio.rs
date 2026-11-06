use super::*;

#[test]
fn should_roundtrip_serialize_and_parse() {
    let vars = vec!["HSol".to_string(), "E_Grid".to_string(), "PR".to_string()];
    let serializado = serializar_rvt(&vars);
    let parsed = parse_rvt(&serializado).expect("roundtrip parse");
    assert_eq!(parsed, vars);
}

#[test]
fn should_keep_other_lines_when_rewriting_the_variable_line() {
    let original = "Plantilla del MCP\r\n  ResultVars=HSol;AzSol\r\nFin\r\n";

    let nuevo = reescribir_rvt(original, &["E_Grid".to_string()]);

    assert_eq!(nuevo, "Plantilla del MCP\r\n  ResultVars=E_Grid\nFin\r\n");
}

#[test]
fn should_read_the_result_vars_line_when_it_is_not_the_first() {
    let texto = "Plantilla del MCP\n  ResultVars=HSol;AzSol\n";

    assert_eq!(parse_rvt(texto).expect("variables"), ["HSol", "AzSol"]);
}
