use super::{campo_por_etiqueta, CampoEtiquetado, Desambiguador};

fn campo(hwnd: isize, etiqueta: Option<&str>) -> CampoEtiquetado {
    CampoEtiquetado {
        hwnd,
        clase: "TEdit".into(),
        etiqueta: etiqueta.map(str::to_string),
        // Later windows sit lower on the form.
        posicion: (hwnd as i32 * 10, 0),
    }
}

#[test]
fn should_refuse_to_guess_when_a_fragment_matches_several_fields() {
    let campos = [
        campo(1, Some("Factor de pérdida constante Uc")),
        campo(2, Some("Factor de pérdida del viento Uv")),
    ];
    let error = campo_por_etiqueta(&campos, "factor de pérdida", Desambiguador::default())
        .unwrap_err()
        .to_string();
    assert!(
        error.contains("several") && error.contains("Uc") && error.contains("Uv"),
        "{error}"
    );
}

/// Three «Núm. de inversores» on the system form: `indice` counts them top to
/// bottom, whatever order the windows were enumerated in.
#[test]
fn should_pick_among_fields_with_the_same_label_by_position_and_class() {
    let mut combo = campo(1, Some("Núm. de inversores"));
    combo.clase = "TComboBox".into();
    let campos = [
        campo(3, Some("Núm. de inversores")),
        combo,
        campo(2, Some("Núm. de inversores")),
    ];
    let como = |clase, indice| Desambiguador { clase, indice };

    let primero = campo_por_etiqueta(&campos, "Núm. de inversores", como(None, Some(0))).unwrap();
    let ultimo = campo_por_etiqueta(&campos, "Núm. de inversores", como(None, Some(2))).unwrap();
    let editable =
        campo_por_etiqueta(&campos, "Núm. de inversores", como(Some("TEdit"), Some(0))).unwrap();
    let fuera = campo_por_etiqueta(&campos, "Núm. de inversores", como(None, Some(3))).unwrap_err();
    let sin_indice =
        campo_por_etiqueta(&campos, "Núm. de inversores", Desambiguador::default()).unwrap_err();

    assert_eq!((primero.hwnd, ultimo.hwnd, editable.hwnd), (1, 3, 2));
    assert!(fuera.to_string().contains("out of range"), "{fuera}");
    assert!(sin_indice.to_string().contains("indice"), "{sin_indice}");
}
