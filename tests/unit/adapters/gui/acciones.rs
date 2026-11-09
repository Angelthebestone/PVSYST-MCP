use super::*;
use serde_json::json;

fn ventana(left: i32, top: i32, right: i32, bottom: i32) -> ffi::Rect {
    ffi::Rect {
        left,
        top,
        right,
        bottom,
    }
}

/// BUG-24: `objetivo` was ignored and the click went to the centre of the window.
#[test]
fn should_click_the_screen_point_the_request_names_when_it_is_on_the_window() {
    let punto = punto_pedido(&json!({ "objetivo": { "x": 919, "y": 565 } })).expect("a point");

    let destino = punto_de_clic_fisico(ventana(0, 0, 1920, 1080), 96, punto);

    assert_eq!(destino.expect("on the window"), (919, 565));
}

#[test]
fn should_refuse_a_point_outside_the_window_on_the_shared_desktop() {
    let punto = punto_pedido(&json!({ "objetivo": { "x": 3000, "y": 10 } })).expect("a point");

    let error = punto_de_clic_fisico(ventana(0, 0, 800, 600), 96, punto).expect_err("outside");

    assert!(error.to_string().contains("3000"), "{error}");
}

#[test]
fn should_not_take_objetivo_and_dx_dy_together() {
    let args = json!({ "objetivo": { "x": 1, "y": 1 }, "dx": 5, "dy": 5 });

    assert!(punto_pedido(&args).is_err());
}

/// GUI-06: `texto` was read by nobody and the click went to the window itself.
#[test]
fn should_refuse_texto_and_id_that_no_click_reads() {
    for args in [
        json!({ "ventana": "TDimensForm", "texto": "Sitio y datos meteo" }),
        json!({ "ventana": "TDimensForm", "id": 23 }),
    ] {
        let error = exigir_sin_texto_ni_id(&args).expect_err("ignored argument");

        assert!(error.to_string().contains("clic_texto"), "{error}");
    }
}

/// GUI-14: the failed close named neither the confirmation nor its buttons.
#[test]
fn should_name_the_confirmation_and_its_buttons_when_a_close_leaves_the_window_open() {
    let botones = vec![(0xE09E0, "Cancelar".into()), (0xE09EA, "No".into())];

    let texto = sigue_abierta("TPerspectiveForm", Some((0xE09D0, botones)));

    assert!(texto.contains("TMessageForm 0xE09D0"), "{texto}");
    assert!(texto.contains("'No' (0xE09EA)"), "{texto}");
}
