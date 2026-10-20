//! `proyecto accion:doctor`: checks a project and its variants for duplicated
//! keys, unresolved components, site mismatches, demo comments and electrical limits.

pub(crate) mod electrico;
pub(crate) mod referencias;
pub(crate) mod sintaxis;

use crate::args::{self, flag, opt_str, req_str};
use crate::config::Paths;
use crate::error::{Error, Result};
use crate::projects::doctor::electrico::{revisar_electrico, ContextoElectrico};
use crate::projects::doctor::referencias::{revisar_componentes, revisar_sitio};
use crate::projects::doctor::sintaxis::{revisar_archivo, DestinoRevision};
use crate::workspace::Alcance;
use serde_json::{json, Value};
use std::fs;
use std::path::{Path, PathBuf};

/// Findings and repairs accumulated over one doctor run.
#[derive(Default)]
pub(crate) struct Informe {
    pub(crate) errores: Vec<String>,
    pub(crate) avisos: Vec<String>,
    pub(crate) reparado: Vec<String>,
    pub(crate) historial: Vec<String>,
}

pub(crate) fn texto(bytes: &[u8]) -> String {
    String::from_utf8_lossy(bytes).into_owned()
}

pub(crate) use crate::workspace::leer_bytes;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Modo {
    Revisar,
    Reparar,
}

struct ContextoVariante<'a> {
    ws: &'a Path,
    paths: &'a Paths,
    nombre: &'a str,
    prj_txt: &'a str,
    modo: Modo,
}

fn revisar_variante(ctx: &ContextoVariante<'_>, var: &Path, inf: &mut Informe) -> Result<()> {
    let destino = DestinoRevision {
        ws: ctx.ws,
        path: var,
        nombre: ctx.nombre,
    };
    revisar_archivo(&destino, ctx.modo, inf)?;
    let var_txt = texto(&leer_bytes(var)?);
    let etiqueta = var
        .file_name()
        .map_or_else(String::new, |n| n.to_string_lossy().into_owned());

    let mut parcial = Informe::default();
    revisar_componentes(ctx.ws, &var_txt, &mut parcial);
    let ctx_elec = ContextoElectrico {
        paths: ctx.paths,
        ws: ctx.ws,
        prj_txt: ctx.prj_txt,
        var_txt: &var_txt,
    };
    revisar_electrico(&ctx_elec, &mut parcial);

    inf.errores.extend(
        parcial
            .errores
            .into_iter()
            .map(|e| format!("{etiqueta}: {e}")),
    );
    inf.avisos.extend(
        parcial
            .avisos
            .into_iter()
            .map(|e| format!("{etiqueta}: {e}")),
    );
    Ok(())
}

fn variantes_de(prj: &Path, filtro: Option<&str>) -> Result<Vec<PathBuf>> {
    let dir = prj.parent().unwrap_or(Path::new("."));
    let stem = prj.file_stem().and_then(|s| s.to_str()).unwrap_or_default();
    let mut out = Vec::new();
    let entradas = fs::read_dir(dir)
        .map_err(|e| Error::io(format!("No se pudo listar {}", dir.display()), e))?;
    for entrada in entradas.flatten() {
        let p = entrada.path();
        let ext = p
            .extension()
            .and_then(|x| x.to_str())
            .unwrap_or("")
            .to_ascii_uppercase();
        let es_variante = ext.len() == 3 && (ext.starts_with("VC") || ext.starts_with("CM"));
        let coincide = filtro.is_none_or(|f| ext.eq_ignore_ascii_case(f));
        if es_variante && coincide && p.file_stem().and_then(|s| s.to_str()) == Some(stem) {
            out.push(p);
        }
    }
    out.sort();
    Ok(out)
}

/// `proyecto accion:doctor {proyecto, variante?, reparar?}`.
pub(crate) fn doctor(paths: &Paths, args: &Value) -> Result<Value> {
    let ws = args::workspace_of(paths, args);
    let proyecto = req_str(args, &["proyecto"], "proyecto")?;
    let modo = if flag(args, &["reparar"]) {
        Modo::Reparar
    } else {
        Modo::Revisar
    };
    let alcance = Alcance::de_args(args);
    let prj = crate::projects::resolve_project_file_con_alcance(&ws, proyecto, alcance)?;
    let nombre = prj
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or(proyecto)
        .to_string();
    let mut inf = Informe::default();

    let destino_prj = DestinoRevision {
        ws: &ws,
        path: &prj,
        nombre: &nombre,
    };
    revisar_archivo(&destino_prj, modo, &mut inf)?;
    let prj_txt = texto(&leer_bytes(&prj)?);
    revisar_sitio(&ws, &nombre, &prj_txt, &mut inf);

    let variantes = variantes_de(&prj, opt_str(args, &["variante"]))?;
    if variantes.is_empty() {
        inf.avisos.push(format!("Sin variantes para '{proyecto}'"));
    }
    let ctx_var = ContextoVariante {
        ws: &ws,
        paths,
        nombre: &nombre,
        prj_txt: &prj_txt,
        modo,
    };
    for var in &variantes {
        revisar_variante(&ctx_var, var, &mut inf)?;
    }

    Ok(json!({
        "ok": inf.errores.is_empty(),
        "errores": inf.errores,
        "avisos": inf.avisos,
        "reparado": inf.reparado,
        "historial_ids": inf.historial,
    }))
}
