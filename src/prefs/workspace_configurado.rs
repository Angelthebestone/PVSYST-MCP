//! PVsystCLI only loads the workspace configured in PVsyst (`DataPath` in
//! `PVsyst.ini`): `run-simulation`, `create-site` and `convert-meteo` crash with
//! `Instruction privilégiée` on any other one. The check runs before the command
//! so the caller gets the two paths instead of a crash.

use crate::config::Paths;
use crate::error::{Error, Result};
use crate::idioma::Texto;
use serde_json::{Map, Value};
use std::path::{Path, PathBuf};

const OTRO_WORKSPACE: Texto = Texto {
    es: "PVsystCLI {comando} solo acepta el workspace configurado en PVsyst (DataPath de PVsyst.ini): '{configurado}'. El workspace pedido es '{pedido}'. Usa el workspace configurado (PVSYST_WORKSPACE o el argumento workspace) o cambia DataPath en PVsyst.",
    en: "PVsystCLI {comando} only accepts the workspace configured in PVsyst (DataPath of PVsyst.ini): '{configurado}'. The requested workspace is '{pedido}'. Use the configured workspace (PVSYST_WORKSPACE or the workspace argument) or change DataPath in PVsyst.",
};

/// Refuses a `workspace` PVsystCLI cannot load for `comando`. The fake CLI used by
/// the tests is exempt, and an unreadable `DataPath` does not block the run.
pub(crate) fn exigir(paths: &Paths, workspace: &Path, comando: &str) -> Result<()> {
    if !es_cli_real(paths) {
        return Ok(());
    }
    let Some(configurado) = workspace_configurado() else {
        return Ok(());
    };
    if mismas_rutas(workspace, &configurado) {
        return Ok(());
    }
    Err(Error::Invalid(OTRO_WORKSPACE.texto_con(
        paths.idioma,
        &[
            ("comando", comando),
            ("configurado", &configurado.display().to_string()),
            ("pedido", &workspace.display().to_string()),
        ],
    )))
}

fn es_cli_real(paths: &Paths) -> bool {
    paths
        .cli_exe
        .file_name()
        .and_then(|n| n.to_str())
        .is_some_and(|n| n.eq_ignore_ascii_case("PVsystCLI.exe"))
}

fn workspace_configurado() -> Option<PathBuf> {
    let ruta = super::ini::ini_path().ok()?;
    let bytes = super::ini::leer(&ruta).ok()?;
    let (texto, _) = super::ini::decodificar(&bytes);
    let secciones = super::ini::parsear(&texto);
    let valor = valor_ini(&secciones, "System", "DataPath")?;
    let valor = valor.trim();
    (!valor.is_empty()).then(|| PathBuf::from(valor))
}

fn valor_ini(secciones: &Map<String, Value>, seccion: &str, clave: &str) -> Option<String> {
    let (_, sec) = secciones
        .iter()
        .find(|(k, _)| k.eq_ignore_ascii_case(seccion))?;
    let obj = sec.as_object()?;
    let (_, valor) = obj.iter().find(|(k, _)| k.eq_ignore_ascii_case(clave))?;
    valor.as_str().map(str::to_string)
}

fn mismas_rutas(a: &Path, b: &Path) -> bool {
    match (a.canonicalize(), b.canonicalize()) {
        (Ok(a), Ok(b)) => a == b,
        _ => normalizar_ruta(a) == normalizar_ruta(b),
    }
}

fn normalizar_ruta(p: &Path) -> String {
    p.to_string_lossy()
        .replace('/', "\\")
        .trim_end_matches('\\')
        .to_lowercase()
}

#[cfg(test)]
#[path = "../../tests/unit/prefs/workspace_configurado.rs"]
mod tests;
