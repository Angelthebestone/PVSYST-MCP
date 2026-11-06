use super::*;

fn mensajes(textos: &[&str]) -> Vec<String> {
    textos.iter().map(|t| (*t).to_string()).collect()
}

#[test]
fn should_name_the_missing_project_instead_of_pvsyst_internal_names_when_the_project_window_opened_empty(
) {
    // Given: PVsyst logged that its project list was empty
    let registrados = mensajes(&["Empty project name detected in fPrjList"]);

    // When
    let texto = explicar(
        "PVsyst does not offer 'VC0' in that list of 'Proyecto - '",
        &registrados,
        Idioma::En,
    );

    // Then: the real cause and the next step, without the internal list name
    assert!(texto.contains("no project loaded"), "{texto}");
    assert!(texto.contains("Next: load the project first"), "{texto}");
    assert!(!texto.contains("fPrjList"), "{texto}");
}

#[test]
fn should_drop_the_benign_parse_notices_from_the_reported_messages() {
    // Given: a real failure next to the parse notices every load emits
    let registrados = mensajes(&[
        "Reading object TEnergyMixSet: unexpected EndTags",
        "PVsyst lost the foreground window (TProjectForm)",
    ]);

    // When
    let texto = explicar("keys were not sent", &registrados, Idioma::En);

    // Then: only the message that matters survives
    assert!(!texto.contains("TEnergyMixSet"), "{texto}");
    assert!(texto.contains("lost the foreground window"), "{texto}");
}

#[test]
fn should_keep_the_plain_error_when_pvsyst_logged_nothing_useful() {
    let texto = explicar("PVsyst did not open 'TSystemForm' in 15 s", &[], Idioma::En);
    assert_eq!(texto, "PVsyst did not open 'TSystemForm' in 15 s");
    assert!(es_aviso_benigno("EndTags"));
    assert!(!es_aviso_benigno("PVsyst lost the foreground window"));
}

#[test]
fn should_speak_the_requested_language() {
    let registrados = mensajes(&["Empty project name detected in fPrjList"]);
    let texto = explicar("falló", &registrados, Idioma::Es);
    assert!(texto.contains("sin ningún proyecto cargado"), "{texto}");
    assert!(texto.contains("carga el proyecto primero"), "{texto}");
}
