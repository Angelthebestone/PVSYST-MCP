use super::*;
use crate::error::Error;

fn solicitud(antes: &[Hwnd]) -> SolicitudEspera<'_> {
    SolicitudEspera {
        antes,
        esperada: None,
        limite: Duration::ZERO,
    }
}

#[test]
fn should_report_enumeration_error_instead_of_no_new_window() {
    let res = esperar_nuevas(solicitud(&[]), || {
        Err(Error::Gui("EnumWindows falló".into()))
    });

    assert!(matches!(res, Err(Error::Gui(m)) if m.contains("EnumWindows")));
}

#[test]
fn should_report_new_window_as_arrived_when_one_appears() {
    let visto = esperar_nuevas(solicitud(&[1, 2]), || Ok(vec![1, 2, 3])).expect("visto");

    assert_eq!(visto.nuevas, [3]);
    assert!(visto.llego);
}

#[test]
fn should_name_expected_class_when_it_neither_arrived_nor_was_open() {
    let visto = Resultado::vacio();

    assert_eq!(
        visto.clase_ausente(Some("TProjectForm")),
        Some("TProjectForm")
    );
    assert_eq!(visto.clase_ausente(None), None);
}

#[test]
fn should_not_flag_expected_class_when_it_was_already_open() {
    let visto = Resultado {
        nuevas: Vec::new(),
        ya_abiertas: vec![7],
        llego: false,
    };

    assert_eq!(visto.clase_ausente(Some("TProjectForm")), None);
}
