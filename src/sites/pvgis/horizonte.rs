//! PVGIS horizon of the project site: imported by PVsyst's horizon window by
//! default, or computed by the MCP with `via: "aproximacion"`.

use super::{adquisicion, normalizacion, persistencia};
use crate::args::{opt_str, req_str, workspace_of};
use crate::config::Paths;
use crate::error::{Error, Result};
use crate::projects::resolve_project_file;
use serde_json::{json, Value};

/// The MCP's own horizon factors (`FHHDiffus` as the mean of cos² h).
const APROXIMACION_HORIZONTE: crate::aproximaciones::Aproximacion =
    crate::aproximaciones::Aproximacion {
        funcion: "sitio horizonte_pvgis via aproximacion (FHHDiffus calculado por el MCP)",
        alternativa: crate::idioma::Texto {
            es: "sitio horizonte_pvgis sin 'via': PVsyst importa PVGIS y calcula los factores",
            en:
                "sitio horizonte_pvgis without 'via': PVsyst imports PVGIS and computes the factors",
        },
    };

/// PVGIS horizon of the project site: imported by PVsyst's horizon window by
/// default, or by the MCP with `via: "aproximacion"`.
pub(crate) fn horizonte(paths: &Paths, args: &Value) -> Result<Value> {
    crate::aproximaciones::exigir_ruta_pvsyst_sin_workspace(
        paths,
        args,
        crate::idioma::Texto {
            es: "El horizonte PVGIS",
            en: "The PVGIS horizon",
        },
    )?;
    rechazar_dry_run(paths, args)?;
    let proyecto = req_str(args, &["proyecto"], "")?;
    let proyecto = proyecto.strip_suffix(".PRJ").unwrap_or(proyecto);
    let variante = opt_str(args, &["variante"]).unwrap_or("VC0");
    crate::aproximaciones::segun_via(
        paths,
        args,
        &APROXIMACION_HORIZONTE,
        (
            || crate::projects::gui::horizonte::importar_pvgis(paths, proyecto, variante),
            || horizonte_pvgis(paths, args),
        ),
    )
}

/// This route has no rehearsal: it opens PVsyst's horizon window and downloads
/// the profile. The refusal states what the real call would cost.
fn rechazar_dry_run(paths: &Paths, args: &Value) -> Result<()> {
    if !crate::args::flag(args, &["dry_run"]) {
        return Ok(());
    }
    let costo = crate::adapters::ledger::costo_estimado(&paths.workspace, 0);
    let restantes = costo["restantes"].to_string();
    let espera = crate::projects::gui::horizonte::ESPERA_DESCARGA
        .as_secs()
        .to_string();
    Err(Error::Invalid(
        crate::idioma::Texto {
            es: "'horizonte_pvgis' no tiene ensayo: se ejecutaría de verdad. Coste: ninguna ejecución de prueba (quedan {restantes}), pero abre la ventana de horizonte de PVsyst y espera hasta {espera} s la descarga de PVGIS. Para ver el plan sin ejecutarlo, llama con 'via':'aproximacion' y dry_run:true.",
            en: "'horizonte_pvgis' has no dry run: it would run for real. Cost: no trial executions ({restantes} left), but it opens PVsyst's horizon window and waits up to {espera} s for the PVGIS download. To see the plan without running it, call it with 'via':'aproximacion' and dry_run:true.",
        }
        .texto_con(paths.idioma, &[("restantes", &restantes), ("espera", &espera)]),
    ))
}

/// Fill the variant's `pvHorizon` with the PVGIS horizon profile of the project site.
fn horizonte_pvgis(paths: &Paths, args: &Value) -> Result<Value> {
    let ws = workspace_of(paths, args);
    let proyecto = req_str(args, &["proyecto"], "")?;
    let variante = opt_str(args, &["variante"]).unwrap_or("VC0");
    let prj = resolve_project_file(&ws, proyecto)?;
    let var = prj.with_extension(variante);
    if !var.exists() {
        return Err(Error::NotFound(format!(
            "Variante '{variante}' no encontrada: {}",
            var.display()
        )));
    }
    let (lat, lon) = persistencia::sitio_prj(&prj)?;
    let todos = adquisicion::descargar_perfil_horizonte(lat, lon)?;
    let total = todos.len();
    let puntos = normalizacion::diezmar(todos);
    let fhh = normalizacion::calcular_fhh(&puntos);

    let peticion = persistencia::PeticionHorizonte {
        paths,
        ws: &ws,
        var: &var,
        proyecto,
        variante,
        lat,
        lon,
        fhh,
        puntos: &puntos,
    };
    let id = persistencia::aplicar_horizonte(&peticion)?;

    Ok(json!({
        "ok": true,
        "variante": var.display().to_string(),
        "puntos": puntos.len(),
        "puntos_pvgis": total,
        "FHHDiffus": fhh,
        "factores_aproximados": true,
        "historial_id": id,
    }))
}
