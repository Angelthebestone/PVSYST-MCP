use super::*;

#[test]
fn should_click_buttons_with_bm_click_only_and_other_controls_with_the_mouse() {
    for boton in ["Button", "TButton", "tbitbtn"] {
        assert_eq!(
            mecanismo_para(boton),
            MecanismoClic::BotonEstandar,
            "{boton}"
        );
    }
    for otro in ["TPanel", "TListView", "SysTreeView32"] {
        assert_eq!(mecanismo_para(otro), MecanismoClic::Raton, "{otro}");
    }
}

fn viva(h: isize, clase: &str) -> VentanaViva {
    VentanaViva {
        h,
        clase: clase.to_string(),
        titulo: String::new(),
    }
}

#[test]
fn should_report_the_parent_that_reopens_after_closing_an_editor() {
    let antes = [viva(0x10, "THomeForm"), viva(0x20, "TPVModuleForm")];
    let despues = [
        viva(0x10, "THomeForm"),
        viva(0x20, "TPVModuleForm"),
        viva(0x30, "TChoixObjetPVForm"),
    ];

    let reabiertas = formas_reabiertas(&antes, &despues, 0x20);

    assert_eq!(reabiertas.len(), 1);
    assert_eq!(reabiertas[0]["class"], "TChoixObjetPVForm");
    assert_eq!(reabiertas[0]["hwnd"], "0x30");
}

#[test]
fn should_not_report_the_home_screen_as_a_reopened_form() {
    let antes = [viva(0x20, "TPVModuleForm")];
    let despues = [viva(0x10, "THomeForm"), viva(0x20, "TPVModuleForm")];

    assert!(formas_reabiertas(&antes, &despues, 0x20).is_empty());
}

#[test]
fn should_suggest_the_next_cerrar_when_a_form_reopens() {
    let reabiertas = [json!({"hwnd": "0x30", "class": "TChoixObjetPVForm"})];

    let aviso = aviso_de_reaparicion(&reabiertas).expect("a warning");

    assert!(aviso.contains("TChoixObjetPVForm"), "{aviso}");
    assert!(aviso.contains("accion:cerrar"), "{aviso}");
    assert!(aviso.contains("0x30"), "{aviso}");
}

#[test]
fn should_stay_quiet_when_nothing_reopens() {
    assert_eq!(aviso_de_reaparicion(&[]), None);
}
