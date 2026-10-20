//! PV module (.PAN) creation from datasheet values.

use super::{con_bom, destino, sin_bom, valor_texto};
use crate::components::{compos_dir, pan::parse_pan};
use crate::config::compos_ro;
use crate::error::{Error, Result};
use crate::format::{edit, object};
use crate::history::escribir_atomico;
use crate::workspace::walk_ext;
use serde_json::{json, Map, Value};
use std::fs;
use std::path::{Path, PathBuf};

/// First text `.PAN` (PVsyst >= 6.40 layout) in the workspace, then DataRO.
fn plantilla_pan(ws: &Path) -> Result<(PathBuf, Vec<u8>)> {
    let mut candidatos = Vec::new();
    let mut avisos = Vec::new();
    for raiz in [compos_dir(ws, "PVModules"), compos_ro().join("PVModules")] {
        avisos.extend(walk_ext(&raiz, ".pan", &mut candidatos).advertencias());
    }
    candidatos.sort();
    for ruta in candidatos {
        if let Ok(bytes) = fs::read(&ruta) {
            let es_texto = parse_pan(&bytes)
                .ok()
                .and_then(|v| v.get("format").and_then(Value::as_str).map(str::to_string))
                .is_some_and(|f| f == "pvsyst8-text");
            if es_texto {
                return Ok((ruta, bytes));
            }
        }
    }
    let incompleto = if avisos.is_empty() {
        String::new()
    } else {
        format!(" (búsqueda incompleta: {})", avisos.join("; "))
    };
    Err(Error::Invalid(format!("No hay plantilla .PAN de texto{incompleto}: exporta un módulo desde PVsyst (Bases de datos → Módulos FV → Exportar) a ComposPV\\PVModules y reintenta")))
}

pub(crate) fn pan(ws: &Path, nombre: &str, datos: &Map<String, Value>) -> Result<Value> {
    let (plantilla, original) = plantilla_pan(ws)?;
    let (bom, cuerpo) = sin_bom(&original);
    let texto = String::from_utf8_lossy(cuerpo).into_owned();
    let raiz = object::parse_first(&texto)
        .ok_or_else(|| Error::Format("La plantilla .PAN no se puede interpretar".into()))?;

    let cambios = datos
        .iter()
        .map(|(k, v)| Ok((k.clone(), Some(valor_texto(k, v)?))))
        .collect::<Result<Vec<_>>>()?;
    let (salida, resultados) = edit::edit_fields(cuerpo, &raiz.obj_type, &cambios)?;

    let mut aplicados = Map::new();
    let mut ignorados = Vec::new();
    for r in resultados {
        if r.action == "not-found" {
            ignorados.push(r.key);
        } else {
            aplicados.insert(r.key, json!(r.new_value));
        }
    }

    let salida = con_bom(bom, &salida);
    let dest = destino(ws, "PVModules", &format!("{nombre}.PAN"))?;
    escribir_atomico(&dest, &salida)?;
    Ok(json!({
        "ok": true,
        "tipo": "PAN",
        "archivo": dest.display().to_string(),
        "plantilla": plantilla.display().to_string(),
        "aplicados": aplicados,
        "ignorados": ignorados,
    }))
}
