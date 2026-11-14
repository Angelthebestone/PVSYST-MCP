//! The effect of an action, read from two instants.

use super::{Efecto, Instantanea};
use crate::adapters::gui::captura::ImagenRgb;

fn imagen(valor: u8) -> ImagenRgb {
    ImagenRgb {
        rgb: vec![valor; 12],
        ancho: 2,
        alto: 2,
    }
}

fn instante(ventanas: &[(isize, &str)], pixel: u8) -> Instantanea {
    Instantanea {
        pid: 1,
        h: 1,
        ventanas: ventanas.iter().map(|(h, c)| (*h, (*c).into())).collect(),
        imagen: Some(imagen(pixel)),
    }
}

#[test]
fn should_report_no_effect_when_nothing_opened_closed_or_repainted() {
    let antes = instante(&[(1, "TSystemeAddForm")], 7);
    let despues = instante(&[(1, "TSystemeAddForm")], 7);

    let efecto = Efecto::entre(&antes, &despues);

    assert!(!efecto.hubo(), "{efecto:?}");
}

#[test]
fn should_name_the_window_a_click_opened() {
    let antes = instante(&[(1, "TProjectForm")], 7);
    let despues = instante(&[(1, "TProjectForm"), (2, "TToolsForm")], 7);

    let efecto = Efecto::entre(&antes, &despues);

    assert_eq!(efecto.abiertas, vec!["TToolsForm"]);
    assert!(efecto.cerradas.is_empty());
    assert!(efecto.hubo());
}

#[test]
fn should_name_the_window_a_click_closed() {
    let antes = instante(&[(1, "TProjectForm"), (2, "TMessageForm")], 7);
    let despues = instante(&[(1, "TProjectForm")], 7);

    let efecto = Efecto::entre(&antes, &despues);

    assert_eq!(efecto.cerradas, vec!["TMessageForm"]);
    assert!(efecto.hubo());
}

#[test]
fn should_count_a_repaint_of_the_window_as_an_effect() {
    let antes = instante(&[(1, "TSystemeAddForm")], 7);
    let despues = instante(&[(1, "TSystemeAddForm")], 9);

    let efecto = Efecto::entre(&antes, &despues);

    assert!(efecto.pantalla_cambio);
    assert!(efecto.hubo());
}

/// A capture that failed cannot prove a change: it must not invent one.
#[test]
fn should_not_claim_a_repaint_when_a_capture_is_missing() {
    let antes = instante(&[(1, "TSystemeAddForm")], 7);
    let mut despues = instante(&[(1, "TSystemeAddForm")], 9);
    despues.imagen = None;

    assert!(!Efecto::entre(&antes, &despues).hubo());
}

/// Without a capture to compare, "nothing changed" is not proved: the answer is
/// "not checked", never `verificado:false`.
#[test]
fn should_not_say_nothing_changed_when_a_capture_is_missing() {
    let antes = instante(&[(1, "TSystemeAddForm")], 7);
    let mut despues = instante(&[(1, "TSystemeAddForm")], 7);
    despues.imagen = None;
    assert_eq!(Efecto::entre(&antes, &despues).verificado(), None);

    let completo = instante(&[(1, "TSystemeAddForm")], 7);
    assert_eq!(Efecto::entre(&antes, &completo).verificado(), Some(false));

    let abierta = instante(&[(1, "TSystemeAddForm"), (2, "TToolsForm")], 7);
    assert_eq!(Efecto::entre(&antes, &abierta).verificado(), Some(true));
}
