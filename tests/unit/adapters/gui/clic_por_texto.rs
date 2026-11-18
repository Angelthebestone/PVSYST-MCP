use super::*;
use crate::adapters::gui::ventana_falsa::{Hijo, VentanaFalsa};
use crate::adapters::gui::ventanas::rect_of;
use crate::idioma::Idioma;
use crate::vista_agente::contrato::{construir, Contexto};

/// BUG-03: `coincidencias:0` used to carry `vista.estado:"ok"`.
#[test]
fn should_report_a_click_without_ocr_match_as_partial() {
    let mut resultado = json!({ "accion": "clic_texto", "texto": "NoExiste", "coincidencias": 0 });
    marcar_sin_ocr(&mut resultado, Origen::Uia, Idioma::Es);
    let args = json!({});

    let vista = construir(
        &Contexto {
            tool: "gui_actuar",
            accion: "clic_texto",
            args: &args,
            idioma: Idioma::Es,
        },
        &resultado,
    );

    let error = vista.error.expect("an unconfirmed click is not a plain ok");
    assert_eq!(error.codigo, "fallo_parcial");
    assert!(error.motivo.contains("NoExiste"), "{}", error.motivo);
}

/// GUI-09: `indice:5` with one match clicked that match through UIA.
#[test]
fn should_not_click_through_uia_when_the_index_is_out_of_range() {
    use crate::adapters::gui::ffi::Rect;
    let una_caja = [Rect {
        left: 0,
        top: 0,
        right: 10,
        bottom: 10,
    }];

    let rechazo = elegir_caja(&una_caja, Some(5), "Herramientas").expect_err("index 5 of 1");
    let sin_nada = elegir_caja(&[], None, "Herramientas").expect_err("not drawn");

    assert!(!recurre_a_uia(&rechazo));
    assert!(recurre_a_uia(&sin_nada));
}

#[test]
fn should_leave_a_click_with_an_ocr_match_as_it_was() {
    let mut resultado = json!({ "accion": "clic_texto", "texto": "Cerrar", "coincidencias": 1 });

    marcar_sin_ocr(&mut resultado, Origen::Ocr, Idioma::Es);

    assert!(resultado.get("error").is_none());
}

/// `TChoixObjetPVForm` names «Abrir» a combo box and a button: the click must
/// press the button, not whichever of the two it met first.
fn form_con_combo_y_boton_abrir() -> VentanaFalsa {
    VentanaFalsa::crear(
        "TChoixObjetPVForm",
        "Eligiendo un sitio geográfico",
        vec![
            Hijo::de("TComboBox", (20, 40, 180, 64)).con_texto("Abrir"),
            Hijo::de("TButton", (200, 40, 280, 64)).con_texto("Abrir"),
        ],
    )
}

/// Screen centre of child `indice` of a fake form (0 is the form).
fn centro(ventana: &VentanaFalsa, indice: usize) -> (i32, i32) {
    let r = rect_of(ventana.hwnd(indice));
    ((r.left + r.right) / 2, (r.top + r.bottom) / 2)
}

#[test]
fn should_click_the_button_when_a_combo_shows_the_same_caption() {
    let form = form_con_combo_y_boton_abrir();

    let pedido = Pedido {
        texto: "Abrir",
        indice: None,
    };

    let objetivo =
        destino_de_clic(form.hwnd(0), pedido, Idioma::Es).expect("the button carries the caption");

    assert_eq!(objetivo.origen, Origen::Control);
    assert_eq!(objetivo.punto, centro(&form, 2));
}

/// Two buttons of `TSystemegridForm` are captioned «Abrir» (measured): pressing
/// one of them would be a guess, so the call names both instead.
#[test]
fn should_name_every_button_that_shares_the_caption_instead_of_guessing() {
    let form = VentanaFalsa::crear(
        "TSystemegridForm",
        "Sistema",
        vec![
            Hijo::de("TButton", (20, 40, 100, 64)).con_texto("Abrir"),
            Hijo::de("TBitBtn", (200, 40, 280, 64)).con_texto("&Abrir"),
        ],
    );

    let pedido = Pedido {
        texto: "Abrir",
        indice: None,
    };

    let error = destino_de_clic(form.hwnd(0), pedido, Idioma::Es)
        .expect_err("two buttons carry the caption");

    let Error::Gui(motivo) = error else {
        panic!("a caption shared by two buttons is not a bad argument: {error:?}");
    };
    assert!(
        motivo.contains("TButton") && motivo.contains("TBitBtn"),
        "{motivo}"
    );
    for indice in [1, 2] {
        let (x, y) = centro(&form, indice);
        assert!(motivo.contains(&format!("({x}, {y})")), "{motivo}");
    }
}

/// 4.1-4: a click that changed nothing used to answer as a success.
#[test]
fn should_warn_that_a_click_without_effect_is_not_verified() {
    let aviso = sin_efecto(&Efecto::default(), Idioma::Es).expect("no effect");

    assert!(aviso.contains("verificado:false"), "{aviso}");
}

#[test]
fn should_not_warn_when_the_click_opened_a_window() {
    let efecto = Efecto {
        abiertas: vec!["TToolsForm".into()],
        ..Efecto::default()
    };

    assert!(sin_efecto(&efecto, Idioma::Es).is_none());
}
