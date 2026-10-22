use super::*;
use std::sync::Arc;

#[test]
fn bgrx_is_converted_to_rgb() {
    assert_eq!(rgb_desde_bgrx(&[10, 20, 30, 0]), [30, 20, 10]);
}

#[test]
fn crop_selects_the_requested_rectangle() {
    let rgb: Vec<u8> = (0u8..6).flat_map(|i| [i, i, i]).collect();
    let dims = DimensionesRecorte {
        ancho: 3,
        alto: 2,
        x: 1,
        y: 0,
        ancho_corte: 2,
        alto_corte: 2,
    };
    let (out, cw, ch) = recortar_rgb(&rgb, &dims);
    assert_eq!((cw, ch), (2, 2));
    assert_eq!(out, [1, 1, 1, 2, 2, 2, 4, 4, 4, 5, 5, 5]);
}

#[test]
fn crop_is_clamped_to_the_image() {
    let rgb: Vec<u8> = (0u8..6).flat_map(|i| [i, i, i]).collect();
    let dims = DimensionesRecorte {
        ancho: 3,
        alto: 2,
        x: 5,
        y: -3,
        ancho_corte: 10,
        alto_corte: 10,
    };
    let (out, cw, ch) = recortar_rgb(&rgb, &dims);
    assert_eq!((cw, ch), (1, 2));
    assert_eq!(out, [2, 2, 2, 5, 5, 5]);
}

#[test]
fn should_handle_zero_dimensions_without_panic() {
    let dims = DimensionesRecorte {
        ancho: 0,
        alto: 0,
        x: 0,
        y: 0,
        ancho_corte: 10,
        alto_corte: 10,
    };
    let (out, cw, ch) = recortar_rgb(&[], &dims);
    assert_eq!((cw, ch), (0, 0));
    assert!(out.is_empty());
}

#[test]
fn should_reject_capture_output_path_outside_workspace_by_default() {
    let ws = std::env::temp_dir().join("test_pvsyst_ws_confined");
    let paths = crate::config::Paths {
        workspace: ws,
        cli_exe: PathBuf::new(),
        gui_exe: PathBuf::new(),
        aproximaciones: false,
        estado: Arc::default(),
    };
    let args = json!({
        "ruta": "../../escaped_capture.png"
    });
    let res = ruta_salida(&paths, &args, 0);
    assert!(res.is_err(), "Must reject escaping path by default");
}

#[test]
fn should_allow_capture_output_path_outside_workspace_when_explicitly_permitted() {
    let ws = std::env::temp_dir().join("test_pvsyst_ws_confined");
    let paths = crate::config::Paths {
        workspace: ws,
        cli_exe: PathBuf::new(),
        gui_exe: PathBuf::new(),
        aproximaciones: false,
        estado: Arc::default(),
    };
    let outside = std::env::temp_dir().join("safe_capture_outside.png");
    let args = json!({
        "ruta": outside.to_string_lossy(),
        "permitir_fuera_del_workspace": true
    });
    let res = ruta_salida(&paths, &args, 0);
    assert!(
        res.is_ok(),
        "Must allow outside path when explicitly permitted"
    );
}

#[test]
fn drawn_area_is_the_physical_size_divided_by_the_scale() {
    let lienzo = Lienzo::dibujado((1200, 960), 1.5);
    assert_eq!((lienzo.ancho, lienzo.alto), (800, 640));
}

#[test]
fn drawn_area_is_the_whole_capture_at_100_percent() {
    let lienzo = Lienzo::dibujado((1200, 960), 1.0);
    assert_eq!((lienzo.ancho, lienzo.alto), (1200, 960));
}

#[test]
fn black_margin_of_a_scaled_capture_is_dropped() {
    // 4x3 capture of a window drawn in its top-left 2x2 at scale 2: the
    // marked pixels are the window, the zeros the margin.
    let mut rgb = vec![0u8; 4 * 3 * 3];
    for (x, y) in [(0, 0), (1, 0), (0, 1), (1, 1)] {
        let i = (y * 4 + x) * 3;
        rgb[i..i + 3].copy_from_slice(&[9, 9, 9]);
    }
    let lienzo = Lienzo::dibujado((4, 3), 2.0);
    let recortado = solo_area_dibujada(rgb, 4, lienzo);
    assert_eq!((lienzo.ancho, lienzo.alto), (2, 2));
    assert_eq!(recortado, vec![9u8; 2 * 2 * 3]);
}

#[test]
fn unscaled_capture_is_returned_untouched() {
    let rgb: Vec<u8> = (0u8..18).collect();
    let lienzo = Lienzo::dibujado((3, 2), 1.0);
    assert_eq!(solo_area_dibujada(rgb.clone(), 3, lienzo), rgb);
}
