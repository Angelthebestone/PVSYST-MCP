use super::*;
use crate::idioma::Idioma;
use serde_json::json;

fn ventana([left, top, right, bottom]: [i32; 4]) -> ffi::Rect {
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
    let punto =
        punto_pedido(&json!({ "objetivo": { "x": 919, "y": 565 } }), Idioma::Es).expect("a point");

    let destino = punto_en_rect((ventana([0, 0, 1920, 1080]), 96), punto, Idioma::Es);

    assert_eq!(destino.expect("on the window"), (919, 565));
}

#[test]
fn should_refuse_a_point_outside_the_window_on_the_shared_desktop() {
    let punto =
        punto_pedido(&json!({ "objetivo": { "x": 3000, "y": 10 } }), Idioma::Es).expect("a point");

    let error =
        punto_en_rect((ventana([0, 0, 800, 600]), 96), punto, Idioma::Es).expect_err("outside");

    assert!(error.to_string().contains("3000"), "{error}");
}

#[test]
fn should_not_take_objetivo_and_dx_dy_together() {
    let args = json!({ "objetivo": { "x": 1, "y": 1 }, "dx": 5, "dy": 5 });

    assert!(punto_pedido(&args, Idioma::Es).is_err());
}
