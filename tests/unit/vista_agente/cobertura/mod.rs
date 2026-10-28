use super::basica::{campos_con_clave, clave_de};
use super::dominio::Cobertura;
use super::listado::listar_ventanas;
use super::mapa::{clases_del_mapa, resumir, ResumenMapa};
use super::ventanas::{de_clase, todas};
use crate::vista_agente::campos_vivos::CampoVivo;
use crate::vista_agente::dominio::Ancla;
use crate::vista_agente::esquemas;
use std::collections::{BTreeSet, HashSet};

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
fn should_register_every_window_class_of_the_map_in_exactly_one_state() {
    let del_mapa: BTreeSet<String> = clases_del_mapa().keys().cloned().collect();
    let registradas: Vec<&str> = todas().map(|v| v.clase).collect();
    let unicas: BTreeSet<String> = registradas.iter().map(|c| (*c).to_string()).collect();
    assert_eq!(
        registradas.len(),
        unicas.len(),
        "una clase registrada dos veces"
    );
    let sin_clasificar: Vec<_> = del_mapa.difference(&unicas).collect();
    let sobrantes: Vec<_> = unicas.difference(&del_mapa).collect();
    assert!(
        sin_clasificar.is_empty(),
        "clases del mapa sin clasificar: {sin_clasificar:?}"
    );
    assert!(
        sobrantes.is_empty(),
        "clases registradas que no están en el mapa: {sobrantes:?}"
    );
    assert_eq!(del_mapa.len(), 82);
}

#[test]
fn should_explain_every_window_that_is_not_a_field_entity() {
    let sin_motivo: Vec<_> = todas()
        .filter(|v| matches!(v.cobertura, Cobertura::SoloAcciones | Cobertura::NoAplica))
        .filter(|v| v.motivo.trim().is_empty())
        .map(|v| v.clase)
        .collect();
    assert!(sin_motivo.is_empty(), "sin motivo escrito: {sin_motivo:?}");
}

#[test]
fn should_register_a_field_entity_only_for_windows_that_show_fields_in_the_map() {
    let mapa = clases_del_mapa();
    let sin_campos: Vec<_> = todas()
        .filter(|v| v.cobertura == Cobertura::Basica)
        .filter(|v| mapa.get(v.clase).is_none_or(|m| m.campos == 0))
        .map(|v| v.clase)
        .collect();
    assert!(
        sin_campos.is_empty(),
        "entidad básica sin campos en el mapa: {sin_campos:?}"
    );
}

#[test]
fn should_register_an_actions_only_window_only_when_the_map_shows_buttons() {
    let mapa = clases_del_mapa();
    let sin_botones: Vec<_> = todas()
        .filter(|v| v.cobertura == Cobertura::SoloAcciones)
        .filter(|v| mapa.get(v.clase).is_none_or(|m| m.botones == 0))
        .map(|v| v.clase)
        .collect();
    assert!(
        sin_botones.is_empty(),
        "solo acciones sin botones en el mapa: {sin_botones:?}"
    );
}

#[test]
fn should_link_every_schema_entity_to_its_registered_window() {
    for entidad in esquemas::todas() {
        let ventana = de_clase(entidad.formulario).expect("window of the schema is registered");
        assert_eq!(
            ventana.esquema,
            Some(entidad.clave),
            "{}",
            entidad.formulario
        );
    }
}

#[test]
fn should_list_every_window_and_count_the_states_without_losing_any() {
    let listado = listar_ventanas();
    assert_eq!(listado["total"], 82);
    let suma: u64 = listado["por_estado"]
        .as_object()
        .expect("states")
        .values()
        .filter_map(serde_json::Value::as_u64)
        .sum();
    assert_eq!(suma, 82);
    let panel = listado["ventanas"]
        .as_array()
        .expect("windows")
        .iter()
        .find(|v| v["clase"] == "TPVModuleForm")
        .expect("module form listed");
    assert_eq!(panel["campos_mapeados"], 16);
    assert!(panel["campos_en_mapa"].as_u64().unwrap_or(0) > 0, "{panel}");
}

#[test]
fn should_key_basic_fields_by_class_and_position_in_reading_order() {
    let claves: Vec<String> = campos_con_clave(vec![
        vivo("TEdit", 200, 40),
        vivo("TCheckBox", 10, 40),
        vivo("TComboBox", 10, 10),
    ])
    .into_iter()
    .map(|(clave, _)| clave)
    .collect();
    assert_eq!(claves, ["combobox_10_10", "checkbox_40_10", "edit_40_200"]);
}

#[test]
fn should_keep_basic_keys_unique_when_two_controls_share_a_spot() {
    let claves: HashSet<String> = campos_con_clave(vec![vivo("TEdit", 5, 5), vivo("TEdit", 5, 5)])
        .into_iter()
        .map(|(clave, _)| clave)
        .collect();
    assert_eq!(claves.len(), 2);
    assert!(claves.contains(&clave_de(&vivo("TEdit", 5, 5))));
}

fn ventana_listada(clase: &str) -> serde_json::Value {
    let todas = listar_ventanas();
    todas["ventanas"]
        .as_array()
        .and_then(|l| l.iter().find(|v| v["clase"] == clase).cloned())
        .unwrap_or_default()
}

#[test]
fn should_link_a_window_to_the_catalogue_function_that_opens_it() {
    let sistema = ventana_listada("TSystemegridForm");
    assert!(
        sistema["abierta_con"]
            .as_array()
            .is_some_and(|l| l.iter().any(|c| c == "variante.sistema")),
        "{sistema}"
    );
}

#[test]
fn should_list_no_function_when_the_catalogue_never_opens_the_window() {
    let sin_funcion = ventana_listada("MicrosoftWindowsTooltip");
    assert_eq!(sin_funcion["abierta_con"], serde_json::json!([]));
}

fn pagina(campos: usize, botones: usize) -> serde_json::Value {
    serde_json::json!({
        "campos": vec![serde_json::json!({}); campos],
        "botones": vec![serde_json::json!({}); botones],
    })
}

#[test]
fn should_count_fields_and_buttons_of_one_instance_when_instances_differ() {
    let ventana = serde_json::json!({ "instancias": [
        { "paginas": [pagina(10, 0)] },
        { "paginas": [pagina(0, 5)] },
    ]});
    let esperado = ResumenMapa {
        instancias: 2,
        campos: 10,
        botones: 0,
    };
    assert_eq!(resumir(&ventana), esperado);
}

#[test]
fn should_count_fields_and_buttons_of_one_page_when_a_tab_repeats_pages() {
    let ventana = serde_json::json!({ "instancias": [{ "paginas": [
        { "pestana": "A", "campos": pagina(6, 0)["campos"], "botones": [] },
        { "pestana": "A", "campos": [], "botones": pagina(0, 2)["botones"] },
    ]}]});
    let resumen = resumir(&ventana);
    assert_eq!((resumen.campos, resumen.botones), (6, 0));
}

#[test]
fn should_resolve_the_schema_every_window_names() {
    for ventana in todas().filter(|v| v.esquema.is_some()) {
        let clave = ventana.esquema.unwrap_or_default();
        assert!(
            esquemas::entidad(clave).is_ok(),
            "{} nombra el esquema {clave}, que no existe",
            ventana.clase
        );
    }
}

#[test]
fn should_read_the_fields_of_a_complete_window_like_a_basic_one() {
    let completa = super::dominio::Cobertura::Completa;
    assert!(super::basica::lee_campos_por_clase(completa));
    assert!(!super::basica::lee_campos_por_clase(
        super::dominio::Cobertura::SoloAcciones
    ));
}
