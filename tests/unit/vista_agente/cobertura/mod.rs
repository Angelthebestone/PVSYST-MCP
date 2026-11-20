use super::basica::{campos_con_clave, clave_de};
use crate::vista_agente::campos_vivos::CampoVivo;
use crate::vista_agente::dominio::Ancla;
use serde_json::{json, Value};
use std::collections::HashSet;

fn vivo(clase: &str, x: i32, y: i32) -> CampoVivo {
    CampoVivo {
        hwnd: 1,
        clase: clase.into(),
        texto: String::new(),
        habilitado: true,
        posicion: Ancla { x, y },
    }
}

#[test]
fn should_keep_keys_unique_when_two_controls_share_a_spot() {
    let claves: HashSet<String> = campos_con_clave(vec![vivo("TEdit", 5, 5), vivo("TEdit", 5, 5)])
        .into_iter()
        .map(|(clave, _)| clave)
        .collect();
    assert_eq!(claves.len(), 2);
    assert!(claves.contains(&clave_de(&vivo("TEdit", 5, 5))));
}

/// One page of the map: one `TEdit` per label and one button per caption, so the page holds as
/// many distinct controls as it lists.
fn pagina(pestana: &str, campos: &[&str], botones: &[&str]) -> Value {
    let campos: Vec<Value> = campos
        .iter()
        .map(|etiqueta| json!({ "clase": "TEdit", "etiqueta": etiqueta }))
        .collect();
    let botones: Vec<Value> = botones
        .iter()
        .map(|texto| json!({ "texto": texto }))
        .collect();
    json!({ "pestana": pestana, "campos": campos, "botones": botones })
}

/// The same page under a nested sheet, the way `TPVModuleForm` records «Parámetros del modelo».
fn con_subpestana(mut pagina: Value, subpestana: &str) -> Value {
    pagina["subpestana"] = json!(subpestana);
    pagina
}

/// A page whose fields carry no label, which is how the map records a field whose text it could
/// not read.
fn pagina_sin_etiquetas(pestana: &str, cuantos: usize) -> Value {
    json!({
        "pestana": pestana,
        "campos": vec![json!({ "clase": "TEdit", "etiqueta": null }); cuantos],
        "botones": [],
    })
}

fn ventana(paginas: &[Value]) -> Value {
    json!({ "instancias": [{ "paginas": paginas }] })
}

/// `TPVModuleForm` records the whole form's buttons (`Imprimir`, `OK`, `Cancelar`) in every
/// page of every tab, and «Parámetros del modelo» has one page per sub-tab: the union of the
/// tab counts those buttons once, and only the labels a sub-tab does not share add up.
#[test]
fn should_count_a_button_shared_by_every_sub_tab_of_a_tab_once() {
    let botones = ["Imprimir", "OK", "Cancelar"];
    let paginas = [
        con_subpestana(
            pagina("Parámetros del modelo", &["Área", "Temperatura"], &botones),
            "Degradación",
        ),
        con_subpestana(
            pagina("Parámetros del modelo", &["Área", "Bifacial"], &botones),
            "IAM personalizado",
        ),
        con_subpestana(
            pagina("Parámetros del modelo", &["Coeficiente"], &botones),
            "Datos adicionales",
        ),
    ];

    let resumen = super::mapa::resumir(&ventana(&paginas));

    assert_eq!((resumen.campos, resumen.botones), (4, 3), "Área se repite");
}

/// A tab PVsyst draws with nested sheets keeps fields on its own page, outside the sub-sheet
/// control: those join the union, and only what the sub-sheets repeat counts once.
#[test]
fn should_add_the_fields_of_a_page_recorded_without_a_sub_tab() {
    let botones = ["OK", "Cancelar"];
    let paginas = [
        pagina("Parámetros del modelo", &["Modelo", "Tecnología"], &botones),
        con_subpestana(
            pagina("Parámetros del modelo", &["Modelo", "Área"], &botones),
            "Degradación",
        ),
        con_subpestana(
            pagina("Parámetros del modelo", &["Bifacial"], &botones),
            "IAM personalizado",
        ),
    ];

    let resumen = super::mapa::resumir(&ventana(&paginas));

    assert_eq!(
        (resumen.campos, resumen.botones),
        (4, 2),
        "Modelo y los botones del formulario se repiten"
    );
}

/// A field the map has no label for is counted by its position in its page: two of them on one
/// page are two, and the field at the same position of two pages of one tab is the same one,
/// because nothing else tells them apart.
#[test]
fn should_count_an_unlabelled_field_by_its_position_in_its_page() {
    let uno = pagina_sin_etiquetas("Datos básicos", 1);
    let dos = pagina_sin_etiquetas("Datos básicos", 2);

    let dos_paginas = super::mapa::resumir(&ventana(&[uno, dos.clone()]));
    let una_pagina = super::mapa::resumir(&ventana(&[dos]));

    assert_eq!(
        dos_paginas.campos, 2,
        "la posición 0 se repite entre páginas"
    );
    assert_eq!(
        una_pagina.campos, 2,
        "dos sin etiqueta en una página son dos"
    );
}

/// Two different fields of one tab that share class and label (`TEdit` «Valor» on two pages)
/// collapse into one: the union falls short, which is the safe error for a coverage figure.
#[test]
fn should_count_two_fields_that_share_class_and_label_once() {
    let paginas = [
        pagina("Datos básicos", &["Valor", "Unidades"], &[]),
        pagina("Datos básicos", &["Valor", "Rango"], &[]),
    ];

    let resumen = super::mapa::resumir(&ventana(&paginas));

    assert_eq!(resumen.campos, 3);
}

/// With no sub-tab in the window, each tab is counted on its own: the same caption on two tabs is
/// two buttons, and a page that repeats a label of its tab adds nothing.
#[test]
fn should_count_each_tab_on_its_own_when_no_page_has_a_sub_tab() {
    let paginas = [
        pagina(
            "Datos básicos",
            &["Fabricante", "Modelo"],
            &["OK", "Cancelar"],
        ),
        pagina("Datos básicos", &["Modelo"], &["OK", "Cancelar"]),
        pagina("Comercial", &["Precio"], &["OK", "Cancelar"]),
    ];

    let resumen = super::mapa::resumir(&ventana(&paginas));

    assert_eq!((resumen.campos, resumen.botones), (3, 4));
}
