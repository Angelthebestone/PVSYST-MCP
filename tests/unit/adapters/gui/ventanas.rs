use super::*;

fn candidato(h: Hwnd, titulo: &str, clase: &str, es_hijo: bool) -> Candidato {
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
        candidato(1, "Sombreados cercanos", "TShadingsForm", false),
        candidato(2, "Advertencia", "TMessageForm", false),
        candidato(3, "Cancelar", "TButton", true),
        candidato(4, "No", "TButton", true),
    ];

    assert_eq!(elegir(&candidatos, "No"), Some(4));
}

#[test]
fn should_still_resolve_a_part_of_a_title_when_nothing_matches_whole() {
    let candidatos = [candidato(1, "Proyecto: New.PRJ", "TProjectForm", false)];

    assert_eq!(elegir(&candidatos, "new.prj"), Some(1));
    assert_eq!(elegir(&candidatos, "otra"), None);
}

#[test]
fn should_fall_back_to_a_hidden_window_only_when_no_visible_one_matches() {
    let mut oculta = candidato(1, "TDEMOForm", "TDEMOForm", false);
    oculta.visible = false;
    let visible = candidato(2, "Otra TDEMOForm", "TOtra", false);

    assert_eq!(elegir(&[oculta, visible], "demoform"), Some(2));
}
