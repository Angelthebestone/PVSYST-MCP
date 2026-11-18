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

/// One page of the map: only the length of `campos` and `botones` matters here.
fn pagina(pestana: &str, subpestana: Option<&str>, (campos, botones): (usize, usize)) -> Value {
    let mut pagina = json!({
        "pestana": pestana,
        "campos": vec![json!({ "etiqueta": "campo" }); campos],
        "botones": vec![json!({ "texto": "boton" }); botones],
    });
    if let Some(subpestana) = subpestana {
        pagina["subpestana"] = json!(subpestana);
    }
    pagina
}

fn ventana(paginas: &[Value]) -> Value {
    json!({ "instancias": [{ "paginas": paginas }] })
}

/// `TPVModuleForm` keeps sub-tabs inside «Parámetros del modelo», and the map used to record
/// them as a single page: each (tab, sub-tab) pair is a page of its own, so the coverage count
/// adds every sub-tab while repeats of one pair still keep only its fullest page.
#[test]
fn should_count_every_sub_tab_of_a_page_as_a_page_of_its_own() {
    for (caso, paginas, campos, botones) in [
        (
            "dos subpestañas de la misma pestaña suman",
            vec![
                pagina("Parámetros del modelo", Some("Degradación"), (3, 1)),
                pagina("Parámetros del modelo", Some("IAM personalizado"), (2, 2)),
            ],
            5,
            3,
        ),
        (
            "la misma subpestaña cuenta una vez, la más llena",
            vec![
                pagina("Parámetros del modelo", Some("Degradación"), (5, 1)),
                pagina("Parámetros del modelo", Some("Degradación"), (2, 2)),
            ],
            5,
            1,
        ),
        (
            "sin subpestaña sigue contando una página por pestaña",
            vec![
                pagina("Datos básicos", None, (4, 1)),
                pagina("Datos básicos", None, (2, 2)),
            ],
            4,
            1,
        ),
        (
            "pestañas, subpestañas y páginas sin subpestaña suman",
            vec![
                pagina("Datos básicos", None, (4, 1)),
                pagina("Parámetros del modelo", Some("Degradación"), (3, 1)),
                pagina("Parámetros del modelo", Some("IAM personalizado"), (2, 2)),
            ],
            9,
            4,
        ),
    ] {
        let resumen = super::mapa::resumir(&ventana(&paginas));
        assert_eq!(
            (resumen.campos, resumen.botones),
            (campos, botones),
            "{caso}"
        );
    }
}
