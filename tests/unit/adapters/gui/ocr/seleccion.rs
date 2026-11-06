use super::{campo_por_etiqueta, CampoEtiquetado};

fn campo(hwnd: isize, etiqueta: Option<&str>) -> CampoEtiquetado {
    CampoEtiquetado {
        hwnd,
        clase: "TEdit".into(),
        etiqueta: etiqueta.map(str::to_string),
    }
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
