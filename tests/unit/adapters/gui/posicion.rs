//! The point a caller reads from `vista` must land on the control `vista` named.

use super::*;

fn rect([left, top, ancho, alto]: [i32; 4]) -> Rect {
    Rect {
        left,
        top,
        right: left + ancho,
        bottom: top + alto,
    }
}

/// `modelo` at (121, 75) and `nombre_archivo` 23 px above it, as on TPVModuleForm
/// whose title bar is 23 px high at 96 DPI (GUI-01).
#[test]
fn should_pick_the_control_vista_published_at_that_point() {
    let ventana = rect([100, 50, 600, 400]);
    let nombre_archivo = (1, rect([100 + 121, 50 + 52, 150, 21]));
    let modelo = (2, rect([100 + 121, 50 + 75, 150, 21]));

    let elegido = control_en((ventana, 1.0), (121, 75), [nombre_archivo, modelo]);

    assert_eq!(elegido, Some(2));
}

#[test]
fn should_read_a_dpi_unaware_window_at_its_96_dpi_layout() {
    let ventana = rect([100, 50, 750, 500]);
    let modelo = (2, rect([100 + 151, 50 + 94, 188, 26]));

    assert_eq!(posicion_de_diseno(ventana, modelo.1, 1.25), (121, 75));
    assert_eq!(control_en((ventana, 1.25), (121, 75), [modelo]), Some(2));
}
