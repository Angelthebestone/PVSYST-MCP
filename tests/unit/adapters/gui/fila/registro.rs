use crate::adapters::gui::fila::error::ErrorFila;
use crate::adapters::gui::fila::registro::geometria_de;

#[test]
fn should_use_19px_rows_when_the_control_is_the_project_tree() {
    let g = geometria_de("TVirtualStringTree").expect("declarada");
    assert_eq!(g.alto_fila_px, 19);
}

#[test]
fn should_use_13px_rows_when_the_control_is_the_component_list() {
    let g = geometria_de("TListBox").expect("declarada");
    assert_eq!(g.alto_fila_px, 13);
}

#[test]
fn should_refuse_the_header_grid_when_it_has_no_measured_geometry() {
    let error = geometria_de("TStringGrid");
    assert!(matches!(
        error,
        Err(ErrorFila::ClaseSinGeometria { ref clase, .. }) if clase == "TStringGrid"
    ));
}

#[test]
fn should_list_the_supported_classes_when_the_class_is_unknown() {
    let texto = geometria_de("TPanel").unwrap_err().to_string();
    assert!(texto.contains("TVirtualStringTree") && texto.contains("TListBox"));
}

#[test]
fn should_report_a_request_error_when_the_class_is_unknown() {
    let error = geometria_de("TPanel").unwrap_err().a_error();
    assert!(matches!(error, crate::error::Error::Invalid(_)));
}

#[test]
fn should_report_a_gui_error_when_the_click_opened_the_item() {
    let error = ErrorFila::Activada {
        indice: 3,
        clases: "TPVModuleForm".into(),
    }
    .a_error();
    assert!(matches!(error, crate::error::Error::Gui(_)));
}
