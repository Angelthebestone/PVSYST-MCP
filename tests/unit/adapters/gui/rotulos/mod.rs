//! The captions of PVsyst's windows in each interface language, and how a window
//! betrays the language it is in.

use super::dialogos::{CANCELAR, OK};
use super::pvsyst::GESTION_ENERGIA;
use super::{IdiomaGui, Rotulo};
use crate::adapters::gui::conduccion::Formulario;
use crate::adapters::gui::ventana_falsa::{Hijo, VentanaFalsa};

/// The same project window with its buttons written in `idioma`: only the texts differ.
fn ventana_de_proyecto(idioma: IdiomaGui) -> VentanaFalsa {
    let (sistema, energia, cancelar) = match idioma {
        IdiomaGui::Es => ("Sistema", "Gestión de la energía", "Cancelar"),
        IdiomaGui::En => ("System", "Energy management", "Cancel"),
        IdiomaGui::Fr => ("Système", "Gestion de l'énergie", "Annuler"),
        IdiomaGui::De => ("System", "Energiemanagement", "Abbrechen"),
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
    for idioma in IdiomaGui::TODOS {
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
    let ventana = ventana_de_proyecto(IdiomaGui::En);
    let formulario = Formulario::de(ventana.hwnd(0));

    assert!(formulario.hijo("System").is_ok());
    let error = formulario
        .hijo(AUSENTE)
        .expect_err("a caption that is not on the form")
        .to_string();
    assert!(error.contains("Ausente/Missing"), "{error}");
}

const AUSENTE: Rotulo<'static> = Rotulo::nuevo("Ausente", ("Missing", "Absent", "Fehlt"));

#[test]
fn should_know_which_language_a_window_is_in_from_its_captions() {
    let textos = |idioma: IdiomaGui| match idioma {
        IdiomaGui::Es => vec!["OK", "Cancelar", "Guardar", "Sistema"],
        IdiomaGui::En => vec!["OK", "Cancel", "Save", "Horizon"],
        IdiomaGui::Fr => vec!["OK", "Annuler", "Enregistrer", "Système"],
        IdiomaGui::De => vec!["OK", "Abbrechen", "Speichern", "Horizont"],
    };
    for idioma in IdiomaGui::TODOS {
        assert_eq!(IdiomaGui::por_textos(textos(idioma)), Some(idioma));
    }
}

#[test]
fn should_not_guess_a_language_from_captions_that_prove_nothing() {
    // "OK" reads the same in all four, and "Sistema" belongs to Spanish while
    // "Cancel" belongs to English: a tie is not an answer.
    assert_eq!(IdiomaGui::por_textos(["OK", "texto libre"]), None);
    assert_eq!(IdiomaGui::por_textos(["Sistema", "Cancel"]), None);
    assert_eq!(IdiomaGui::por_textos([]), None);
}

#[test]
fn should_read_the_language_name_pvsyst_writes_in_its_ini() {
    assert_eq!(IdiomaGui::de_nombre("Spanish"), Some(IdiomaGui::Es));
    assert_eq!(IdiomaGui::de_nombre(" english "), Some(IdiomaGui::En));
    assert_eq!(IdiomaGui::de_nombre("French"), Some(IdiomaGui::Fr));
    assert_eq!(IdiomaGui::de_nombre("German"), Some(IdiomaGui::De));
    // Italian is a PVsyst language the captions do not cover.
    assert_eq!(IdiomaGui::de_nombre("Italian"), None);
}
