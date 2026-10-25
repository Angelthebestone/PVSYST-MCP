//! `leer_resultados_csv`: read the most recent PVsyst result CSVs.

mod lookup;
mod parse;

pub(crate) use lookup::{resultado_de, variant_kwp};

use crate::args::{flag, workspace_of};
use crate::config::Paths;
use crate::error::Result;
use serde_json::{json, Value};
use std::path::Path;

const TOPE_TEXTO_CSV: usize = 32_768;

fn recorte_seguro(texto: &str, tope: usize) -> &str {
    if texto.len() <= tope {
        return texto;
    }
    let mut fin = tope;
    while !texto.is_char_boundary(fin) {
        fin -= 1;
    }
    &texto[..fin]
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum IncluirTexto {
    Si,
    No,
}

impl IncluirTexto {
    pub(crate) fn de_bool(v: bool) -> Self {
        if v {
            Self::Si
        } else {
            Self::No
        }
    }
}

pub(crate) fn resumen_de_csv(p: &Path, include_text: IncluirTexto) -> Result<Value> {
    let text = lookup::leer_texto(p)?;
    let mut entry = json!({
        "path": p.to_string_lossy(),
        "size_bytes": text.len(),
        "summary": parse::summarize_csv(&text),
    });
    if include_text == IncluirTexto::Si {
        let body = if text.len() > TOPE_TEXTO_CSV {
            format!(
                "{}\n...[truncated at {} of {} bytes; use leer with offset]",
                recorte_seguro(&text, TOPE_TEXTO_CSV),
                TOPE_TEXTO_CSV,
                text.len()
            )
        } else {
            text
        };
        entry["text"] = json!(body);
    }
    Ok(entry)
}

pub(crate) fn leer_resultados(paths: &Paths, args: &Value) -> Result<Value> {
    let workspace = workspace_of(paths, args);
    let count = args
        .get("count")
        .and_then(serde_json::Value::as_u64)
        .unwrap_or(1)
        .max(1) as usize;
    let include_text = IncluirTexto::de_bool(flag(args, &["include_text"]));

    let results = lookup::csvs_a_leer(&workspace, args, count)?
        .iter()
        .map(|p| resumen_de_csv(p, include_text))
        .collect::<Result<Vec<_>, _>>()?;
    Ok(json!({ "count": results.len(), "results": results }))
}

#[cfg(test)]
#[path = "../../../tests/unit/simulation/results/mod.rs"]
mod tests;
