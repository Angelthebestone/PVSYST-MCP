use super::*;

fn control(class: &str, title: &str, visible: bool) -> Value {
    json!({"class": class, "title": title, "visible": visible})
}

#[test]
fn should_summarise_buttons_tabs_and_field_captions_of_a_form() {
    let controles = vec![
        control("TButton", "Aceptar", true),
        control("TButton", "Cancelar", true),
        control("TPageControl", "", true),
        control("TEdit", "Nombre", true),
        control("TComboBox", "Tipo", true),
    ];
    let res = resumen_formulario(&controles);
    assert_eq!(res["botones"], json!(["Aceptar", "Cancelar"]));
    assert_eq!(res["pestanas"], json!(1));
    assert_eq!(res["campos"], json!(["Nombre", "Tipo"]));
}

#[test]
fn should_leave_hidden_controls_out_of_the_summary() {
    let controles = vec![
        control("TButton", "Oculto", false),
        control("TPageControl", "", false),
    ];
    let res = resumen_formulario(&controles);
    assert_eq!(res["botones"], json!([]));
    assert_eq!(res["pestanas"], json!(0));
}
