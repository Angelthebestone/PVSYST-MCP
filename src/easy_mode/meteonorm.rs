//! Meteonorm data: PVsyst generates synthetic hourly weather with Meteonorm 9
//! (`-swg`) and needs its station database for that. An installation without it
//! fails the simulation after ~11 s and one trial execution, so the check runs
//! before the run starts.

use crate::config::{data_ro, Paths};
use crate::error::{Error, Result};
use crate::idioma::{Idioma, Texto};
use std::path::{Path, PathBuf};

/// Folder and file PVsyst looks for when it initialises Meteonorm.
const CARPETA_DATOS: &str = "DataMN9";
const BASE_ESTACIONES: &str = "station.db.enc";

/// Station database of the Meteonorm data shipped in `base`.
fn estaciones_en(base: &Path) -> PathBuf {
    base.join(CARPETA_DATOS).join(BASE_ESTACIONES)
}

/// Folders that may hold the Meteonorm data of the PVsyst installation at
/// `paths`: PVsyst ships it beside its executables, and the read-only data
/// folder may carry it too. `None` when no installation is visible, so there is
/// nothing to judge and the run itself reports what it finds.
pub(crate) fn bases_de_instalacion(paths: &Paths) -> Option<Vec<PathBuf>> {
    if !paths.gui_exe.is_file() {
        return None;
    }
    let instalacion = paths.gui_exe.parent()?.to_path_buf();
    let ro = data_ro();
    let mut bases = vec![instalacion];
    if !bases.contains(&ro) {
        bases.push(ro);
    }
    Some(bases)
}

/// Whether the Meteonorm station database is in one of `bases`.
pub(crate) fn esta_disponible(bases: &[PathBuf]) -> bool {
    bases.iter().any(|b| estaciones_en(b).is_file())
}

/// Rejects a synthetic-weather run when the installation carries no Meteonorm
/// data: PVsyst would spend a trial execution to say exactly that.
pub(crate) fn exigir_si_sintetica(
    bases: Option<&[PathBuf]>,
    sintetica: bool,
    idioma: Idioma,
) -> Result<()> {
    if !sintetica {
        return Ok(());
    }
    let Some(bases) = bases else {
        return Ok(());
    };
    if esta_disponible(bases) {
        return Ok(());
    }
    let buscado: Vec<String> = bases
        .iter()
        .map(|b| estaciones_en(b).display().to_string())
        .collect();
    Err(Error::Invalid(
        Texto {
            es: "'synthetic_weather' necesita Meteonorm: PVsyst genera la meteorología horaria sintética con Meteonorm 9 y no hay base de estaciones en {buscado}. Pasa synthetic_weather:false con un sitio que ya tenga un .MET horario, o instala Meteonorm; si no, la simulación fallaría tras gastar una ejecución de prueba.",
            en: "'synthetic_weather' needs Meteonorm: PVsyst generates the synthetic hourly weather with Meteonorm 9 and no station database was found at {buscado}. Pass synthetic_weather:false with a site that already has an hourly .MET, or install Meteonorm; the run would otherwise fail after spending a trial execution.",
        }
        .texto_con(idioma, &[("buscado", &buscado.join(" / "))]),
    ))
}

#[cfg(test)]
#[path = "../../tests/unit/easy_mode/meteonorm.rs"]
mod tests;
