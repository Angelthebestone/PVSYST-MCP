//! Pure domain logic for PVsyst `.RVT` result-variable templates.
//!
//! Validates variable identifiers against the complete set of result variables
//! written by PVsyst 8.1.7 in real `.RVT` files, result CSV headers and variant
//! files, avoiding fabricated or invalid names.

use crate::error::{Error, Result};

pub(crate) const PREFIX_RESULT_VARS: &str = "ResultVars=";
const SEPARADOR_VARS: char = ';';

/// Exhaustive list of 122 verified result variables from real PVsyst 8.1.7
/// `Demo_PVsystCLI_BatchVariables.RVT`, CSV headers and simulation definitions.
pub(crate) const VARIABLES_VALIDAS: &[&str] = &[
    "AlbTrp",
    "Alb_Eff",
    "Alb_Gl",
    "Alb_Inc",
    "AngInc",
    "AngProf",
    "ArrayON",
    "AvgOpCost_yr",
    "AzSol",
    "BatteryCost",
    "BeamEff",
    "BeamHor",
    "BeamInc",
    "BeamTrp",
    "Bm_Gl",
    "CircEff",
    "CircInc",
    "CircTrp",
    "DTArr",
    "DTArrGl",
    "DifA_Gl",
    "DifITrp",
    "DifSInc",
    "DifS_Gl",
    "DiffEff",
    "DiffHor",
    "EArrMPP",
    "EArrNom",
    "EArrRef",
    "EArray",
    "EDirUse",
    "EOutInv",
    "EUseful",
    "E_Grid",
    "EffArrC",
    "EffArrR",
    "EffInvB",
    "EffInvR",
    "EffSysC",
    "EffSysR",
    "FEffAlb",
    "FEffDif",
    "FHrzAlb",
    "FHrzBm",
    "FHrzCrc",
    "FHrzDif",
    "FHrzGl",
    "FIAMAlb",
    "FIAMBm",
    "FIAMCrc",
    "FIAMDif",
    "FIAMGl",
    "FIAMShd",
    "FShdAlb",
    "FShdBm",
    "FShdCrc",
    "FShdDif",
    "FShdGl",
    "FTransp",
    "GIncLss",
    "GincThr",
    "GlobEff",
    "GlobHor",
    "GlobHrz",
    "GlobIAM",
    "GlobInc",
    "GlobShd",
    "GlobTrp",
    "HBndTrp",
    "HSol",
    "HrzALss",
    "HrzBLss",
    "HrzCLss",
    "HrzDLss",
    "HrzLoss",
    "IAMALss",
    "IAMBLss",
    "IAMCLss",
    "IAMDLss",
    "IAMLoss",
    "IArray",
    "IL_Imax",
    "IL_Oper",
    "IL_Pmax",
    "IL_Pmin",
    "IL_Vmax",
    "IL_Vmin",
    "IRR",
    "InvLoss",
    "InverterCost",
    "LCOE",
    "Lc",
    "Lcr",
    "Ls",
    "Lsr",
    "MisLoss",
    "ModQual",
    "ModuleCost",
    "NPV",
    "OhmLoss",
    "OwnFunds",
    "PR",
    "PaybackPeriod",
    "ROI",
    "ShdALss",
    "ShdBLss",
    "ShdCLss",
    "ShdDLss",
    "ShdLoss",
    "Syst_ON",
    "TArrWtd",
    "TArray",
    "TExtON",
    "T_Amb",
    "TempLss",
    "TotInstCost",
    "TotLoan",
    "UArray",
    "WindVel",
    "Ya",
    "Yf",
    "Yr",
];

/// Returns the canonical variable name if valid, or `None` if unknown.
pub(crate) fn buscar_variable_valida(nombre: &str) -> Option<&'static str> {
    let limpio = nombre.trim();
    VARIABLES_VALIDAS
        .iter()
        .copied()
        .find(|v| v.eq_ignore_ascii_case(limpio))
}

/// Parses the variable names from `.RVT` text content.
pub(crate) fn parse_rvt(texto: &str) -> Result<Vec<String>> {
    let linea = texto
        .lines()
        .map(str::trim)
        .find(|l| !l.is_empty())
        .unwrap_or("");
    let contenido = if let Some(idx) = linea.to_ascii_lowercase().find("resultvars=") {
        &linea[idx + PREFIX_RESULT_VARS.len()..]
    } else {
        linea
    };
    let vars: Vec<String> = contenido
        .split(SEPARADOR_VARS)
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .map(str::to_string)
        .collect();
    Ok(vars)
}

/// Appends valid variables to an existing list, rejecting unknowns.
pub(crate) fn anadir_variables(actuales: &[String], nuevas: &[&str]) -> Result<Vec<String>> {
    let mut resultado = actuales.to_vec();
    for var in nuevas {
        let canonica = buscar_variable_valida(var).ok_or_else(|| {
            Error::Invalid(format!(
                "Variable '{var}' desconocida; no es una variable de resultado de PVsyst"
            ))
        })?;
        if !resultado.iter().any(|v| v.eq_ignore_ascii_case(canonica)) {
            resultado.push(canonica.to_string());
        }
    }
    Ok(resultado)
}

/// Removes variables from an existing list, rejecting missing ones.
pub(crate) fn eliminar_variables(actuales: &[String], a_eliminar: &[&str]) -> Result<Vec<String>> {
    for var in a_eliminar {
        let existe = actuales.iter().any(|v| v.eq_ignore_ascii_case(var.trim()));
        if !existe {
            return Err(Error::NotFound(format!(
                "Variable '{var}' no encontrada en la plantilla RVT"
            )));
        }
    }
    let resultado = actuales
        .iter()
        .filter(|v| !a_eliminar.iter().any(|d| d.eq_ignore_ascii_case(v)))
        .cloned()
        .collect();
    Ok(resultado)
}

/// Reorders variables according to `nuevo_orden`, ensuring all variables match.
pub(crate) fn reordenar_variables(
    actuales: &[String],
    nuevo_orden: &[&str],
) -> Result<Vec<String>> {
    if nuevo_orden.len() != actuales.len() {
        return Err(Error::Invalid(format!(
            "Cantidad de variables para reordenar ({}) no coincide con la actual ({})",
            nuevo_orden.len(),
            actuales.len()
        )));
    }
    let mut reordenadas = Vec::with_capacity(nuevo_orden.len());
    for var in nuevo_orden {
        let canonica = buscar_variable_valida(var).ok_or_else(|| {
            Error::Invalid(format!(
                "Variable '{var}' desconocida; no es una variable de resultado de PVsyst"
            ))
        })?;
        let presente = actuales.iter().any(|v| v.eq_ignore_ascii_case(canonica));
        if !presente {
            return Err(Error::NotFound(format!(
                "Variable '{var}' no está presente en la plantilla actual"
            )));
        }
        if !reordenadas
            .iter()
            .any(|v: &String| v.eq_ignore_ascii_case(canonica))
        {
            reordenadas.push(canonica.to_string());
        }
    }
    if reordenadas.len() != actuales.len() {
        return Err(Error::Invalid(
            "La lista de reordenación contiene variables duplicadas".into(),
        ));
    }
    Ok(reordenadas)
}

/// Serializes variables into the canonical single-line `.RVT` format.
pub(crate) fn serializar_rvt(variables: &[String]) -> String {
    format!("  ResultVars={}\n", variables.join(";"))
}

#[cfg(test)]
#[path = "../../tests/unit/plantilla_variables/dominio.rs"]
mod tests;
