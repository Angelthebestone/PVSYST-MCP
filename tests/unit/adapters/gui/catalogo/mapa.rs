use serde_json::Value;

/// A page that records a sub-tab labels it with text: the coverage counter keys the
/// page by that label, and a blank or non-text one would fold it into its tab.
fn exigir_subpestana_con_texto(clase: &str, pagina: &Value) {
    let Some(subpestana) = pagina.get("subpestana") else {
        return;
    };
    assert!(
        subpestana.as_str().is_some_and(|s| !s.trim().is_empty()),
        "{clase}: subpestana debe ser un texto no vacío, no {subpestana}"
    );
}

/// What is stable across PVsyst languages is recorded as structure: every page of
/// the map says the position of its tab, so a caller never needs the tab's name, and a
/// page that records a sub-tab labels it with text, which is what the coverage counter
/// keys it by.
#[test]
fn should_record_the_tab_index_of_every_page_of_the_window_map() {
    let mapa = super::recurso::mapa_de_ventanas();
    let instancias = mapa["ventanas"]
        .as_object()
        .unwrap()
        .iter()
        .flat_map(|(clase, v)| {
            v["instancias"]
                .as_array()
                .unwrap()
                .iter()
                .map(move |i| (clase, i))
        });
    let mut paginas = 0;

    for (clase, instancia) in instancias {
        let nombres = instancia["pestanas"].as_array().unwrap();
        for pagina in instancia["paginas"].as_array().unwrap() {
            let esperado = nombres.iter().position(|n| *n == pagina["pestana"]);
            let registrado = pagina["indice_pestana"].as_u64().map(|i| i as usize);
            assert_eq!(registrado, esperado, "{clase}: {}", pagina["pestana"]);
            exigir_subpestana_con_texto(clase, pagina);
            paginas += 1;
        }
    }
    assert!(paginas > 0);
    assert!(mapa["nota"].as_str().unwrap().contains("indice_pestana"));
}
