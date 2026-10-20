//! Element catalogue definitions and insertion from PVsyst demo projects.

use crate::args::{self, opt_str, req_str};
use crate::config::Paths;
use crate::error::{Error, Result};
use crate::format::{edit, insert};
use crate::projects::anadir::baterias::ajustar_sistema;
use crate::projects::anadir::horizonte::neutralizar_horizonte;
use crate::projects::anadir::objetos::{campos_de, padre_de, verificar_tipo};
use crate::projects::anadir::{escribir_con_historial, leer_bytes, ruta_variante};
use crate::workspace::Alcance;
use serde_json::{json, Value};

/// One catalogue entry: the demo variant and the object types to copy from it.
pub(crate) struct Elemento {
    pub(crate) nombre: &'static str,
    pub(crate) demo: &'static str,
    pub(crate) tipos: &'static [&'static str],
    pub(crate) complementos: &'static [&'static str],
    pub(crate) campos_sistema: &'static [(&'static str, &'static str)],
}

pub(crate) const ELEMENTOS: &[Elemento] = &[
    Elemento {
        nombre: "baterias",
        demo: "_DEMO_Residential.VC5",
        tipos: &["pvGridStorage"],
        complementos: &["carga"],
        campos_sistema: &[
            ("Flags", "$10010040"),
            ("BatteryFile", "Generic_LI_Module_180Ah.BTR"),
            ("CycleSOWBeg", "1.000"),
            ("StaticSOWBeg", "1.000"),
            ("TypeTempBatt", "1"),
            ("TempBattRef", "20.0"),
            ("TempBattMois", "20,20,20,20,20,20,20,20,20,20,20,20,"),
            ("NBattSerie", "1"),
            ("NBattParall", "2"),
        ],
    },
    Elemento {
        nombre: "carga",
        demo: "_DEMO_Commercial.VC3",
        tipos: &["pvLoad"],
        complementos: &[],
        campos_sistema: &[],
    },
    Elemento {
        nombre: "horizonte",
        demo: "_DEMO_Residential.VC1",
        tipos: &["pvHorizon"],
        complementos: &[],
        campos_sistema: &[],
    },
    Elemento {
        nombre: "envejecimiento",
        demo: "_DEMO_Commercial.VC8",
        tipos: &["pvAgeing"],
        complementos: &[],
        campos_sistema: &[],
    },
    Elemento {
        nombre: "bifacial",
        demo: "_DEMO_Utility.VC2",
        tipos: &["pvBiFacial", "pvBiFacialSheds"],
        complementos: &[],
        campos_sistema: &[],
    },
    Elemento {
        nombre: "carbono",
        demo: "_DEMO_Residential.VC6",
        tipos: &["pvCarbonBalance"],
        complementos: &[],
        campos_sistema: &[],
    },
];

fn elementos_pendientes(
    catalogo: &'static Elemento,
    nuevo: &[u8],
) -> Result<Vec<&'static Elemento>> {
    let mut pendientes = vec![catalogo];
    for nombre in catalogo.complementos {
        let comp = ELEMENTOS
            .iter()
            .find(|e| e.nombre == *nombre)
            .ok_or_else(|| {
                Error::Invalid(format!("Complemento '{nombre}' ausente del catálogo"))
            })?;
        if edit::find_span(&edit::spans_of(nuevo), comp.tipos[0]).is_none() {
            pendientes.push(comp);
        }
    }
    Ok(pendientes)
}

fn insertar_elemento(
    mut nuevo: Vec<u8>,
    elem: &Elemento,
    insertados: &mut Vec<Value>,
) -> Result<Vec<u8>> {
    let ruta = crate::config::proyectos_demo_ro().join(elem.demo);
    let fuente = leer_bytes(&ruta).map_err(|e| {
        Error::NotFound(format!(
            "Elemento '{}': no se pudo leer la demo {}: {e}",
            elem.nombre,
            ruta.display()
        ))
    })?;
    let spans_fuente = edit::spans_of(&fuente);
    for tipo in elem.tipos {
        let si = edit::find_span(&spans_fuente, tipo)
            .ok_or_else(|| Error::NotFound(format!("No se encuentra {tipo} en {}", elem.demo)))?;
        let padre = padre_de(&spans_fuente, si).ok_or_else(|| {
            Error::Invalid(format!("{tipo} no tiene objeto padre en {}", elem.demo))
        })?;
        let bloque = insert::extraer_objeto(&fuente, tipo)?;
        let bloque = if elem.nombre == "horizonte" {
            neutralizar_horizonte(&bloque)?
        } else {
            bloque
        };
        nuevo = insert::insertar_objeto(&nuevo, &padre, &bloque, None, insert::SiExiste::Rechazar)?;
        insertados.push(json!({"tipo": tipo, "padre": padre, "campos": campos_de(&bloque)}));
    }
    Ok(nuevo)
}

/// `proyecto accion:anadir`: copies a catalogue element from a demo variant into target variant.
pub(crate) fn anadir(paths: &Paths, args: &Value) -> Result<Value> {
    let ws = args::workspace_of(paths, args);
    let proyecto = req_str(args, &["proyecto"], "proyecto")?;
    let elemento = req_str(args, &["elemento"], "elemento")?;
    let variante = opt_str(args, &["variante"]).unwrap_or("VC0");
    let catalogo = ELEMENTOS
        .iter()
        .find(|e| e.nombre == elemento)
        .ok_or_else(|| {
            let validos: Vec<&str> = ELEMENTOS.iter().map(|e| e.nombre).collect();
            Error::NotFound(format!(
                "Elemento '{elemento}' desconocido; válidos: {}",
                validos.join(", ")
            ))
        })?;

    let alcance = Alcance::de_args(args);
    let prj = crate::projects::resolve_project_file_con_alcance(&ws, proyecto, alcance)?;
    let destino = ruta_variante(&prj, variante)?;
    let mut nuevo = leer_bytes(&destino)?;
    let pendientes = elementos_pendientes(catalogo, &nuevo)?;
    let mut insertados = Vec::new();
    for elem in &pendientes {
        nuevo = insertar_elemento(nuevo, elem, &mut insertados)?;
    }
    let neutralizado = pendientes.iter().any(|e| e.nombre == "horizonte");
    if !catalogo.campos_sistema.is_empty() {
        nuevo = ajustar_sistema(&nuevo, catalogo.campos_sistema)?;
    }
    for elem in &pendientes {
        for tipo in elem.tipos {
            verificar_tipo(&nuevo, tipo)?;
        }
    }
    let id = escribir_con_historial(&ws, &destino, &nuevo, &format!("anadir {elemento}"))?;
    Ok(json!({
        "ok": true,
        "proyecto": proyecto,
        "variante": variante,
        "archivo": destino.display().to_string(),
        "elemento": elemento,
        "origen": catalogo.demo,
        "insertados": insertados,
        "neutralizado": neutralizado,
        "historial_id": id,
        "nota": "Edita los campos con editar accion:campos; luego proyecto accion:doctor"
    }))
}

#[cfg(test)]
#[path = "../../../tests/unit/projects/anadir/catalogo.rs"]
mod tests;
