use super::caso::{caso, especifica, round1, Caso};
use crate::args::{req_str, workspace_of};
use crate::config::Paths;
use crate::error::{Error, Result};
use crate::objects::leer_objeto;
use crate::simulation::report_pdf;
use crate::simulation::results::resultado_de;
use serde_json::{json, Map, Value};
use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::Path;

pub(super) const CLAVES_EXCLUIDAS: &[&str] = &[
    "Day_", "VarSimul", "Varident", "ValMonth", "ValuesX", "ValuesY", "ValuesZ", "Accum", "Date",
    "WRFactor",
];
pub(super) const MAX_DIFERENCIAS: usize = 100;

pub(super) fn excluida(clave: &str) -> bool {
    CLAVES_EXCLUIDAS.iter().any(|p| clave.starts_with(p))
}

pub(super) fn kpi_de(c: &Caso) -> Result<Value> {
    let resumen = report_pdf::resumen(&c.tokens)?;
    let perdidas: Map<String, Value> = report_pdf::perdidas(&c.tokens)?["nodos"]
        .as_array()
        .into_iter()
        .flatten()
        .filter_map(|n| Some((n["concepto"].as_str()?.to_string(), n.get("pct")?.clone())))
        .collect();
    Ok(json!({
        "variante": c.variante,
        "energia_kwh": c.energia_kwh,
        "produccion_especifica_kwh_kwp": especifica(c),
        "pr_pct": resumen["pr_pct"],
        "perdidas": perdidas,
    }))
}

pub(super) fn deltas_respecto_primera(kpis: &[Value]) -> Vec<Value> {
    let Some(base) = kpis.first() else {
        return Vec::new();
    };
    kpis.iter()
        .skip(1)
        .map(|k| {
            let mut d = Map::new();
            for campo in ["energia_kwh", "produccion_especifica_kwh_kwp", "pr_pct"] {
                if let (Some(a), Some(b)) = (k[campo].as_f64(), base[campo].as_f64()) {
                    d.insert(campo.into(), json!(round1(a - b)));
                }
            }
            if let (Some(a), Some(b)) = (k["energia_kwh"].as_f64(), base["energia_kwh"].as_f64()) {
                d.insert("energia_pct".into(), json!(round1((a / b - 1.0) * 100.0)));
            }
            let perdidas: Map<String, Value> = base["perdidas"]
                .as_object()
                .into_iter()
                .flatten()
                .filter_map(|(label, v)| {
                    let delta = k["perdidas"][label.as_str()].as_f64()? - v.as_f64()?;
                    Some((label.clone(), json!(round1(delta))))
                })
                .collect();
            d.insert("perdidas_pct".into(), Value::Object(perdidas));
            json!({"variante": k["variante"], "frente_a": base["variante"], "delta": d})
        })
        .collect()
}

pub(super) fn diferencias_parametros(
    paths: &Paths,
    workspace: &Path,
    proyecto: &str,
    variantes: &[String],
) -> Result<Vec<Value>> {
    let mut mapas: Vec<(&str, Map<String, Value>)> = Vec::new();
    for v in variantes {
        let archivo = workspace.join("Projects").join(format!("{proyecto}.{v}"));
        let mut mapa = Map::new();
        for item in leer_objeto(paths, &json!({"archivo": archivo.display().to_string()}))?
            .as_array()
            .into_iter()
            .flatten()
        {
            if let (Some(c), Some(val)) = (item["campo"].as_str(), item.get("valor")) {
                if !excluida(c) {
                    mapa.insert(c.to_string(), val.clone());
                }
            }
        }
        mapas.push((v, mapa));
    }
    let mut claves: BTreeSet<&str> = BTreeSet::new();
    for (_, m) in &mapas {
        claves.extend(m.keys().map(String::as_str));
    }
    let mut diferencias = Vec::new();
    for c in claves {
        let mut valores = BTreeMap::new();
        for (v, m) in &mapas {
            valores.insert(*v, m.get(c).cloned().unwrap_or(Value::Null));
        }
        let iguales = valores
            .values()
            .all(|val| val == &valores[variantes[0].as_str()]);
        if !iguales {
            diferencias.push(json!({"parametro": c, "valores": valores}));
            if diferencias.len() >= MAX_DIFERENCIAS {
                break;
            }
        }
    }
    Ok(diferencias)
}

pub(super) fn variantes_con_resultados(workspace: &Path, proyecto: &str) -> Vec<String> {
    let Ok(entradas) = fs::read_dir(workspace.join("Projects")) else {
        return Vec::new();
    };
    let mut out: Vec<String> = entradas
        .filter_map(std::result::Result::ok)
        .map(|e| e.path())
        .filter(|p| {
            p.file_stem()
                .and_then(|s| s.to_str())
                .is_some_and(|s| s.eq_ignore_ascii_case(proyecto))
        })
        .filter_map(|p| {
            p.extension()
                .and_then(|e| e.to_str())
                .map(str::to_uppercase)
        })
        .filter(|e| e.starts_with("VC") || e.starts_with("CM"))
        .filter(|e| {
            matches!(
                resultado_de(workspace, Some(proyecto), Some(e), None),
                Ok((_, Some(_)))
            )
        })
        .collect();
    out.sort();
    out.dedup();
    out
}

pub(crate) fn comparar(paths: &Paths, args: &Value) -> Result<Value> {
    let workspace = workspace_of(paths, args);
    let proyecto = req_str(args, &["proyecto", "project"], "proyecto")?;
    let variantes: Vec<String> = match args.get("variantes").and_then(Value::as_array) {
        Some(lista) => lista
            .iter()
            .filter_map(Value::as_str)
            .map(str::to_string)
            .collect(),
        None => variantes_con_resultados(&workspace, proyecto),
    };
    if variantes.len() < 2 {
        return Err(Error::Invalid(format!(
            "Se necesitan al menos 2 variantes con resultados en '{proyecto}' (hay {})",
            variantes.len()
        )));
    }
    let mut kpis = Vec::new();
    for v in &variantes {
        let argumentos = json!({"workspace": workspace.display().to_string(),
                                "proyecto": proyecto, "variante": v});
        kpis.push(kpi_de(&caso(paths, &argumentos)?)?);
    }
    let parametros = diferencias_parametros(paths, &workspace, proyecto, &variantes)?;
    Ok(json!({
        "proyecto": proyecto,
        "variantes": variantes,
        "parametros_distintos": parametros,
        "kpis": kpis,
        "deltas_respecto_primera": deltas_respecto_primera(&kpis),
    }))
}
