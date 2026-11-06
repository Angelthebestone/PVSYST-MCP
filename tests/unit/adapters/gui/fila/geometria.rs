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
fn should_aim_at_the_centre_of_row_zero_when_the_header_is_21px() {
    assert_eq!(arbol().dy_de_fila(0, ALTO_ARBOL), Ok(30));
}

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
