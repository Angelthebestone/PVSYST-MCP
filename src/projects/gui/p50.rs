//! P50-P90 evaluation set-up by PVsyst: the "Energy management" window adds
//! the evaluation to the variant with PVsyst's own uncertainty defaults (the
//! weather variability comes from the weather source). PVsystCLI then prints
//! the evaluation page in the report (`-rp:p50p90`).

use super::ProyectoEnPvsyst;
use crate::adapters::gui::sesion_pvsyst::Sesion;
use crate::adapters::uia;
use crate::config::Paths;
use crate::error::{Error, Result};
use serde_json::{json, Map, Value};

const PESTANA: &str = "Estimación P50 - P90";
/// Fields PVsyst stores for the evaluation; they can be edited later with `editar`.
const CAMPOS: [&str; 10] = [
    "MeteoKind",
    "MeteoAnnualVariab",
    "PVModelVariab",
    "InverterVariab",
    "SoilingVariab",
    "DegradVariab",
    "P90Custom",
    "PxxCustom",
    "PxxReport",
    "PxxMainResults",
];

/// Adds the P50-P90 evaluation to `variante` as PVsyst defines it.
pub(crate) fn configurar(paths: &Paths, proyecto: &str, variante: &str) -> Result<Value> {
    let sesion = Sesion::abrir(paths)?;
    sesion.ejecutar(|s| {
        let p = ProyectoEnPvsyst::abrir(s, paths, proyecto, variante)?;
        p.ventana.pulsar("Gestión de la energía")?;
        let energia = s.esperar("TSystemeAddForm")?;
        uia::invocar(energia.h, Some(PESTANA), Some("tab item"), "seleccionar", None)?;
        std::thread::sleep(std::time::Duration::from_millis(800));
        energia.pulsar("OK")?;
        s.aceptar_mensajes();
        let raiz = p.guardar_variante()?;
        let guardado: Map<String, Value> = CAMPOS
            .iter()
            .filter_map(|k| raiz.find_deep(k).map(|v| ((*k).to_string(), json!(v))))
            .collect();
        if guardado.is_empty() {
            return Err(Error::Gui(
                "PVsyst no guardó la evaluación P50-P90 (no disponible en sistemas aislados y de bombeo)".into(),
            ));
        }
        Ok(json!({
            "origen": "pvsyst",
            "p50_p90": guardado,
            "avisos_pvsyst": s.mensajes(),
            "siguiente_paso": "simular con paginas_reporte incluyendo 'p50p90' y leer con resultados que:p50_p90",
        }))
    })
}
