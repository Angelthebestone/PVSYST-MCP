//! Element catalogue definitions and insertion from PVsyst demo projects.

use crate::args::{self, opt_str, req_str};
use crate::config::Paths;
use crate::error::{Error, Result};
use crate::format::{edit, insert};
use crate::idioma::{Idioma, Texto};
use crate::projects::anadir::baterias::ajustar_sistema;
use crate::projects::anadir::horizonte::neutralizar_horizonte;
use crate::projects::anadir::objetos::{campos_de, padre_de, verificar_tipo};
use crate::projects::anadir::{escribir_con_historial, ruta_variante};
use crate::projects::tipo_sistema::TipoSistema;
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
    /// System type the element belongs to: PVsyst cannot use it in any other.
    pub(crate) sistema: Option<TipoSistema>,
    /// What the caller has to edit after inserting it, when the content is the demo's.
    pub(crate) aviso: Option<Texto>,
}

/// The demo geometry of a field zone is not the project's.
const ZONA_DE_DEMO: Texto = Texto {
    es: "La geometría insertada es la de la demo (Zone #1 y sus módulos): edítala para que coincida con el proyecto",
    en: "The inserted geometry is the demo's (Zone #1 and its modules): edit it to match the project",
};

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
        sistema: None,
        aviso: None,
    },
    Elemento {
        nombre: "carga",
        demo: "_DEMO_Commercial.VC3",
        tipos: &["pvLoad"],
        complementos: &[],
        campos_sistema: &[],
        sistema: None,
        aviso: None,
    },
    Elemento {
        nombre: "horizonte",
        demo: "_DEMO_Residential.VC1",
        tipos: &["pvHorizon"],
        complementos: &[],
        campos_sistema: &[],
        sistema: None,
        aviso: None,
    },
    Elemento {
        nombre: "envejecimiento",
        demo: "_DEMO_Commercial.VC8",
        tipos: &["pvAgeing"],
        complementos: &[],
        campos_sistema: &[],
        sistema: None,
        aviso: None,
    },
    Elemento {
        nombre: "bifacial",
        demo: "_DEMO_Utility.VC2",
        tipos: &["pvBiFacial", "pvBiFacialSheds"],
        complementos: &[],
        campos_sistema: &[],
        sistema: None,
        aviso: None,
    },
    Elemento {
        nombre: "carbono",
        demo: "_DEMO_Residential.VC6",
        tipos: &["pvCarbonBalance"],
        complementos: &[],
        campos_sistema: &[],
        sistema: None,
        aviso: None,
    },
    Elemento {
        nombre: "domo",
        demo: "_DEMO_Commercial.VC7",
        tipos: &["pvDomeOrient"],
        complementos: &[],
        campos_sistema: &[],
        sistema: None,
        aviso: None,
    },
    Elemento {
        nombre: "zona_campo",
        demo: "_DEMO_Tracking.VC2",
        tipos: &["pvShdFieldZone"],
        complementos: &[],
        campos_sistema: &[],
        sistema: None,
        aviso: Some(ZONA_DE_DEMO),
    },
    Elemento {
        nombre: "bifacial_seguidor",
        demo: "_DEMO_Utility.VC5",
        tipos: &["pvBiFacial", "pvBiFacialTrackers"],
        complementos: &[],
        campos_sistema: &[],
        sistema: None,
        aviso: None,
    },
    Elemento {
        nombre: "circuito_hidraulico",
        demo: "_DEMO_Pumping.VC0",
        tipos: &["pvHydrCircuit"],
        complementos: &[],
        campos_sistema: &[
            ("SystemType", "Pumping"),
            ("PumpFile", "Generic_6kW_H60_160_FR10.PMP"),
        ],
        sistema: Some(TipoSistema::Bombeo),
        aviso: None,
    },
];

/// Whether an element brings the objects it needs to work, or only itself.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum AlcanceElemento {
    /// The element plus its complements (a battery needs a load to discharge into).
    ConComplementos,
    /// Only the element: inserting objects the caller did not ask for changes the project.
    SoloElElemento,
}

fn elementos_pendientes(
    catalogo: &'static Elemento,
    nuevo: &[u8],
    alcance: AlcanceElemento,
) -> Result<Vec<&'static Elemento>> {
    let mut pendientes = vec![catalogo];
    if alcance == AlcanceElemento::SoloElElemento {
        return Ok(pendientes);
    }
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

/// `bloque` inserted under `padre` unless the variant already has a `tipo`; the flag
/// says whether it was inserted.
fn insertar_si_falta(
    actual: Vec<u8>,
    (tipo, padre): (&str, &str),
    bloque: &[u8],
) -> Result<(Vec<u8>, bool)> {
    if edit::find_span(&edit::spans_of(&actual), tipo).is_some() {
        return Ok((actual, false));
    }
    let nuevo = insert::insertar_objeto(&actual, padre, bloque, None, insert::SiExiste::Rechazar)?;
    Ok((nuevo, true))
}

