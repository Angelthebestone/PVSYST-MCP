//! Geographical site consistency and module/inverter component reference checks.

use crate::components;
use crate::format::{insert, object};
use crate::projects::doctor::{texto, Informe};
use std::fs;
use std::path::Path;
use std::time::SystemTime;

/// Newest results CSV of `proyecto` and its `Geographical Site;` value.
fn sitio_de_resultados(ws: &Path, proyecto: &str) -> Option<String> {
    let destino = format!("{proyecto}.PRJ").to_ascii_lowercase();
    let mut mejor: Option<(SystemTime, String)> = None;
    let Ok(entries) = fs::read_dir(ws.join("Results")) else {
        return None;
    };
    for entrada in entries.flatten() {
        let p = entrada.path();
        if !p.extension().is_some_and(|x| x.eq_ignore_ascii_case("csv")) {
            continue;
        }
        let Ok(bytes) = fs::read(&p) else { continue };
        let mut prj_ok = false;
        let mut sitio = None;
        for linea in texto(&bytes).lines().take(80) {
            let mut c = linea.split(';');
            match c.next().map(str::trim) {
                Some("Project") => {
                    prj_ok = c
                        .next()
                        .is_some_and(|v| v.trim().to_ascii_lowercase() == destino);
                }
                Some("Geographical Site") => sitio = c.next().map(|v| v.trim().to_string()),
                _ => {}
            }
        }
        let fecha = entrada.metadata().and_then(|m| m.modified()).ok();
        if let (true, Some(s), Some(f)) = (prj_ok, sitio, fecha) {
            if mejor.as_ref().is_none_or(|(g, _)| f > *g) {
                mejor = Some((f, s));
            }
        }
    }
    mejor.map(|(_, s)| s)
}

/// Compares the PRJ `SitePrj.Site` with the newest results CSV of the project.
pub(crate) fn revisar_sitio(ws: &Path, proyecto: &str, prj_txt: &str, inf: &mut Informe) {
    let bloque = match insert::extraer_objeto(prj_txt.as_bytes(), "SitePrj") {
        Ok(b) => b,
        Err(e) => {
            inf.avisos
                .push(format!("No se pudo extraer SitePrj del proyecto: {e}"));
            return;
        }
    };
    let site =
        object::parse_first(&texto(&bloque)).and_then(|n| n.field("Site").map(str::to_string));
    let Some(csv_site) = sitio_de_resultados(ws, proyecto) else {
        return;
    };
    match site {
        Some(s) if !s.trim().eq_ignore_ascii_case(csv_site.trim()) => inf.avisos.push(format!(
            "Sitio del proyecto '{s}' distinto del último resultado '{csv_site}'"
        )),
        _ => {}
    }
}

pub(crate) fn aviso_no_resuelto(nombre: &str, error: &str) -> String {
    if nombre.to_ascii_lowercase().starts_with("generic_") {
        format!(
            "{nombre} no está como archivo (¿componente interno de PVsyst?); no se pudo verificar"
        )
    } else {
        format!("{nombre} no se encuentra en el workspace: {error}")
    }
}

/// Every `PVModule` / `GInverter` reference should resolve to a component file.
pub(crate) fn revisar_componentes(ws: &Path, var_txt: &str, inf: &mut Informe) {
    let Some(var) = object::parse_first(var_txt) else {
        inf.errores
            .push("Variante no es un archivo PVObject legible".into());
        return;
    };
    for (campo, kind) in [("PVModule", "PAN"), ("GInverter", "OND")] {
        let Some(nombre) = var.find_deep(campo) else {
            continue;
        };
        if nombre.is_empty() {
            inf.errores.push(format!("Referencia vacía en {campo}"));
        } else if let Err(e) = components::resolve_component(
            ws,
            &components::ConsultaComponente::referencia(kind, nombre),
        ) {
            inf.avisos.push(aviso_no_resuelto(nombre, &e.to_string()));
        }
    }
}

#[cfg(test)]
#[path = "../../../tests/unit/projects/doctor/referencias.rs"]
mod tests;
