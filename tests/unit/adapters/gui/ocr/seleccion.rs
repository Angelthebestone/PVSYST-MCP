use super::{campo_por_etiqueta, valor_logico, CampoEtiquetado};

fn campo(hwnd: isize, etiqueta: Option<&str>) -> CampoEtiquetado {
    CampoEtiquetado {
        hwnd,
        clase: "TEdit".into(),
        etiqueta: etiqueta.map(str::to_string),
    }
}

#[test]
fn should_pick_the_field_when_label_matches_ignoring_case_and_spacing() {
    let campos = [
        campo(1, Some("Factor de pérdida constante Uc")),
        campo(2, Some("Uv")),
    ];
    let elegido = campo_por_etiqueta(&campos, "factor  de PÉRDIDA constante uc").unwrap();
    assert_eq!(elegido.hwnd, 1);
}

#[test]
fn should_prefer_the_exact_label_when_another_label_contains_it() {
    let campos = [
        campo(1, Some("Potencia")),
        campo(2, Some("Potencia nominal")),
    ];
    assert_eq!(campo_por_etiqueta(&campos, "Potencia").unwrap().hwnd, 1);
}

#[test]
fn should_accept_a_fragment_when_it_identifies_one_field() {
    let campos = [
        campo(1, Some("Factor de pérdida constante Uc")),
        campo(2, Some("Factor de pérdida del viento Uv")),
    ];
    assert_eq!(campo_por_etiqueta(&campos, "viento").unwrap().hwnd, 2);
}

#[test]
fn should_refuse_to_guess_when_a_fragment_matches_several_fields() {
    let campos = [
        campo(1, Some("Factor de pérdida constante Uc")),
        campo(2, Some("Factor de pérdida del viento Uv")),
    ];
    let error = campo_por_etiqueta(&campos, "factor de pérdida")
        .unwrap_err()
        .to_string();
    assert!(
        error.contains("varios") && error.contains("Uc") && error.contains("Uv"),
        "{error}"
    );
}

#[test]
fn should_list_the_labels_seen_when_no_field_matches() {
    let campos = [campo(1, Some("Uc")), campo(2, None)];
    let error = campo_por_etiqueta(&campos, "inexistente")
        .unwrap_err()
        .to_string();
    assert!(error.contains("Uc"), "{error}");
}

#[test]
fn should_read_logical_values_in_spanish_and_english() {
    assert!(valor_logico("Sí").unwrap());
    assert!(valor_logico(" TRUE ").unwrap());
    assert!(!valor_logico("no").unwrap());
    assert!(!valor_logico("0").unwrap());
    assert!(valor_logico("quizá").is_err());
}
