//! P50-P90 evaluation set-up by PVsyst: the "Energy management" window adds
//! the evaluation to the variant with PVsyst's own uncertainty defaults (the
//! weather variability comes from the weather source). PVsystCLI then prints
//! the evaluation page in the report (`-rp:p50p90`).

use super::rotulos::{rotulos_p50, RotulosP50};
use super::{abrir_sesion, ProyectoEnPvsyst};
use crate::adapters::gui::conduccion::Formulario;
use crate::adapters::gui::ocr::esta_marcado;
use crate::adapters::gui::rotulos::Idioma;
use crate::adapters::uia;
use crate::config::Paths;
use crate::error::{Error, Result};
use serde_json::{json, Map, Value};

const ROTULOS: RotulosP50 = rotulos_p50(Idioma::Es);

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

/// What the call would do and what it would cost, without opening PVsyst. The
/// window itself spends no trial execution; the report that reads it back does.
pub(crate) fn plan(paths: &Paths, proyecto: &str, variante: &str) -> Value {
    let variante = variante.to_ascii_uppercase();
    let archivo = crate::projects::lifecycle::variantes::ruta_variante(
        &paths.projects_dir(),
        proyecto,
        &variante,
    );
    json!({
        "accion": "p50_p90",
        "dry_run": true,
        "proyecto": proyecto,
        "variante": variante,
        "variant_file": archivo.as_ref().map(|p| p.to_string_lossy().to_string()).ok(),
        "variant_file_exists": archivo.is_ok_and(|p| p.is_file()),
        "requiere_gui": true,
        "plan": [
            crate::idioma::Texto {
                es: "abrir el proyecto en PVsyst y activar la evaluación P50/P90 (ventana Gestión de energía)",
                en: "open the project in PVsyst and enable the P50-P90 evaluation (Energy management window)",
            }
            .en_idioma(paths.idioma),
            crate::idioma::Texto {
                es: "guardar la variante desde PVsyst",
                en: "save the variant from PVsyst",
            }
            .en_idioma(paths.idioma),
        ],
        "costo_estimado": crate::adapters::ledger::costo_estimado(&paths.workspace, 0),
        "siguiente_paso": crate::idioma::Texto {
            es: "repite sin dry_run y después simula con paginas_reporte incluyendo 'p50p90'",
            en: "repeat without dry_run and then simulate with paginas_reporte including 'p50p90'",
        }
        .en_idioma(paths.idioma),
    })
}

/// The evaluation is stored only once «Activo» is ticked; ticking it again would switch it off.
fn activar_evaluacion(energia: &Formulario) -> Result<()> {
    if esta_marcado(energia.hijo(ROTULOS.activo)?) == Some(true) {
        return Ok(());
    }
    energia.elegir(ROTULOS.activo)?;
    std::thread::sleep(std::time::Duration::from_millis(500));
    Ok(())
}

/// Adds the P50-P90 evaluation to `variante` as PVsyst defines it.
pub(crate) fn configurar(paths: &Paths, proyecto: &str, variante: &str) -> Result<Value> {
    let sesion = abrir_sesion(paths)?;
    sesion.ejecutar(|s| {
        let p = ProyectoEnPvsyst::abrir(s, paths, proyecto, variante)?;
        p.ventana.pulsar(ROTULOS.gestion_energia)?;
        let energia = s.esperar(ROTULOS.formulario_energia)?;
        uia::invocar(energia.h, Some(ROTULOS.pestana), Some("tab item"), "seleccionar", None)?;
        std::thread::sleep(std::time::Duration::from_millis(800));
        activar_evaluacion(&energia)?;
        energia.pulsar(ROTULOS.ok)?;
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
