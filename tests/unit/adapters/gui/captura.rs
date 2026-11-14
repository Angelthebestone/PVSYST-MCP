use super::*;

#[test]
fn should_convert_bgrx_pixels_to_rgb_when_decoding_a_capture() {
    assert_eq!(rgb_desde_bgrx(&[10, 20, 30, 0]), [30, 20, 10]);
}

#[test]
fn should_reject_capture_output_path_outside_workspace_by_default() {
    let ws = std::env::temp_dir().join("test_pvsyst_ws_confined");
    let paths = crate::config::Paths::para_test(ws);
    let args = json!({
        "ruta": "../../escaped_capture.png"
    });
    let res = ruta_salida(&paths, &args, 0);
    assert!(
        matches!(res, Err(crate::error::Error::Invalid(_))),
        "Must reject escaping path by default"
    );
}
