use crate::adapters::gui::fila::error::ErrorFila;
use crate::adapters::gui::fila::geometria::Geometria;
use crate::adapters::gui::fila::registro::{geometria_de, Control};

/// The geometry production declares for the project tree, not a copy of it.
fn arbol() -> Geometria {
    let control = Control {
        clase: "TVirtualStringTree",
        anfitrion: "TProjectLoadForm",
        titulo: "Proyectos",
    };
    *geometria_de(&control).expect("el árbol de proyectos está declarado")
}

/// Captured height of the project tree: 29 row slots after a 21 px header.
const ALTO_ARBOL: i32 = 578;

#[test]
fn should_reject_the_row_when_it_is_past_the_last_visible_slot() {
    let error = arbol().dy_de_fila(29, ALTO_ARBOL);
    assert_eq!(
        error,
        Err(ErrorFila::FueraDeRango {
            indice: 29,
            visibles: 29,
            ultima: 28
        })
    );
}

/// The module chooser is measured by its Spanish title: another title is a language
/// difference, not a control nobody measured.
#[test]
fn should_say_the_title_differs_when_the_chooser_is_in_another_language() {
    let lista = |titulo| Control {
        clase: "TListBox",
        anfitrion: "TChoixObjetPVForm",
        titulo,
    };
    assert!(geometria_de(&lista("Eligiendo un módulo FV")).is_ok());
    let error = geometria_de(&lista("Choosing a PV module")).unwrap_err();
    assert!(matches!(error, ErrorFila::TituloDistinto { .. }), "{error}");
    let sin_medir = Control {
        clase: "TListBox",
        anfitrion: "TOtroForm",
        titulo: "x",
    };
    assert!(matches!(
        geometria_de(&sin_medir),
        Err(ErrorFila::ClaseSinGeometria { .. })
    ));
}
