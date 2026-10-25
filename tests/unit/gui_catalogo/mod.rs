use super::dominio::{buscar, pedido_de_menu, Acceso, Efecto, PedidoMenu, VENTANA_INICIO};
use super::entradas::funciones;
use serde_json::json;
use std::collections::HashSet;

#[test]
fn should_have_unique_keys_when_catalogue_is_built() {
    let total = funciones().count();
    let claves: HashSet<_> = funciones().map(|f| f.clave).collect();
    assert_eq!(claves.len(), total);
}

#[test]
fn should_have_unique_command_ids_per_window_when_menu_items_are_listed() {
    let ids: Vec<_> = funciones()
        .filter_map(|f| match f.acceso {
            Acceso::Menu { id } => Some((f.ventana, id)),
            Acceso::MenuPosicion { .. } | Acceso::Boton { .. } => None,
        })
        .collect();
    let unicos: HashSet<_> = ids.iter().collect();
    assert_eq!(unicos.len(), ids.len());
}

#[test]
fn should_name_a_catalogued_parent_that_opens_the_host_when_function_requires_one() {
    for f in funciones().filter(|f| f.requiere.is_some()) {
        let padre = buscar(funciones(), f.requiere.unwrap_or_default());
        assert_eq!(padre.len(), 1, "{} requiere una clave inexistente", f.clave);
        assert_eq!(
            padre[0].abre,
            Some(f.ventana),
            "{}: el padre no abre la ventana anfitriona",
            f.clave
        );
    }
}

#[test]
fn should_require_a_parent_when_button_lives_outside_the_main_form_and_the_project_window() {
    for f in funciones().filter(|f| f.ventana != VENTANA_INICIO && f.ventana != "TProjectForm") {
        assert!(
            f.requiere.is_some(),
            "{} no sabe cómo abrir {}",
            f.clave,
            f.ventana
        );
    }
}

#[test]
fn should_list_whole_menu_when_key_is_a_prefix() {
    let archivo = buscar(funciones(), "archivo");
    assert!(archivo.len() > 1);
    assert!(archivo.iter().all(|f| f.clave.starts_with("archivo.")));
}

#[test]
fn should_not_match_a_longer_area_name_when_prefix_shares_leading_letters() {
    // "inicio" must not pull in a hypothetical "inicio_rapido.x": the dot is part of the prefix.
    let halladas = buscar(funciones(), "inicio");
    assert!(halladas.iter().all(|f| f.clave.starts_with("inicio.")));
}

#[test]
fn should_find_nothing_when_key_is_unknown() {
    assert!(buscar(funciones(), "no.existe").is_empty());
}

#[test]
fn should_flag_exit_as_closing_pvsyst_and_simulation_as_modifying_when_listing_effects() {
    assert_eq!(
        buscar(funciones(), "archivo.salida")[0].efecto,
        Efecto::Cierra
    );
    assert_eq!(
        buscar(funciones(), "variante.ejecutar_simulacion")[0].efecto,
        Efecto::Modifica
    );
}

#[test]
fn should_describe_entry_point_in_json_when_function_is_a_nested_button() {
    let boton = buscar(funciones(), "herramientas.factor_transposicion")[0].a_json();
    assert_eq!(boton["via"], "boton");
    assert_eq!(boton["ventana"], "TToolsForm");
    assert_eq!(boton["requiere"], "inicio.herramientas");
    assert_eq!(boton["abre"], "TTranspositionForm");
}

#[test]
fn should_have_unique_positions_per_window_when_menu_items_are_named_by_place() {
    let posiciones: Vec<_> = funciones()
        .filter_map(|f| match f.acceso {
            Acceso::MenuPosicion { menu, item } => Some((f.ventana, menu, item)),
            Acceso::Menu { .. } | Acceso::Boton { .. } => None,
        })
        .collect();
    let unicas: HashSet<_> = posiciones.iter().collect();
    assert_eq!(unicas.len(), posiciones.len());
}

#[test]
fn should_describe_a_positioned_menu_item_with_menu_and_item_when_listing_the_project_window() {
    let ficha = buscar(funciones(), "ventana_proyecto.proyecto.configuracion")[0].a_json();

    assert_eq!(ficha["via"], "menu_posicion");
    assert_eq!(ficha["ventana"], "TProjectForm");
    assert_eq!(
        ficha["destino"],
        serde_json::json!({ "menu": 0, "item": 8 })
    );
}

#[test]
fn should_read_menu_ids_at_run_time_when_the_project_window_hosts_a_menu_item() {
    // The project window's command ids move between instances: a fixed id there
    // would post the wrong command.
    let con_id_fijo: Vec<_> = funciones()
        .filter(|f| f.ventana == "TProjectForm" && matches!(f.acceso, Acceso::Menu { .. }))
        .map(|f| f.clave)
        .collect();

    assert!(con_id_fijo.is_empty(), "{con_id_fijo:?}");
}

#[test]
fn should_reach_every_host_window_without_cycles_when_functions_require_a_parent() {
    for f in funciones() {
        let mut visitadas = HashSet::from([f.clave]);
        let mut actual = f;
        while let Some(padre) = actual.requiere {
            assert!(
                visitadas.insert(padre),
                "{}: requiere forma un ciclo en {padre}",
                f.clave
            );
            actual = *buscar(funciones(), padre)
                .first()
                .unwrap_or_else(|| panic!("{}: requiere una clave inexistente: {padre}", f.clave));
        }
    }
}

#[test]
fn should_name_the_command_by_id_when_the_request_gives_only_an_id() {
    assert_eq!(
        pedido_de_menu(&json!({ "id": 155 })).ok(),
        Some(PedidoMenu::Id(155))
    );
}

#[test]
fn should_name_the_command_by_place_when_the_request_gives_menu_and_item() {
    let pedido = pedido_de_menu(&json!({ "menu": 2, "item": 4 })).ok();

    assert_eq!(pedido, Some(PedidoMenu::Posicion { menu: 2, item: 4 }));
}

#[test]
fn should_reject_the_request_when_it_gives_half_a_position_both_kinds_or_nothing() {
    for pedido in [
        json!({ "menu": 1 }),
        json!({ "item": 1 }),
        json!({ "id": 1, "menu": 0, "item": 0 }),
        json!({}),
    ] {
        assert!(pedido_de_menu(&pedido).is_err(), "{pedido}");
    }
}

#[test]
fn should_reject_the_request_when_a_number_is_negative_fractional_or_too_big() {
    for pedido in [
        json!({ "id": 70000 }),
        json!({ "id": 1.5 }),
        json!({ "menu": -1, "item": 0 }),
    ] {
        assert!(pedido_de_menu(&pedido).is_err(), "{pedido}");
    }
}

#[test]
fn should_map_every_window_a_function_opens_with_its_pages_and_fields_when_the_resource_is_built() {
    let recurso = super::recurso::como_recurso();
    let ventanas = &recurso["mapa"]["ventanas"];
    // Windows that are not VCL forms of PVsyst or that only show a message.
    let sin_mapa = ["#32770", "TMessageForm"];

    for f in funciones() {
        let Some(clase) = f.abre.filter(|c| !sin_mapa.contains(c)) else {
            continue;
        };
        let instancias = ventanas[clase]["instancias"].as_array();
        let paginas = instancias
            .into_iter()
            .flatten()
            .flat_map(|i| i["paginas"].as_array().into_iter().flatten())
            .count();
        assert!(
            paginas > 0,
            "{}: {clase} no está en docs/gui_mapa.json",
            f.clave
        );
    }
}
