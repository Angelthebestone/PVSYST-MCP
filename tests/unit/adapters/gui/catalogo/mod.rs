use super::dominio::{buscar, Acceso};
use super::entradas::funciones;
use super::recurso::EnEjecucion;
use crate::idioma::Idioma;
use std::collections::HashSet;
use std::path::Path;

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

/// GUI-12: `menus` listed ids and positions but no name for any entry.
#[test]
fn should_name_a_menu_entry_by_the_catalogued_function_at_its_position() {
    let (f, menu, item) = funciones()
        .find_map(|f| match f.acceso {
            Acceso::MenuPosicion { menu, item } => Some((f, usize::from(menu), usize::from(item))),
            _ => None,
        })
        .expect("the catalogue has an entry by position");

    let nombre = super::funcion_de_menu(f.ventana, (menu, item), None);

    assert_eq!(nombre, Some((f.clave, f.etiqueta)));
    assert_eq!(
        super::funcion_de_menu("TNoExisteForm", (menu, item), None),
        None
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
fn should_carry_no_user_paths_or_project_names_when_the_window_map_is_published() {
    let mapa = super::recurso::como_recurso(&vivo("", None), Idioma::default())["mapa"].to_string();

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

fn vivo<'a>(gui_exe: &'a str, idioma_gui: Option<&'a str>) -> EnEjecucion<'a> {
    EnEjecucion {
        gui_exe: Path::new(gui_exe),
        idioma_gui,
    }
}

/// The map is a snapshot: it says when and on which version it was measured, how
/// much it covers, and warns when the PVsyst the server drives is another one.
#[test]
fn should_describe_the_map_and_warn_when_the_running_version_differs() {
    let mapa = super::recurso::mapa_de_ventanas();
    let medida = super::recurso::como_recurso(
        &vivo(r"C:\Program Files\PVsyst8.1.8\PVsyst.exe", None),
        Idioma::En,
    );
    let otra = super::recurso::como_recurso(
        &vivo(r"C:\Program Files\PVsyst8.2.0\PVsyst.exe", None),
        Idioma::En,
    );

    let meta = &medida["metadatos"];
    assert_eq!(
        meta["windows_total"],
        mapa["ventanas"].as_object().unwrap().len()
    );
    assert!(meta["measured_on"].as_str().is_some_and(|f| f.len() == 10));
    assert_eq!(meta["pvsyst_version"], "8.1.8");
    assert!(meta["cobertura"]["ventanas_con_campos"].as_u64() > Some(0));
    assert_eq!(meta["aviso"], serde_json::Value::Null);
    let aviso = otra["metadatos"]["aviso"].as_str().unwrap();
    assert!(
        aviso.contains("8.2.0") && aviso.contains("8.1.8"),
        "{aviso}"
    );
}

/// The map's texts are in the language it was measured in: a PVsyst running in
/// another one is told so, and the same language is not.
#[test]
fn should_warn_when_the_running_pvsyst_speaks_another_language_than_the_map() {
    let exe = r"C:\Program Files\PVsyst8.1.8\PVsyst.exe";
    let mapa = super::recurso::mapa_de_ventanas();
    let medido = mapa["idioma"].as_str().unwrap();

    let igual = super::recurso::como_recurso(&vivo(exe, Some(medido)), Idioma::En);
    let ingles = super::recurso::como_recurso(&vivo(exe, Some("en")), Idioma::En);
    let italiano = super::recurso::como_recurso(&vivo(exe, Some("Italian")), Idioma::Es);

    assert_eq!(igual["metadatos"]["aviso"], serde_json::Value::Null);
    assert_eq!(ingles["metadatos"]["idioma_en_ejecucion"], "en");
    let aviso = ingles["metadatos"]["aviso"].as_str().unwrap();
    assert!(
        aviso.contains("'en'") && aviso.contains(&format!("'{medido}'")),
        "{aviso}"
    );
    let aviso = italiano["metadatos"]["aviso"].as_str().unwrap();
    assert!(
        aviso.contains("'Italian'") && aviso.contains("estructural"),
        "{aviso}"
    );
}
