//! Element catalogue definitions and insertion from PVsyst demo projects.

use crate::args::{self, opt_str, req_str};
use crate::config::Paths;
use crate::error::{Error, Result};
use crate::format::{edit, insert};
use crate::projects::anadir::baterias::ajustar_sistema;
use crate::projects::anadir::horizonte::neutralizar_horizonte;
use crate::projects::anadir::objetos::{campos_de, padre_de, verificar_tipo};
use crate::projects::anadir::{escribir_con_historial, ruta_variante};
use crate::workspace::leer_bytes;
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
                Error::NotFound(format!("Complemento '{nombre}' ausente del catálogo"))
            })?;
        if edit::find_span(&edit::spans_of(nuevo), comp.tipos[0]).is_none() {
            pendientes.push(comp);
        }
    }
    Ok(pendientes)
}

fn insertar_elemento(nuevo: Vec<u8>, elem: &Elemento) -> Result<(Vec<u8>, Vec<Value>)> {
    let ruta = crate::config::proyectos_demo_ro().join(elem.demo);
    let fuente = leer_bytes(&ruta).map_err(|e| {
        Error::NotFound(format!(
            "Elemento '{}': no se pudo leer la demo {}: {e}",
            elem.nombre,
            ruta.display()
        ))
    })?;
    let spans_fuente = edit::spans_of(&fuente);
    elem.tipos.iter().try_fold(
        (nuevo, Vec::<Value>::new()),
        |(actual, insertados), tipo| {
            let si = edit::find_span(&spans_fuente, tipo).ok_or_else(|| {
                Error::NotFound(format!("No se encuentra {tipo} en {}", elem.demo))
            })?;
            let padre = padre_de(&spans_fuente, si).ok_or_else(|| {
                Error::Invalid(format!("{tipo} no tiene objeto padre en {}", elem.demo))
            })?;
            let bloque = insert::extraer_objeto(&fuente, tipo)?;
            let bloque = if elem.nombre == "horizonte" {
                neutralizar_horizonte(&bloque)?
            } else {
                bloque
            };
            let actual = insert::insertar_objeto(
                &actual,
                &padre,
                &bloque,
                None,
                insert::SiExiste::Rechazar,
            )?;
            let insertado = json!({"tipo": tipo, "padre": padre, "campos": campos_de(&bloque)});
            Ok((
                actual,
                insertados
                    .into_iter()
                    .chain(std::iter::once(insertado))
                    .collect(),
            ))
        },
    )
}

fn catalogo_de(nombre: &str) -> Result<&'static Elemento> {
    ELEMENTOS
        .iter()
        .find(|e| e.nombre == nombre)
        .ok_or_else(|| {
            let validos: Vec<&str> = ELEMENTOS.iter().map(|e| e.nombre).collect();
            Error::NotFound(format!(
                "Elemento '{nombre}' desconocido; válidos: {}",
                validos.join(", ")
            ))
        })
}

/// Bytes after inserting catalogue element `nombre`, plus what was inserted.
#[derive(Debug)]
pub(crate) struct ElementoAplicado {
    pub(crate) bytes: Vec<u8>,
    pub(crate) insertados: Vec<Value>,
}

/// Insert catalogue element `nombre` into `bytes`, including its complements.
pub(crate) fn aplicar_elemento(bytes: Vec<u8>, nombre: &str) -> Result<ElementoAplicado> {
    let catalogo = catalogo_de(nombre)?;
    let pendientes = elementos_pendientes(catalogo, &bytes)?;
    let (bytes, insertados) = pendientes.iter().try_fold(
        (bytes, Vec::<Value>::new()),
        |(actual, insertados), elem| {
            let (nuevo, nuevos) = insertar_elemento(actual, elem)?;
            Ok((nuevo, insertados.into_iter().chain(nuevos).collect()))
        },
    )?;
    let bytes = if catalogo.campos_sistema.is_empty() {
        bytes
    } else {
        ajustar_sistema(&bytes, catalogo.campos_sistema)?
    };
    pendientes.iter().try_for_each(|elem| {
        elem.tipos
            .iter()
            .try_for_each(|tipo| verificar_tipo(&bytes, tipo))
    })?;
    Ok(ElementoAplicado { bytes, insertados })
}

/// `proyecto accion:anadir`: copies a catalogue element from a demo variant into target variant.
pub(crate) fn anadir(paths: &Paths, args: &Value) -> Result<Value> {
    let ws = args::workspace_of(paths, args);
    let proyecto = req_str(args, &["proyecto"], "proyecto")?;
    let elemento = req_str(args, &["elemento"], "elemento")?;
    let variante = opt_str(args, &["variante"]).unwrap_or("VC0");
    let catalogo = catalogo_de(elemento)?;

    let alcance = Alcance::de_args(args);
    let prj = crate::projects::resolve_project_file_con_alcance(&ws, proyecto, alcance)?;
    let destino = ruta_variante(&prj, variante)?;
    let aplicado = aplicar_elemento(leer_bytes(&destino)?, elemento)?;
    let neutralizado = aplicado
        .insertados
        .iter()
        .any(|v| v.get("tipo").and_then(Value::as_str) == Some("pvHorizon"));
    let id = escribir_con_historial(
        &ws,
        &destino,
        &aplicado.bytes,
        &format!("anadir {elemento}"),
    )?;
    Ok(json!({
        "ok": true,
        "proyecto": proyecto,
        "variante": variante,
        "archivo": destino.display().to_string(),
        "elemento": elemento,
        "origen": catalogo.demo,
        "insertados": aplicado.insertados,
        "neutralizado": neutralizado,
        "historial_id": id,
        "nota": "Edita los campos con editar accion:campos; luego proyecto accion:doctor"
    }))
}

#[cfg(test)]
#[path = "../../../tests/unit/projects/anadir/catalogo.rs"]
mod tests;
