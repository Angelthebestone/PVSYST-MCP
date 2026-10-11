//! Duplicated configuration key inspection, cleanup, and demo comment detection.

use crate::error::Result;
use crate::format::{edit, insert, object};
use crate::projects::anadir::escribir_con_historial;
use crate::projects::doctor::{leer_bytes, texto, Informe, Modo};
use std::collections::HashMap;
use std::path::Path;

/// Target file context for syntax and header inspection.
pub(crate) struct DestinoRevision<'a> {
    pub(crate) ws: &'a Path,
    pub(crate) path: &'a Path,
    pub(crate) nombre: &'a str,
}

/// Selector `tipo[k]` for every object of `bytes`, counting per type.
fn selectores(bytes: &[u8]) -> Vec<String> {
    let mut cuenta: HashMap<String, usize> = HashMap::new();
    edit::spans_of(bytes)
        .iter()
        .map(|s| {
            let k = cuenta.entry(s.ty.clone()).or_insert(0);
            let sel = format!("{}[{}]", s.ty, k);
            *k += 1;
            sel
        })
        .collect()
}

/// Object types whose repeated keys are real conflicts.
const TIPOS_CONFIGURACION: &[&str] = &["pvOrient", "pvSubArray", "pvSystem"];

/// Configuration objects of `bytes` with conflicting duplicated keys.
pub(crate) fn buscar_duplicados(bytes: &[u8]) -> Vec<(String, String)> {
    selectores(bytes)
        .into_iter()
        .filter(|sel| {
            let tipo = sel.split('[').next().unwrap_or_default();
            TIPOS_CONFIGURACION.contains(&tipo)
        })
        .filter_map(|sel| {
            let dups: Vec<(String, Vec<String>)> = insert::claves_duplicadas(bytes, &sel)
                .into_iter()
                .filter(|(k, v)| !k.ends_with("_contd") && v.iter().any(|x| *x != v[0]))
                .collect();
            if dups.is_empty() {
                return None;
            }
            let detalle: Vec<String> = dups
                .iter()
                .map(|(k, v)| format!("{k}: {}", v.join(" | ")))
                .collect();
            Some((sel.clone(), format!("{sel} duplica {}", detalle.join("; "))))
        })
        .collect()
}

fn limpiar_comentario_demo(
    bytes: &[u8],
    nombre: &str,
    etiqueta: &str,
    inf: &mut Informe,
) -> Result<Option<Vec<u8>>> {
    let raiz = object::parse_first(&texto(bytes));
    let root_ty = edit::spans_of(bytes).first().map(|s| s.ty.clone());
    if let (Some(root), Some(ty)) = (raiz, root_ty) {
        let comentario = root.field("Comment").unwrap_or("");
        if comentario.contains("DEMO") && !nombre.starts_with("_DEMO") {
            let cambios = [("Comment".to_string(), Some(nombre.to_string()))];
            let nuevo = edit::edit_fields(bytes, &ty, &cambios)?.0;
            inf.reparado.push(format!(
                "{etiqueta}: Comment de demo sustituido por '{nombre}'"
            ));
            return Ok(Some(nuevo));
        }
    }
    Ok(None)
}

/// Removes duplicated keys and repairs demo Comment when `modo` is Reparar.
pub(crate) fn revisar_archivo(
    destino: &DestinoRevision<'_>,
    modo: Modo,
    inf: &mut Informe,
) -> Result<()> {
    let etiqueta = destino
        .path
        .file_name()
        .map_or_else(String::new, |n| n.to_string_lossy().into_owned());
    let mut bytes = leer_bytes(destino.path)?;
    let mut cambiado = false;

    for (sel, descripcion) in buscar_duplicados(&bytes) {
        if modo == Modo::Revisar {
            inf.errores.push(format!("{etiqueta}: {descripcion}"));
            continue;
        }
        bytes = insert::quitar_duplicados(&bytes, &sel)?;
        cambiado = true;
        inf.reparado.push(format!("{etiqueta}: {descripcion}"));
    }

    if modo == Modo::Reparar {
        if let Some(nuevo) = limpiar_comentario_demo(&bytes, destino.nombre, &etiqueta, inf)? {
            bytes = nuevo;
            cambiado = true;
        }
    } else {
        let raiz = object::parse_first(&texto(&bytes));
        if let Some(root) = raiz {
            let c = root.field("Comment").unwrap_or("");
            if c.contains("DEMO") && !destino.nombre.starts_with("_DEMO") {
                inf.avisos
                    .push(format!("{etiqueta}: Comment '{c}' parece de demo"));
            }
        }
    }

    if cambiado {
        let id =
            escribir_con_historial(destino.ws, destino.path, &bytes, "proyecto doctor reparar")?;
        inf.historial.push(id);
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn should_detect_duplicated_keys_in_configuration() {
        let texto =
            b"PVObject_Root=pvOrient\nFieldTilt=10\nFieldTilt=20\nEnd of PVObject pvOrient\n";
        let dups = buscar_duplicados(texto);
        assert_eq!(dups.len(), 1);
        assert_eq!(dups[0].0, "pvOrient[0]");
        assert!(dups[0].1.contains("FieldTilt"));
    }

    #[test]
    fn should_handle_unclosed_pvorient_absorbing_group_keys() {
        let texto = b"PVObject_=pvVCalcul\n  PVObject_=pvOrient\n    FieldAzim=0.0\n    FieldTilt=10.0\n  FieldTilt=25.0\n  FieldAzim=20.0\nEnd of PVObject pvVCalcul\n";
        let dups = buscar_duplicados(texto);
        assert_eq!(dups.len(), 1);
        assert_eq!(dups[0].0, "pvOrient[0]");
        assert!(dups[0].1.contains("FieldAzim: 0.0 | 20.0"));
        assert!(dups[0].1.contains("FieldTilt: 10.0 | 25.0"));
    }

    #[test]
    fn should_ignore_result_types_and_contd_keys() {
        let texto =
            b"PVObject_Root=pvSimulPlot\nFieldTilt=10\nFieldTilt=20\nEnd of PVObject pvSimulPlot\n";
        assert!(buscar_duplicados(texto).is_empty());
    }

    #[test]
    fn should_ignore_identical_values_and_contd() {
        let texto = b"PVObject_Root=pvSystem\nFlags=1\nFlags=1\nX_contd=2\nX_contd=3\nEnd of PVObject pvSystem\n";
        assert!(buscar_duplicados(texto).is_empty());
    }

    #[test]
    fn should_report_clean_when_no_duplicates_exist() {
        let texto = b"PVObject_Root=pvOrient\nFieldTilt=10\nEnd of PVObject pvOrient\n";
        assert!(buscar_duplicados(texto).is_empty());
    }
}
