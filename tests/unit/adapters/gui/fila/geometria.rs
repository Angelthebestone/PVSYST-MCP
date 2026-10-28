use crate::adapters::gui::ffi::escalar_a_dpi;
use crate::adapters::gui::fila::error::ErrorFila;
use crate::adapters::gui::fila::geometria::Geometria;

const ARBOL: Geometria = Geometria {
    clase: "arbol",
    cabecera_px: 21,
    alto_fila_px: 19,
    x_px: 100,
};

/// Captured height of the project tree: 29 row slots after a 21 px header.
const ALTO_ARBOL: i32 = 578;

#[test]
fn should_aim_at_the_centre_of_row_zero_when_the_header_is_21px() {
    assert_eq!(ARBOL.dy_de_fila(0, ALTO_ARBOL), Ok(30));
}

#[test]
fn should_step_one_row_height_when_the_index_grows() {
    let a = ARBOL.dy_de_fila(3, ALTO_ARBOL).unwrap_or(0);
    let b = ARBOL.dy_de_fila(4, ALTO_ARBOL).unwrap_or(0);
    assert_eq!(b - a, 19);
}

#[test]
fn should_land_inside_the_band_measured_for_row_18() {
    let dy = ARBOL.dy_de_fila(18, ALTO_ARBOL).unwrap_or(0);
    assert!((363..=382).contains(&dy), "dy {dy}");
}

#[test]
fn should_scale_the_row_point_by_the_window_dpi_when_it_is_150_percent() {
    let dy = ARBOL.dy_de_fila(3, ALTO_ARBOL).unwrap_or(0);
    assert_eq!(escalar_a_dpi(dy, 144), dy * 3 / 2);
}

#[test]
fn should_count_the_slots_that_fit_when_the_control_has_a_header() {
    assert_eq!(ARBOL.filas_visibles(ALTO_ARBOL), 29);
}

#[test]
fn should_reject_the_row_when_it_is_past_the_last_visible_slot() {
    let error = ARBOL.dy_de_fila(29, ALTO_ARBOL);
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
fn should_reject_every_row_when_the_control_is_shorter_than_its_header() {
    assert!(ARBOL.dy_de_fila(0, 10).is_err());
}

#[test]
fn should_not_overflow_when_the_index_is_huge() {
    assert!(ARBOL.dy_de_fila(usize::MAX, ALTO_ARBOL).is_err());
}
