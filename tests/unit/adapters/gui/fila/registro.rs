use crate::adapters::gui::fila::error::ErrorFila;
use crate::adapters::gui::fila::registro::{geometria_de, Control};

const ARBOL: Control<'static> = Control {
    clase: "TVirtualStringTree",
    anfitrion: "TProjectLoadForm",
    titulo: "Proyectos",
};
const LISTA_DE_MODULOS: Control<'static> = Control {
    clase: "TListBox",
    anfitrion: "TChoixObjetPVForm",
    titulo: "Eligiendo un módulo FV",
};

#[test]
fn should_use_19px_rows_when_the_control_is_the_project_tree() {
    let g = geometria_de(&ARBOL).expect("declarada");
    assert_eq!(g.alto_fila_px, 19);
}

#[test]
fn should_use_13px_rows_when_the_control_is_the_module_list() {
    let g = geometria_de(&LISTA_DE_MODULOS).expect("declarada");
    assert_eq!(g.alto_fila_px, 13);
}

#[test]
fn should_refuse_a_list_of_another_chooser_because_only_the_module_list_was_measured() {
    let inversores = Control {
        titulo: "Eligiendo un inversor de red",
        ..LISTA_DE_MODULOS
    };
    assert!(matches!(
        geometria_de(&inversores),
        Err(ErrorFila::ClaseSinGeometria { .. })
    ));
}

#[test]
fn should_refuse_a_tree_of_another_window_even_though_its_class_was_measured() {
    let otro = Control {
        anfitrion: "TOtroForm",
        ..ARBOL
    };
    assert!(geometria_de(&otro).is_err());
}

#[test]
fn should_refuse_the_header_grid_when_it_has_no_measured_geometry() {
    let rejilla = Control {
        clase: "TStringGrid",
        ..LISTA_DE_MODULOS
    };
    assert!(matches!(
        geometria_de(&rejilla),
        Err(ErrorFila::ClaseSinGeometria { ref clase, .. }) if clase == "TStringGrid"
    ));
}

#[test]
fn should_list_where_the_geometry_was_measured_when_the_control_is_unknown() {
    let texto = geometria_de(&Control {
        clase: "TPanel",
        ..ARBOL
    })
    .unwrap_err()
    .to_string();
    assert!(
        texto.contains("TVirtualStringTree de TProjectLoadForm"),
        "{texto}"
    );
    assert!(texto.contains("TListBox de TChoixObjetPVForm"), "{texto}");
}

#[test]
fn should_report_a_request_error_when_the_control_is_unknown() {
    let error = geometria_de(&Control {
        clase: "TPanel",
        ..ARBOL
    })
    .unwrap_err()
    .a_error();
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

#[test]
fn should_report_a_gui_error_when_the_click_closed_another_window() {
    let error = ErrorFila::VentanaCerrada {
        indice: 0,
        clases: "TMessageForm".into(),
    }
    .a_error();
    assert!(matches!(error, crate::error::Error::Gui(_)));
}

#[test]
fn should_use_26px_rows_when_the_control_is_the_object_table_of_the_3d_editor() {
    let tabla = Control {
        clase: "TVirtualStringTree",
        anfitrion: "TShdObjectSelectionForm",
        titulo: "Lista y gestión de objetos",
    };
    let g = geometria_de(&tabla).expect("declarada");
    assert_eq!(g.alto_fila_px, 26);
    assert_eq!(g.centro_de_fila(0), 45);
}
