//! Horizon import by PVsyst itself: its horizon window downloads the PVGIS
//! profile for the project site and computes the diffuse and albedo factors.

use super::{objeto, ProyectoEnPvsyst};
use crate::adapters::gui::sesion_pvsyst::Sesion;
use crate::config::Paths;
use crate::error::{Error, Result};
use serde_json::{json, Value};
use std::time::{Duration, Instant};

/// PVGIS answers in a few seconds; allow for slow networks.
const ESPERA_DESCARGA: Duration = Duration::from_secs(90);

/// Imports the PVGIS horizon of the project site into `variante`.
pub(crate) fn importar_pvgis(paths: &Paths, proyecto: &str, variante: &str) -> Result<Value> {
    let sesion = Sesion::abrir(paths)?;
    sesion.ejecutar(|s| {
        let p = ProyectoEnPvsyst::abrir(s, paths, proyecto, variante)?;
        p.ventana.pulsar("Horizonte")?;
        let horizonte = s.esperar("THorizonForm")?;
        horizonte.pulsar("Leer / Importar")?;
        let importar = s.esperar("TImportHorizonForm")?;
        importar.elegir("PVGIS")?;
        importar.pulsar("Importar desde web")?;
        let fin = Instant::now() + ESPERA_DESCARGA;
        while !importar.habilitado("OK") {
            if Instant::now() >= fin {
                return Err(Error::Gui(
                    "PVsyst no pudo descargar el horizonte de PVGIS".into(),
                ));
            }
            std::thread::sleep(Duration::from_millis(500));
        }
        importar.pulsar("OK")?;
        horizonte.pulsar("OK")?;
        s.aceptar_mensajes();
        let raiz = p.guardar_variante()?;
        let h = objeto(&raiz, "pvHorizon")
            .ok_or_else(|| Error::Format("PVsyst no guardó el horizonte".into()))?;
        let campo = |k: &str| h.field(k);
        Ok(json!({
            "origen": "pvsyst",
            "fuente": "PVGIS (importación de PVsyst)",
            "comentario": campo("Comment"),
            "FHHDiffus": campo("FHHDiffus"),
            "FHPDiffus": campo("FHPDiffus"),
            "FHPAlbedo": campo("FHPAlbedo"),
            "puntos": h.find_deep("NPtsEff"),
            "avisos_pvsyst": s.mensajes(),
        }))
    })
}
