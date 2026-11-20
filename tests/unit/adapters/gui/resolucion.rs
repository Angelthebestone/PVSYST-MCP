use super::*;

fn candidato(h: Hwnd, [titulo, clase]: [&str; 2], es_hijo: bool) -> Candidato {
    Candidato {
        h,
        titulo: titulo.into(),
        clase: clase.into(),
        visible: true,
        es_hijo,
    }
}

/// GUI-08: «No» resolved to the form whose title contains «cerca**no**s».
#[test]
fn should_prefer_the_button_named_no_over_a_form_whose_title_contains_it() {
    let candidatos = [
        candidato(1, ["Sombreados cercanos", "TShadingsForm"], false),
        candidato(2, ["Advertencia", "TMessageForm"], false),
        candidato(3, ["Cancelar", "TButton"], true),
        candidato(4, ["No", "TButton"], true),
    ];

    assert_eq!(elegir(&candidatos, "No"), Some(4));
}

#[test]
fn should_still_resolve_a_part_of_a_title_when_nothing_matches_whole() {
    let candidatos = [candidato(1, ["Proyecto: New.PRJ", "TProjectForm"], false)];

    assert_eq!(elegir(&candidatos, "new.prj"), Some(1));
    assert_eq!(elegir(&candidatos, "otra"), None);
}

#[test]
fn should_fall_back_to_a_hidden_window_only_when_no_visible_one_matches() {
    let mut oculta = candidato(1, ["TDEMOForm", "TDEMOForm"], false);
    oculta.visible = false;
    let visible = candidato(2, ["Otra TDEMOForm", "TOtra"], false);

    assert_eq!(elegir(&[oculta, visible], "demoform"), Some(2));
}

/// 4.1-7: `ventana:"OK"` pressed the OK of whichever window was listed first.
#[test]
fn should_report_the_controls_that_tie_for_a_name() {
    let candidatos = [
        candidato(1, ["Proyecto", "TProjectForm"], false),
        candidato(2, ["OK", "TButton"], true),
        candidato(3, ["OK", "TButton"], true),
    ];

    assert_eq!(controles_empatados(&candidatos, "OK"), vec![2, 3]);
}

#[test]
fn should_not_call_a_tie_a_name_that_one_control_or_a_window_answers() {
    let uno = [
        candidato(1, ["Proyecto", "TProjectForm"], false),
        candidato(2, ["OK", "TButton"], true),
        candidato(3, ["Cancelar", "TButton"], true),
    ];
    assert_eq!(controles_empatados(&uno, "OK"), vec![2]);

    // A top-level window with that name wins over the controls: no tie to resolve.
    let ventana = [
        candidato(1, ["OK", "TOkForm"], false),
        candidato(2, ["OK", "TButton"], true),
        candidato(3, ["OK", "TButton"], true),
    ];
    assert!(controles_empatados(&ventana, "OK").is_empty());
}

#[test]
fn should_ignore_a_hidden_control_with_the_same_name() {
    let mut oculto = candidato(3, ["OK", "TButton"], true);
    oculto.visible = false;
    let candidatos = [candidato(2, ["OK", "TButton"], true), oculto];

    assert_eq!(controles_empatados(&candidatos, "OK"), vec![2]);
}
