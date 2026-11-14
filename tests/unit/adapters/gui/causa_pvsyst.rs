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
