//! Cache key derivation from the project, variant and referenced component files.

use crate::components::{resolve_component, ConsultaComponente};
use crate::error::{Error, Result};
use crate::projects::resolve_project_file;
use std::fs;
use std::path::{Path, PathBuf};

pub(super) const OFFSET_FNV: u64 = 0xcbf2_9ce4_8422_2325;
pub(super) const PRIMO_FNV: u64 = 0x0000_0100_0000_01b3;
pub(super) const OPCIONES_SALIDA: [&str; 2] = ["-ocf", "-rpf"];
pub(super) const OPCIONES_ENTRADA: [&str; 7] =
    ["-bpf", "-brf", "-imf", "-isf", "-ipf", "-cof", "-irf"];

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum ClaveCache {
    Valida(String),
    Inutilizable { razon: String },
}

pub(super) enum LecturaReferencia {
    Ok(Vec<u8>),
    NotFound(String),
    IoError(String),
}

pub(super) fn absorber(hash: u64, data: &[u8]) -> u64 {
    data.iter()
        .fold(hash, |h, b| (h ^ u64::from(*b)).wrapping_mul(PRIMO_FNV))
}

pub(super) fn etiquetar(hash: u64, rotulo: &str, data: &[u8]) -> u64 {
    let h = absorber(hash, rotulo.as_bytes());
    let h = absorber(h, &(data.len() as u64).to_le_bytes());
    absorber(h, data)
}

pub(super) fn valor<'a>(cli_args: &'a [String], flag: &str) -> Option<&'a str> {
    let prefijo = format!("{flag}:");
    cli_args
        .iter()
        .find_map(|a| a.strip_prefix(prefijo.as_str()))
}

pub(super) fn argumentos_clave(cli_args: &[String]) -> Vec<String> {
    let mut salida = Vec::new();
    let mut saltar_valor = false;
    for arg in cli_args {
        if saltar_valor {
            saltar_valor = false;
            continue;
        }
        if OPCIONES_SALIDA.contains(&arg.as_str()) {
            saltar_valor = true;
            continue;
        }
        if OPCIONES_SALIDA
            .iter()
            .any(|o| arg.starts_with(&format!("{o}:")))
        {
            continue;
        }
        salida.push(arg.clone());
    }
    salida.sort();
    salida
}

pub(super) fn referencias(texto: &str) -> Vec<(String, String)> {
    texto
        .lines()
        .filter_map(|linea| {
            let (_, v) = linea.trim().split_once('=')?;
            let v = v.trim();
            let ext = Path::new(v).extension()?.to_str()?.to_ascii_uppercase();
            matches!(ext.as_str(), "SIT" | "MET" | "PAN" | "OND").then(|| (ext, v.to_owned()))
        })
        .collect()
}

pub(super) fn leer_referencia(workspace: &Path, ext: &str, nombre: &str) -> LecturaReferencia {
    let ruta = match ext {
        "SIT" => workspace.join("Sites").join(nombre),
        "MET" => workspace.join("Meteo").join(nombre),
        _ => match resolve_component(workspace, &ConsultaComponente::referencia(ext, nombre)) {
            Ok(p) => p,
            Err(e) => {
                return LecturaReferencia::NotFound(format!(
                    "Componente '{nombre}' ({ext}) no encontrado: {e}"
                ))
            }
        },
    };
    if !ruta.exists() {
        return LecturaReferencia::NotFound(format!(
            "Referencia no encontrada: '{nombre}' ({ext}) en '{}'",
            ruta.display()
        ));
    }
    match fs::read(&ruta) {
        Ok(b) => LecturaReferencia::Ok(b),
        Err(e) => LecturaReferencia::IoError(format!(
            "Error de E/S leyendo referencia '{nombre}' ({ext}) en '{}': {e}",
            ruta.display()
        )),
    }
}

pub(super) fn leer_opcion(workspace: &Path, opcion: &str, v: &str) -> LecturaReferencia {
    let cand_direct = PathBuf::from(v);
    let cand_ws = workspace.join(v);
    let ruta = if cand_direct.is_file() {
        Some(cand_direct)
    } else if cand_ws.is_file() {
        Some(cand_ws)
    } else {
        None
    };
    let Some(p) = ruta else {
        return LecturaReferencia::NotFound(format!(
            "Archivo de opción '{opcion}' no encontrado: '{v}'"
        ));
    };
    match fs::read(&p) {
        Ok(b) => LecturaReferencia::Ok(b),
        Err(e) => LecturaReferencia::IoError(format!(
            "Error de E/S leyendo archivo de opción '{opcion}' en '{}': {e}",
            p.display()
        )),
    }
}

fn hashear_referencias(
    workspace: &Path,
    referidos: Vec<(String, String)>,
    mut hash: u64,
) -> std::result::Result<u64, String> {
    for (ext, nombre) in referidos {
        match leer_referencia(workspace, &ext, &nombre) {
            LecturaReferencia::Ok(bytes) => {
                hash = etiquetar(hash, &format!("ref:{nombre}"), &bytes);
            }
            LecturaReferencia::NotFound(razon) | LecturaReferencia::IoError(razon) => {
                return Err(razon);
            }
        }
    }
    Ok(hash)
}

fn hashear_opciones(
    workspace: &Path,
    cli_args: &[String],
    mut hash: u64,
) -> std::result::Result<u64, String> {
    for opcion in OPCIONES_ENTRADA {
        if let Some(v) = valor(cli_args, opcion) {
            match leer_opcion(workspace, opcion, v) {
                LecturaReferencia::Ok(bytes) => {
                    hash = etiquetar(hash, opcion, &bytes);
                }
                LecturaReferencia::NotFound(razon) | LecturaReferencia::IoError(razon) => {
                    return Err(razon);
                }
            }
        }
    }
    Ok(hash)
}

pub(crate) fn clave(workspace: &Path, cli_args: &[String]) -> Result<ClaveCache> {
    let proyecto = valor(cli_args, "-p")
        .ok_or_else(|| Error::Invalid("Falta el proyecto (-p) para la clave de caché".into()))?;
    let prj = resolve_project_file(workspace, proyecto)?;
    let mut archivos = vec![("proyecto".to_owned(), prj.clone())];
    if let Some(variante) = valor(cli_args, "-v") {
        archivos.push(("variante".to_owned(), prj.with_extension(variante)));
    }

    let mut referidos = Vec::new();
    let mut hash = argumentos_clave(cli_args).iter().fold(OFFSET_FNV, |h, a| {
        absorber(absorber(h, a.as_bytes()), b"\n")
    });

    for (rotulo, ruta) in &archivos {
        let contenido = match fs::read(ruta) {
            Ok(b) => b,
            Err(e) => {
                return Ok(ClaveCache::Inutilizable {
                    razon: format!(
                        "Error de E/S leyendo archivo '{rotulo}' en '{}': {e}",
                        ruta.display()
                    ),
                })
            }
        };
        hash = etiquetar(hash, rotulo, &contenido);
        referidos.extend(referencias(&String::from_utf8_lossy(&contenido)));
    }

    let hash = match hashear_referencias(workspace, referidos, hash) {
        Ok(h) => h,
        Err(razon) => return Ok(ClaveCache::Inutilizable { razon }),
    };
    let hash = match hashear_opciones(workspace, cli_args, hash) {
        Ok(h) => h,
        Err(razon) => return Ok(ClaveCache::Inutilizable { razon }),
    };

    Ok(ClaveCache::Valida(format!("{hash:016x}")))
}
