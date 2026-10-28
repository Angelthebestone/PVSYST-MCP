use crate::adapters::gui::ffi::escalar_a_dpi;
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
const ANCHO_ARBOL: i32 = 1149;

#[test]
fn should_aim_at_the_centre_of_row_zero_when_the_header_is_21px() {
    assert_eq!(arbol().dy_de_fila(0, ALTO_ARBOL), Ok(30));
}

#[test]
fn should_step_one_row_height_when_the_index_grows() {
    let a = arbol().dy_de_fila(3, ALTO_ARBOL).unwrap_or(0);
    let b = arbol().dy_de_fila(4, ALTO_ARBOL).unwrap_or(0);
    assert_eq!(b - a, 19);
}

#[test]
fn should_land_inside_the_band_measured_for_row_18() {
    let dy = arbol().dy_de_fila(18, ALTO_ARBOL).unwrap_or(0);
    assert!((363..=382).contains(&dy), "dy {dy}");
}

#[test]
fn should_scale_the_row_point_by_the_window_dpi_when_it_is_150_percent() {
    let dy = arbol().dy_de_fila(3, ALTO_ARBOL).unwrap_or(0);
    assert_eq!(escalar_a_dpi(dy, 144), dy * 3 / 2);
}

#[test]
fn should_count_the_slots_that_fit_when_the_control_has_a_header() {
    assert_eq!(arbol().filas_visibles(ALTO_ARBOL), 29);
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

#[test]
fn should_say_there_are_no_rows_when_the_control_is_shorter_than_its_header() {
    assert_eq!(
        arbol().dy_de_fila(0, 10),
        Err(ErrorFila::SinFilasVisibles { alto_px: 10 })
    );
}

#[test]
fn should_refuse_a_control_with_too_few_slots_to_tell_a_band_from_the_background() {
    assert!(matches!(
        arbol().dy_de_fila(0, 21 + 2 * 19),
        Err(ErrorFila::PocasFilas { visibles: 2, .. })
    ));
}

#[test]
fn should_not_overflow_when_the_index_is_huge() {
    assert!(arbol().dy_de_fila(usize::MAX, ALTO_ARBOL).is_err());
}

#[test]
fn should_give_the_click_point_when_it_lies_inside_the_control() {
    assert_eq!(
        arbol().punto_de_fila(1, (ANCHO_ARBOL, ALTO_ARBOL)),
        Ok((100, 49))
    );
}

#[test]
fn should_refuse_the_click_when_the_point_is_past_the_width_of_the_control() {
    assert_eq!(
        arbol().punto_de_fila(0, (80, ALTO_ARBOL)),
        Err(ErrorFila::FueraDelControl {
            x_px: 100,
            ancho_px: 80
        })
    );
}
