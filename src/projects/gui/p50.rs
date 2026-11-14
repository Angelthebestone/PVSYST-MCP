//! P50-P90 evaluation set-up by PVsyst: the "Energy management" window adds
//! the evaluation to the variant with PVsyst's own uncertainty defaults (the
//! weather variability comes from the weather source). PVsystCLI then prints
//! the evaluation page in the report (`-rp:p50p90`).

use super::rotulos::FORMULARIO_ENERGIA;
use super::{abrir_sesion, ProyectoEnPvsyst};
use crate::adapters::gui::conduccion::{Campo, Formulario};
use crate::adapters::gui::ocr::esta_marcado;
use crate::adapters::gui::rotulos::dialogos::OK;
use crate::adapters::gui::rotulos::pvsyst::GESTION_ENERGIA;
use crate::config::Paths;
use crate::error::{Error, Result};
use crate::idioma::{Idioma, Texto};
use serde_json::{json, Map, Value};
use std::time::Duration;

/// Page "P50 - P90" of the energy window: the third one, by index, so the tab
/// caption (which PVsyst translates) is never read. Measured on 8.1.8 (QA 2026-11-11).
const PAGINA_P50: usize = 2;
/// The «Activo» check box, found by class and position and not by its caption: the
/// first control of that page, key `checkbox_134_34` of `gui_observar que:vista`
/// (row 134, column 34 at 96 DPI). Measured on 8.1.8 (QA 2026-11-11).
const CASILLA_ACTIVO: Campo = Campo {
    clase: "TCheckBox",
    x: 34,
    y: 134,
};
/// A control only that page has: the radio button «Probabilidad», key
/// `radiobutton_364_633` (row 364, column 633). The order of the tabs changes between
/// PVsyst versions, so the page is confirmed by what it holds, never by its caption.
const MARCA_PAGINA_P50: Campo = Campo {
    clase: "TRadioButton",
    x: 633,
    y: 364,
};
/// Lets the page build its controls and the form react to the click.
const PAUSA_PAGINA: Duration = Duration::from_millis(400);
const PAUSA_CASILLA: Duration = Duration::from_millis(500);

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
        "costo_estimado": crate::adapters::ledger::costo_estimado(paths, 0),
        "siguiente_paso": crate::idioma::Texto {
            es: "repite sin dry_run y después simula con paginas_reporte incluyendo 'p50p90'",
            en: "repeat without dry_run and then simulate with paginas_reporte including 'p50p90'",
        }
        .en_idioma(paths.idioma),
    })
}

/// Shows the P50-P90 page: the usual index first, then the other tabs when the
/// order is not the one measured.
fn mostrar_pagina_p50(energia: &Formulario) -> Result<()> {
    let candidatas =
        std::iter::once(PAGINA_P50).chain((0..energia.paginas()?).filter(|i| *i != PAGINA_P50));
    for indice in candidatas {
        energia.pagina(indice)?;
        std::thread::sleep(PAUSA_PAGINA);
        if energia.control(MARCA_PAGINA_P50).is_ok() {
            return Ok(());
        }
    }
    Err(Error::Gui(
        Texto {
            es: "Ninguna pestaña de '{titulo}' es la de P50-P90: ¿cambió la versión de PVsyst? Lee gui_observar que:inventario para ver sus pestañas",
            en: "No tab of '{titulo}' is the P50-P90 one: did the PVsyst version change? Read gui_observar que:inventario to see its tabs",
        }
        .texto_con(Idioma::de_entorno(), &[("titulo", &energia.titulo())]),
    ))
}

/// The evaluation is stored only once «Activo» is ticked; ticking it again would switch it off.
fn activar_evaluacion(energia: &Formulario) -> Result<()> {
    let casilla = energia.control(CASILLA_ACTIVO)?;
    if esta_marcado(casilla) == Some(true) {
        return Ok(());
    }
    energia.marcar_en(casilla);
    std::thread::sleep(PAUSA_CASILLA);
    Ok(())
}

/// Adds the P50-P90 evaluation to `variante` as PVsyst defines it.
pub(crate) fn configurar(paths: &Paths, proyecto: &str, variante: &str) -> Result<Value> {
    let sesion = abrir_sesion(paths)?;
    sesion.ejecutar(|s| {
        let p = ProyectoEnPvsyst::abrir(s, paths, proyecto, variante)?;
        p.ventana.pulsar(GESTION_ENERGIA)?;
        let energia = s.esperar(FORMULARIO_ENERGIA)?;
        mostrar_pagina_p50(&energia)?;
        activar_evaluacion(&energia)?;
        energia.pulsar(OK)?;
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
