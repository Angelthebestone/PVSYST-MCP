//! `pvsyst://gui`: the catalogue plus the window map observed on a live PVsyst.

use super::dominio::Funcion;
use super::entradas::funciones;
use crate::idioma::{Idioma, Texto};
use serde_json::{json, Value};
use std::path::Path;
use std::sync::OnceLock;

/// Windows seen while walking the interface: classes, titles, tabs and controls.
const MAPA_DE_VENTANAS: &str = include_str!("../../../../docs/gui_mapa.json");

/// The window map, parsed once; `Null` if the embedded file is not valid JSON.
pub(crate) fn mapa_de_ventanas() -> &'static Value {
    static MAPA: OnceLock<Value> = OnceLock::new();
    MAPA.get_or_init(|| serde_json::from_str(MAPA_DE_VENTANAS).unwrap_or(Value::Null))
}

/// `8.1.8` out of `8.1.8 (most windows were walked on 8.1.7)`.
fn version_medida(mapa: &Value) -> Option<&str> {
    mapa.get("version_pvsyst")?
        .as_str()?
        .split_whitespace()
        .next()
}

/// Version of the PVsyst install the server drives: the numbers of the folder
/// its executable sits in (`PVsyst8.1.8`); `None` when it is not an install folder.
fn version_en_ejecucion(gui_exe: &Path) -> Option<String> {
    let carpeta = gui_exe.parent()?.file_name()?.to_str()?;
    let numeros = crate::config::version_de(carpeta)?;
    Some(
        numeros
            .iter()
            .map(u32::to_string)
            .collect::<Vec<_>>()
            .join("."),
    )
}

/// How much of the GUI the map covers: windows, those with fields mapped, and
/// the controls and limits it records as unreachable.
fn cobertura(mapa: &Value) -> Value {
    let ventanas = mapa.get("ventanas").and_then(Value::as_object);
    let con_campos = ventanas.map_or(0, |v| v.values().filter(|w| tiene_campos(w)).count());
    let largo = |clave: &str| {
        mapa.get(clave)
            .and_then(Value::as_array)
            .map_or(0, Vec::len)
    };
    json!({
        "ventanas_con_campos": con_campos,
        "controles_sin_hwnd": largo("controles_sin_hwnd"),
        "limites": largo("limites"),
    })
}

fn tiene_campos(ventana: &Value) -> bool {
    ventana
        .get("instancias")
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
        .filter_map(|i| i.get("paginas")?.as_array())
        .flatten()
        .any(|p| {
            p.get("campos")
                .and_then(Value::as_array)
                .is_some_and(|c| !c.is_empty())
        })
}

const VERSION_DISTINTA: Texto = Texto {
    es: "La GUI en ejecución es PVsyst {en_ejecucion}; el mapa se midió en {medida}: las pestañas, anclas y posiciones de menú pueden haber cambiado. Lo que vale es el inventario en vivo (gui_observar que:inventario).",
    en: "The running GUI is PVsyst {en_ejecucion}; the map was measured on {medida}: tabs, anchors and menu positions may have changed. The live inventory (gui_observar que:inventario) is what holds.",
};

/// What the map says about itself: when and on which version it was measured,
/// how big it is and, if the running PVsyst differs, a warning.
fn metadatos(mapa: &Value, gui_exe: &Path, idioma: Idioma) -> Value {
    let medida = version_medida(mapa);
    let en_ejecucion = version_en_ejecucion(gui_exe);
    let aviso = match (&en_ejecucion, medida) {
        (Some(vivo), Some(medida)) if vivo != medida => {
            Some(VERSION_DISTINTA.texto_con(idioma, &[("en_ejecucion", vivo), ("medida", medida)]))
        }
        _ => None,
    };
    json!({
        "measured_on": mapa.get("medido_el"),
        "pvsyst_version": medida,
        "pvsyst_version_en_ejecucion": en_ejecucion,
        "idioma_medido": mapa.get("idioma"),
        "windows_total": mapa.get("ventanas").and_then(Value::as_object).map_or(0, serde_json::Map::len),
        "cobertura": cobertura(mapa),
        "aviso": aviso,
    })
}

/// Machine-readable map of the GUI: every catalogued function and every window.
/// `gui_exe` is the executable the server drives, to compare its version with
/// the one the map was measured on.
pub(crate) fn como_recurso(gui_exe: &Path, idioma: Idioma) -> Value {
    let mapa = mapa_de_ventanas();
    let uso = Texto {
        es: "gui_actuar accion:funcion funcion:<clave> abre una función; gui_actuar accion:menu ventana id abre un menú no catalogado.",
        en: "gui_actuar accion:funcion funcion:<clave> opens a function; gui_actuar accion:menu ventana id opens a menu that is not catalogued.",
    };
    json!({
        "uso": uso.en_idioma(idioma),
        "metadatos": metadatos(mapa, gui_exe, idioma),
        "funciones": funciones().map(Funcion::a_json).collect::<Vec<_>>(),
        "mapa": mapa,
    })
}