fn insertar_elemento(
    nuevo: Vec<u8>,
    elem: &Elemento,
) -> Result<(Vec<u8>, Vec<Value>, Vec<String>)> {
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
        (nuevo, Vec::<Value>::new(), Vec::<String>::new()),
        |(actual, insertados, ya_presentes), tipo| {
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
            let (actual, insertado) = insertar_si_falta(actual, (tipo, &padre), &bloque)?;
            // A type already in the variant was not inserted: it is reported as present,
            // so a call that changed nothing never looks like a call that did.
            let nuevo = insertado
                .then(|| json!({"tipo": tipo, "padre": padre, "campos": campos_de(&bloque)}));
            let presente = (!insertado).then(|| tipo.to_string());
            Ok((
                actual,
                insertados.into_iter().chain(nuevo).collect(),
                ya_presentes.into_iter().chain(presente).collect(),
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

/// Validates the catalogue element name without reading any file.
pub(crate) fn validar_elemento(args: &Value) -> Result<()> {
    catalogo_de(req_str(args, &["elemento"], "elemento")?).map(|_| ())
}

/// Bytes after inserting catalogue element `nombre`, plus what was inserted.
#[derive(Debug)]
pub(crate) struct ElementoAplicado {
    pub(crate) bytes: Vec<u8>,
    pub(crate) insertados: Vec<Value>,
    /// Types the variant already had: nothing was inserted for them.
    pub(crate) ya_presentes: Vec<String>,
}

/// Insert catalogue element `nombre` into `bytes`, with the complements `alcance` asks for.
pub(crate) fn aplicar_elemento(
    bytes: Vec<u8>,
    nombre: &str,
    alcance: AlcanceElemento,
) -> Result<ElementoAplicado> {
    let catalogo = catalogo_de(nombre)?;
    exigir_sistema(catalogo, &bytes)?;
    let pendientes = elementos_pendientes(catalogo, &bytes, alcance)?;
    let (bytes, insertados, ya_presentes) = pendientes.iter().try_fold(
        (bytes, Vec::<Value>::new(), Vec::<String>::new()),
        |(actual, insertados, ya_presentes), elem| {
            let (nuevo, nuevos, presentes) = insertar_elemento(actual, elem)?;
            Ok((
                nuevo,
                insertados.into_iter().chain(nuevos).collect(),
                ya_presentes.into_iter().chain(presentes).collect(),
            ))
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
    Ok(ElementoAplicado {
        bytes,
        insertados,
        ya_presentes,
    })
}

/// The element belongs to one system type: in another, PVsyst cannot use the object.
fn exigir_sistema(catalogo: &Elemento, bytes: &[u8]) -> Result<()> {
    let Some(esperado) = catalogo.sistema else {
        return Ok(());
    };
    TipoSistema::exigir(bytes, esperado).map_err(|e| {
        Error::Invalid(format!(
            "El elemento '{}' solo aplica a proyectos de {:?}: {e}",
            catalogo.nombre, esperado
        ))
    })
}

/// The variant already had the element: the call changed nothing.
const YA_PRESENTE: Texto = Texto {
    es: "La variante ya tiene {tipos}: no se insertó nada",
    en: "The variant already has {tipos}: nothing was inserted",
};

/// What the caller has to know about this insertion.
fn avisos_de(catalogo: &Elemento, aplicado: &ElementoAplicado) -> Vec<String> {
    let idioma = Idioma::de_entorno();
    let mut avisos = Vec::new();
    if aplicado.insertados.is_empty() && !aplicado.ya_presentes.is_empty() {
        avisos.push(YA_PRESENTE.texto_con(idioma, &[("tipos", &aplicado.ya_presentes.join(", "))]));
    }
    if let Some(texto) = catalogo.aviso {
        avisos.push(texto.en_idioma(idioma).to_string());
    }
    avisos
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
    let aplicado = aplicar_elemento(
        leer_bytes(&destino)?,
        elemento,
        AlcanceElemento::ConComplementos,
    )?;
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
    let mut respuesta = json!({
        "ok": true,
        "proyecto": proyecto,
        "variante": variante,
        "archivo": destino.display().to_string(),
        "elemento": elemento,
        "origen": catalogo.demo,
        "insertados": aplicado.insertados,
        "ya_presentes": aplicado.ya_presentes,
        "neutralizado": neutralizado,
        "historial_id": id,
        "nota": "Edita los campos con editar accion:campos; luego proyecto accion:doctor"
    });
    let avisos = avisos_de(catalogo, &aplicado);
    if !avisos.is_empty() {
        respuesta["avisos"] = json!(avisos);
    }
    Ok(respuesta)
}

#[cfg(test)]
#[path = "../../../tests/unit/projects/anadir/catalogo.rs"]
mod tests;
