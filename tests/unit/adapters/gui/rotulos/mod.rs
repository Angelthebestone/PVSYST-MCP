//! The captions of PVsyst's windows in each interface language, and how a window
//! betrays the language it is in.

use super::dialogos::{CANCELAR, OK};
use super::pvsyst::GESTION_ENERGIA;
use super::{Idioma, Rotulo};
use crate::adapters::gui::conduccion::Formulario;
use crate::adapters::gui::ventana_falsa::{Hijo, VentanaFalsa};

/// The same project window with its buttons written in `idioma`: only the texts differ.
fn ventana_de_proyecto(idioma: Idioma) -> VentanaFalsa {
    let (sistema, energia, cancelar) = match idioma {
        Idioma::Es => ("Sistema", "Gestión de la energía", "Cancelar"),
        Idioma::En => ("System", "Energy management", "Cancel"),
        Idioma::Fr => ("Système", "Gestion de l'énergie", "Annuler"),
        Idioma::De => ("System", "Energiemanagement", "Abbrechen"),
    };
    VentanaFalsa::crear(
        "TProjectForm",
        "Proyecto",
        vec![
            Hijo::de("TPngBitBtn", (10, 10, 150, 40)).con_texto(sistema),
            Hijo::de("TPngBitBtn", (10, 50, 150, 80)).con_texto(energia),
            Hijo::de("TPngBitBtn", (10, 90, 150, 120)).con_texto(cancelar),
            Hijo::de("TPngBitBtn", (10, 130, 150, 160)).con_texto("OK"),
        ],
    )
}

#[test]
fn should_find_the_same_button_whatever_language_pvsyst_is_in() {
    for idioma in Idioma::TODOS {
        let ventana = ventana_de_proyecto(idioma);
        let formulario = Formulario::de(ventana.hwnd(0));

        let energia = formulario.hijo(GESTION_ENERGIA);
        let cancelar = formulario.hijo(CANCELAR);

        assert_eq!(energia.ok(), Some(ventana.hwnd(2)), "{idioma:?}");
        assert_eq!(cancelar.ok(), Some(ventana.hwnd(3)), "{idioma:?}");
        assert!(formulario.habilitado(OK), "{idioma:?}");
    }
}

#[test]
fn should_still_find_a_button_by_a_plain_caption_and_name_what_it_looked_for() {
    let ventana = ventana_de_proyecto(Idioma::En);
    let formulario = Formulario::de(ventana.hwnd(0));

    assert!(formulario.hijo("System").is_ok());
    let error = formulario
        .hijo(AUSENTE)
        .expect_err("a caption that is not on the form")
        .to_string();
    assert!(error.contains("Ausente/Missing"), "{error}");
}

const AUSENTE: Rotulo<'static> = Rotulo::nuevo("Ausente", ("Missing", "Absent", "Fehlt"));
