//! Horizon block neutralization: zeroes profile points and clears Meteonorm references.

use crate::error::{Error, Result};
use crate::format::edit;

pub(crate) const COMENTARIO_HORIZONTE_VACIO: &str =
    "Horizonte vacío (sin obstrucciones): usa sitio accion:horizonte_pvgis o edita los puntos";

/// Clears a copied `pvHorizon` (demo data is Meteonorm) so no Meteonorm data reaches the target.
/// The 100 `Point_k` lines stay (zeroed) to maintain PVsyst's padding.
pub(crate) fn neutralizar_horizonte(bloque: &[u8]) -> Result<Vec<u8>> {
    let spans = edit::spans_of(bloque);
    let si = edit::find_span(&spans, "pvHorizon")
        .ok_or_else(|| Error::Format("El bloque copiado no es un pvHorizon".into()))?;
    let objeto = &spans[si];
    let texto = String::from_utf8_lossy(bloque).into_owned();
    let mut cambios: Vec<(String, Option<String>)> = texto
        .split('\n')
        .enumerate()
        .filter(|(i, _)| *i > objeto.open && *i < objeto.close)
        .filter_map(|(_, l)| l.trim().split_once('=').map(|(k, _)| k.trim().to_string()))
        .filter(|k| k.starts_with("Point_"))
        .map(|k| (k, Some("0.0,0.0".to_string())))
        .collect();
    cambios.push(("NPtsEff".into(), Some("0".into())));
    let (sin_puntos, _) = edit::edit_fields(bloque, "pvHorizon", &cambios)?;
    let cabecera: Vec<(String, Option<String>)> = vec![
        ("Comment".into(), Some(COMENTARIO_HORIZONTE_VACIO.into())),
        ("FHHDiffus".into(), Some("1.000".into())),
        ("FHPDiffus".into(), Some("1.000".into())),
        ("FHPAlbedo".into(), Some("1.000".into())),
    ];
    let (neutro, _) = edit::edit_fields(&sin_puntos, "pvHorizon", &cabecera)?;
    if String::from_utf8_lossy(&neutro)
        .to_ascii_lowercase()
        .contains("meteonorm")
    {
        return Err(Error::Invalid(
            "El horizonte copiado aún contiene datos de Meteonorm".into(),
        ));
    }
    Ok(neutro)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn demo(nombre: &str) -> Vec<u8> {
        std::fs::read(crate::config::proyectos_demo_ro().join(nombre)).expect("PVsyst demo data")
    }

    #[test]
    #[ignore = "needs the PVsyst 8.1.7 demo data: run with --ignored"]
    fn should_neutralize_horizon_removing_meteonorm_data() {
        let fuente = demo("_DEMO_Residential.VC1");
        let bloque = crate::format::insert::extraer_objeto(&fuente, "pvHorizon").expect("bloque");
        let neutro = neutralizar_horizonte(&bloque).expect("neutralizado");
        let texto = String::from_utf8_lossy(&neutro).to_ascii_lowercase();
        assert!(!texto.contains("meteonorm"));
        assert!(texto
            .lines()
            .filter(|l| l.trim().starts_with("point_"))
            .all(|l| l.trim().ends_with("=0.0,0.0")));
        assert!(texto.contains("nptseff=0"));
        assert!(texto.contains("fhhdiffus=1.000"));
    }

    #[test]
    #[ignore = "needs the PVsyst 8.1.7 demo data: run with --ignored"]
    fn should_replace_points_in_place_preserving_count() {
        let fuente = demo("_DEMO_Residential.VC1");
        let bloque = crate::format::insert::extraer_objeto(&fuente, "pvHorizon").expect("bloque");
        let neutro = neutralizar_horizonte(&bloque).expect("neutralizado");
        let mut cambios: Vec<(String, Option<String>)> = (1..=3)
            .map(|k| (format!("Point_{k}"), Some(format!("{k}.0,5.0"))))
            .collect();
        cambios.push(("NPtsEff".into(), Some("3".into())));
        let (con_puntos, _) = edit::edit_fields(&neutro, "pvHorizon", &cambios).expect("editar");
        let lineas_antes = neutro.split(|b| *b == b'\n').count();
        let lineas_despues = con_puntos.split(|b| *b == b'\n').count();
        assert_eq!(lineas_antes, lineas_despues, "ninguna clave nueva");
        let texto = String::from_utf8_lossy(&con_puntos).into_owned();
        let nodo = crate::format::object::parse_first(&texto).expect("parseable");
        assert_eq!(nodo.find_deep("Point_3"), Some("3.0,5.0"));
        assert_eq!(nodo.find_deep("Point_4"), Some("0.0,0.0"));
    }
}
