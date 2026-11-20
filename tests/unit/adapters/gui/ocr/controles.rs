use super::*;
use crate::adapters::gui::inventario::tipos::Fuente;
use crate::adapters::gui::ocr::dominio::PalabraOcr;

const fn caja([left, top, right, bottom]: [i32; 4]) -> Rect {
    Rect {
        left,
        top,
        right,
        bottom,
    }
}

fn boton(texto: &str) -> Control {
    Control {
        clase: "TButton".into(),
        hwnd: "0x1".into(),
        rect: caja([100, 100, 180, 130]),
        texto: texto.into(),
        visible: true,
        enabled: true,
        parent: None,
        pestana: None,
        indice_pestana: None,
        rol: Rol::Boton,
        indice_clase: 0,
        etiqueta: None,
        etiqueta_fuente: None,
        fuente: Fuente::Win32,
        patrones: Vec::new(),
        marcado: None,
        campo: None,
    }
}

fn linea_dibujada(texto: &str) -> LineaOcr {
    LineaOcr {
        texto: texto.into(),
        rect: caja([110, 105, 170, 125]),
        palabras: vec![PalabraOcr {
            texto: texto.into(),
            rect: caja([110, 105, 170, 125]),
        }],
    }
}

/// The harness presses a button by its text only when the text is the button's own
/// caption, so the form read has to say where each text came from.
#[test]
fn should_say_where_the_text_of_a_button_was_read() {
    let con_titulo = procesar_boton(&boton("Abrir"), &[]);
    assert_eq!(con_titulo["texto"], "Abrir");
    assert_eq!(con_titulo["etiqueta_fuente"], "window_text");

    let dibujado = procesar_boton(&boton(""), &[linea_dibujada("Tabla")]);
    assert_eq!(dibujado["texto"], "Tabla");
    assert_eq!(dibujado["etiqueta_fuente"], "ocr");

    let sin_texto = procesar_boton(&boton(""), &[]);
    assert_eq!(sin_texto["texto"], "");
    assert!(sin_texto["etiqueta_fuente"].is_null(), "{sin_texto}");
}
