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
                "{}\n...[truncated at {} of {} bytes; use leer_archivo with offset]",
                &text[..text.floor_char_boundary(TOPE_TEXTO_CSV)],
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
mod tests {
    use super::*;
    use std::fs;

    fn resultado(proyecto: &str, variante: &str) -> String {
        format!("Project;{proyecto}.PRJ;\nSimulation variant;{proyecto}.{variante};\ndate;E_Grid\n01/01/90 01:00;1\n")
    }

    #[test]
    fn should_read_only_the_csv_whose_metadata_matches_project_and_variant() {
        let ws = tempfile::tempdir().expect("ws");
        let dir = ws.path().join("Results");
        fs::create_dir_all(&dir).expect("Results");
        fs::write(dir.join("a.csv"), resultado("MCP_E2E", "VC0")).expect("a");
        fs::write(dir.join("b.csv"), resultado("MCP_E2E", "VC1")).expect("b");
        fs::write(dir.join("c.csv"), resultado("OTRO", "VC0")).expect("c");

        let args =
            json!({ "workspace": ws.path(), "proyecto": "mcp_e2e", "variante": "vc0", "count": 5 });
        let out = leer_resultados(&Paths::resolve(), &args).expect("read");
        assert_eq!(out["count"], 1);
        assert!(out["results"][0]["path"]
            .as_str()
            .expect("path")
            .ends_with("a.csv"));
    }

    #[test]
    fn should_reject_missing_csv_file_when_reading_results() {
        let ws = tempfile::tempdir().expect("ws");
        let args = json!({ "archivo": "no_existe.csv", "workspace": ws.path() });
        assert!(leer_resultados(&Paths::resolve(), &args).is_err());
    }

    #[test]
    fn should_reject_results_dir_outside_workspace_by_default() {
        let ws = tempfile::tempdir().expect("ws");
        let fuera = tempfile::tempdir().expect("fuera");
        fs::write(fuera.path().join("x.csv"), resultado("P", "VC0")).expect("x");
        let args = json!({ "workspace": ws.path(), "dir": fuera.path() });
        assert!(leer_resultados(&Paths::resolve(), &args).is_err());
    }
}
