//! Whether a simulation result can be used at all.
//!
//! PVsystCLI exits 0 and prints "Simulation done in ..." even when the run left
//! per-hour incidents (`ErrorList`), when PVsyst itself warns that the result is
//! incoherent, or when the saved result does not match the array it describes.
//! A performance ratio above 1 is impossible for a real array, so reporting
//! `exito` there hands the caller a figure that must not be used (E115, E126).

use serde_json::{json, Value};

/// PR ceiling: PVsyst's own reports stay below 1, and a saved result above this
/// cannot come from the array it names.
pub(crate) const PR_MAXIMO_PLAUSIBLE_PCT: f64 = 105.0;

/// PVsyst's incoherence warning, in the languages its log and report use.
const MARCAS_INCOHERENCIA: [&str; 3] = [
    "support@pvsyst.com",
    "Incoherencia en los resultados",
    "Incoherence in the",
];

/// PVsyst's summary line of the per-hour incidents of a run.
const MARCA_ERRORLIST: &str = "ErrorList:";

/// `clasificacion` values this module can report.
pub(crate) const ADVERTENCIA: &str = "advertencia";
pub(crate) const FALLO: &str = "fallo";

/// What PVsyst's own output says about the result.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum Coherencia {
    Coherente,
    /// The result exists but PVsyst logged incidents during the run.
    Advertencia(Vec<String>),
    /// The result is physically impossible or PVsyst flagged it as incoherent.
    Incoherente(Vec<String>),
}

impl Coherencia {
    /// `advertencia`, `fallo`, or `None` when there is nothing to report.
    pub(crate) fn clasificacion(&self) -> Option<&'static str> {
        match self {
            Self::Coherente => None,
            Self::Advertencia(_) => Some(ADVERTENCIA),
            Self::Incoherente(_) => Some(FALLO),
        }
    }

    /// Why the result is not coherent, in the order PVsyst reported it.
    pub(crate) fn motivos(&self) -> &[String] {
        match self {
            Self::Coherente => &[],
            Self::Advertencia(m) | Self::Incoherente(m) => m,
        }
    }
}

/// Verdict for the warnings PVsyst logged and the PR of the report it wrote.
pub(crate) fn evaluar(avisos: &[String], pr_pct: Option<f64>) -> Coherencia {
    let mut graves = Vec::new();
    let mut leves = Vec::new();
    for aviso in avisos {
        if MARCAS_INCOHERENCIA.iter().any(|m| aviso.contains(m)) {
            graves.push(aviso.clone());
        } else if aviso.contains(MARCA_ERRORLIST) {
            leves.push(aviso.clone());
        }
    }
    if let Some(pr) = pr_pct.filter(|pr| *pr > PR_MAXIMO_PLAUSIBLE_PCT) {
        graves.push(format!(
            "PR {pr:.1} % is above {PR_MAXIMO_PLAUSIBLE_PCT:.0} %: the result does not match the array it describes"
        ));
    }
    if !graves.is_empty() {
        return Coherencia::Incoherente(graves);
    }
    if !leves.is_empty() {
        return Coherencia::Advertencia(leves);
    }
    Coherencia::Coherente
}

/// Degrades a `simular` result: `success:false` when the result cannot be used,
/// and the verdict under `coherencia` in both cases.
pub(crate) fn aplicar(resultado: &mut Value, avisos: &[String], pr_pct: Option<f64>) {
    let veredicto = evaluar(avisos, pr_pct);
    let Some(clasificacion) = veredicto.clasificacion() else {
        return;
    };
    if let Some(obj) = resultado.as_object_mut() {
        obj.insert("clasificacion".into(), json!(clasificacion));
        obj.insert(
            "coherencia".into(),
            json!({ "clasificacion": clasificacion, "motivos": veredicto.motivos() }),
        );
        if clasificacion == FALLO {
            obj.insert("success".into(), json!(false));
        }
    }
}

/// Notes the verdict of a result read from its report; a read has no `success`
/// to degrade, so only the verdict is added.
pub(crate) fn anotar(resultado: &mut Value, pr_pct: Option<f64>) {
    let veredicto = evaluar(&[], pr_pct);
    let Some(clasificacion) = veredicto.clasificacion() else {
        return;
    };
    resultado["coherencia"] = json!({
        "clasificacion": clasificacion,
        "motivos": veredicto.motivos(),
    });
}

/// PR of the report at `ruta`, when it is a PVsyst report this run wrote.
pub(crate) fn pr_del_informe(ruta: &str) -> Option<f64> {
    let bytes = std::fs::read(ruta).ok()?;
    let tokens = super::report_pdf::tokens(&bytes);
    super::report_pdf::resumen(&tokens)
        .ok()?
        .get("pr_pct")?
        .as_f64()
}
