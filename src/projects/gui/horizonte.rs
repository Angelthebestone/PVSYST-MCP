//! Horizon import by PVsyst itself: its horizon window downloads the PVGIS
//! profile for the project site and computes the diffuse and albedo factors.

use super::{abrir_sesion, objeto, ProyectoEnPvsyst};
use crate::config::Paths;
use crate::error::{Error, Result};
use serde_json::{json, Value};
use std::time::{Duration, Instant};

use super::rotulos::{rotulos_horizonte, RotulosHorizonte};
use crate::adapters::gui::rotulos::Idioma;

const ROTULOS: RotulosHorizonte = rotulos_horizonte(Idioma::Es);
/// PVGIS answers in a few seconds; allow for slow networks.
const ESPERA_DESCARGA: Duration = Duration::from_secs(90);

/// Imports the PVGIS horizon of the project site into `variante`.
pub(crate) fn importar_pvgis(paths: &Paths, proyecto: &str, variante: &str) -> Result<Value> {
    let sesion = abrir_sesion(paths)?;
    sesion.ejecutar(|s| {
        let p = ProyectoEnPvsyst::abrir(s, paths, proyecto, variante)?;
        p.ventana.pulsar(ROTULOS.horizonte)?;
        let horizonte = s.esperar(ROTULOS.formulario_horizonte)?;
        horizonte.pulsar(ROTULOS.leer_importar)?;
        let importar = s.esperar(ROTULOS.formulario_importar)?;
        importar.elegir(ROTULOS.opcion_pvgis)?;
        importar.pulsar(ROTULOS.importar_desde_web)?;
        let fin = Instant::now() + ESPERA_DESCARGA;
        while !importar.habilitado(ROTULOS.ok) {
            if Instant::now() >= fin {
                return Err(Error::Gui(
                    "PVsyst no pudo descargar el horizonte de PVGIS".into(),
                ));
            }
            std::thread::sleep(Duration::from_millis(500));
        }
        importar.pulsar(ROTULOS.ok)?;
        horizonte.pulsar(ROTULOS.ok)?;
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
