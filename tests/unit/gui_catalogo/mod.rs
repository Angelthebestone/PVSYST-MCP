use super::dominio::{buscar, Acceso};
use super::entradas::funciones;
use std::collections::HashSet;

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
fn should_carry_no_user_paths_or_project_names_when_the_window_map_is_published() {
    let mapa = super::recurso::como_recurso()["mapa"].to_string();

    for patron_prohibido in ["C:\\\\Users", "\\\\Users\\\\", "/Users/", "/home/", "MCP_"] {
        assert!(
            !mapa.contains(patron_prohibido),
            "el mapa contiene el patron prohibido: {patron_prohibido}"
        );
    }
    assert!(
        !mapa.contains("C:\\\\"),
        "el mapa contiene una ruta de disco"
    );
}
