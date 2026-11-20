//! Horizon import by PVsyst itself: its horizon window downloads the PVGIS
//! profile for the project site and computes the diffuse and albedo factors.

use super::{abrir_sesion, objeto, ProyectoEnPvsyst};
use crate::config::Paths;
use crate::error::{Error, Result};
use crate::format::object::PvNode;
use serde_json::{json, Value};
use std::time::{Duration, Instant};

use super::rotulos::{FORMULARIO_HORIZONTE, FORMULARIO_IMPORTAR_HORIZONTE};
use crate::adapters::gui::rotulos::dialogos::OK;
use crate::adapters::gui::rotulos::pvsyst::{
    HORIZONTE, IMPORTAR_DESDE_WEB, LEER_IMPORTAR, OPCION_PVGIS,
};

/// PVGIS answers in a few seconds; allow for slow networks.
pub(crate) const ESPERA_DESCARGA: Duration = Duration::from_secs(90);

/// Imports the PVGIS horizon of the project site into `variante`.
pub(crate) fn importar_pvgis(paths: &Paths, proyecto: &str, variante: &str) -> Result<Value> {
    let sesion = abrir_sesion(paths)?;
    sesion.ejecutar(|s| {
        let p = ProyectoEnPvsyst::abrir(s, paths, proyecto, variante)?;
        p.ventana.pulsar(HORIZONTE)?;
        let horizonte = s.esperar(FORMULARIO_HORIZONTE)?;
        horizonte.pulsar(LEER_IMPORTAR)?;
        let importar = s.esperar(FORMULARIO_IMPORTAR_HORIZONTE)?;
        importar.elegir(OPCION_PVGIS)?;
        importar.pulsar(IMPORTAR_DESDE_WEB)?;
        let fin = Instant::now() + ESPERA_DESCARGA;
        while !importar.habilitado(OK) {
            if Instant::now() >= fin {
                return Err(Error::Gui(
                    "PVsyst no pudo descargar el horizonte de PVGIS".into(),
                ));
            }
            std::thread::sleep(Duration::from_millis(500));
        }
        importar.pulsar(OK)?;
        horizonte.pulsar(OK)?;
        s.aceptar_mensajes();
        let raiz = p.guardar_variante()?;
        respuesta(&raiz, &s.mensajes())
    })
}

/// Shading factors PVsyst computes from an imported horizon.
const FACTORES: [&str; 3] = ["FHHDiffus", "FHPDiffus", "FHPAlbedo"];

/// The horizon as saved; a variant without points or without any factor means PVsyst
/// did not import it, which is an error rather than a reply full of `null`.
fn respuesta(raiz: &PvNode, avisos: &[String]) -> Result<Value> {
    let h = objeto(raiz, "pvHorizon")
        .ok_or_else(|| Error::Format("PVsyst no guardó el horizonte".into()))?;
    let puntos = h.find_deep("NPtsEff");
    let sin_factores = FACTORES.iter().all(|k| h.field(k).is_none());
    if puntos.is_none() || sin_factores {
        return Err(Error::Gui(
            "PVsyst guardó la variante sin el horizonte importado (sin puntos o sin factores)"
                .into(),
        ));
    }
    let mut salida = json!({
        "origen": "pvsyst",
        "fuente": "PVGIS (importación de PVsyst)",
        "comentario": h.field("Comment"),
        "puntos": puntos,
        "avisos_pvsyst": avisos,
    });
    for k in FACTORES {
        salida[k] = json!(h.field(k));
    }
    Ok(salida)
}

#[cfg(test)]
#[path = "../../../tests/unit/projects/gui/horizonte.rs"]
mod tests;
