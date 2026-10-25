//! Handlers for workspace management, file reading/writing, editing and listing.

use super::declaracion::con;
use crate::admin;
use crate::args::workspace_of;
use crate::components::{self, pan};
use crate::config::Paths;
use crate::error::{Error, Result};
use crate::history;
use crate::objects;
use crate::projects::{self, lifecycle};
use crate::sites::readers;
use crate::workspace::Alcance;
use serde_json::{json, Value};
use std::path::Path;

pub(super) fn ejecutar_workspace(paths: &Paths, args: &Value) -> Result<Value> {
    let valor = args.get("accion").and_then(Value::as_str).unwrap_or("");
    match valor {
        "crear" => admin::crear_workspace(paths, args),
        "exportar" => admin::exportar_workspace(paths, args),
        "importar" => admin::importar_workspace(paths, args),
        "exportar_proyecto" => admin::exportar_proyecto(paths, args),
        "historial" => {
            let n = args
                .get("n")
                .and_then(Value::as_u64)
                .and_then(|n| usize::try_from(n).ok());
            Ok(history::listar(&workspace_of(paths, args), n.unwrap_or(20)))
        }
        "restaurar" => history::restaurar(
            &workspace_of(paths, args),
            args.get("id").and_then(Value::as_str).unwrap_or_default(),
            Alcance::de_args(args),
        ),
        _ => Err(Error::Invalid(format!(
            "Valor '{valor}' no soportado para workspace"
        ))),
    }
}

pub(super) fn ejecutar_escribir_archivo(paths: &Paths, args: &Value) -> Result<Value> {
    crate::workspace::escribir_archivo(paths, args)
}

pub(super) fn ejecutar_leer(paths: &Paths, args: &Value) -> Result<Value> {
    if args.get("serie").and_then(Value::as_bool) == Some(true) {
        return crate::serie_horaria::leer_serie(paths, args);
    }
    let pide_offset = args.get("offset").and_then(Value::as_u64).is_some();
    let pide_max = args.get("max_bytes").and_then(Value::as_u64).is_some();
    let pide_base64 = crate::args::flag(args, &["base64"]);
    if pide_offset || pide_max || pide_base64 {
        return crate::workspace::leer_archivo(paths, args);
    }
    if args.get("objeto").is_some() || args.get("plano").is_some() {
        return objects::leer_objeto(paths, args);
    }
    let archivo = args
        .get("archivo")
        .and_then(Value::as_str)
        .unwrap_or_default();
    let ext = extension_mayuscula(archivo);

    if es_proyecto(&workspace_of(paths, args), archivo, &ext) {
        let completo = args.get("detalle").and_then(Value::as_str) == Some("completo");
        let con_proyecto = con(args, "proyecto", json!(archivo));
        return projects::leer_proyecto(paths, &con(&con_proyecto, "completo", json!(completo)));
    }
    if es_archivo_divisas(paths, args, archivo) {
        return crate::divisas::leer_divisas(paths, args);
    }
    despachar_por_extension(paths, args, archivo)
}

fn es_archivo_divisas(paths: &Paths, args: &Value, archivo: &str) -> bool {
    let nombre_archivo = Path::new(archivo)
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or(archivo);
    if nombre_archivo.eq_ignore_ascii_case("Currency.dat") {
        return true;
    }
    if extension_mayuscula(archivo) == "DAT" {
        return empieza_con_cabecera_divisas(paths, args, archivo);
    }
    false
}

fn empieza_con_cabecera_divisas(paths: &Paths, args: &Value, archivo: &str) -> bool {
    use std::fs::File;
    use std::io::Read;
    let ws = workspace_of(paths, args);
    let peticion = crate::workspace::PeticionResolucion {
        nombre: archivo,
        subcarpeta_ws: None,
        rutas_dataro: Vec::new(),
    };
    let Ok(ruta) = crate::workspace::resolver_archivo_con_fallback(&ws, args, &peticion) else {
        return false;
    };
    let Ok(mut file) = File::open(&ruta) else {
        return false;
    };
    let mut buf = [0u8; 17];
    if file.read_exact(&mut buf).is_err() {
        return false;
    }
    &buf == b"CURRENCY_RATES_V2"
}

fn despachar_por_extension(paths: &Paths, args: &Value, archivo: &str) -> Result<Value> {
    let ext = extension_mayuscula(archivo);
    match ext.as_str() {
        "PAN" => pan::leer_modulo(paths, &con(args, "modulo", json!(archivo))),
        "SIT" => readers::leer_sitio(paths, &con(args, "sitio", json!(archivo))),
        "MET" => readers::leer_meteo(paths, &con(args, "meteo", json!(archivo))),
        "DAM" => crate::serie_horaria::leer_serie(paths, args),
        "HOR" => crate::horizonte_archivo::leer_horizonte(paths, args),
        "MEF" => crate::meteo_mef::leer_mef(paths, args),
        "SHD" | "BLD" | "SHO" => crate::escena_3d::leer_escena(paths, args),
        "OND" | "BTR" | "PMP" | "RLT" | "RLP" | "GEN" | "REV" | "CLI" => {
            let con_nombre = con(args, "componente", json!(archivo));
            components::leer_componente(paths, &con(&con_nombre, "tipo", json!(ext)))
        }
        _ => objects::leer_objeto(paths, args)
            .or_else(|_| crate::workspace::leer_archivo(paths, args)),
    }
}

fn es_proyecto(workspace: &Path, archivo: &str, ext: &str) -> bool {
    let variante = ext.len() == 3 && (ext.starts_with("VC") || ext.starts_with("CM"));
    let nombre = ext.is_empty()
        && workspace
            .join("Projects")
            .join(format!("{archivo}.PRJ"))
            .exists();
    ext == "PRJ" || variante || nombre
}

pub(super) fn ejecutar_editar(paths: &Paths, args: &Value) -> Result<Value> {
    let valor = super::workspace::TOOL_EDITAR.accion(args);
    match valor {
        "campos" if args.get("ediciones").is_some() => lifecycle::editar_variante(paths, args),
        "campos" => objects::editar_objeto(paths, args),
        "insertar_objeto" => projects::anadir::insertar_objeto_archivo(paths, args),
        "eliminar_objeto" => projects::anadir::eliminar_objeto_archivo(paths, args),
        "rvt" | "plantilla_rvt" => crate::plantilla_variables::gestionar_rvt(paths, args),
        _ => Err(Error::Invalid(format!(
            "Valor '{valor}' no soportado para editar"
        ))),
    }
}

pub(super) fn ejecutar_listar(paths: &Paths, args: &Value) -> Result<Value> {
    let valor = super::workspace::TOOL_LISTAR.accion(args);
    match valor {
        "proyectos" => projects::listar_proyectos(paths, args),
        "componentes" => components::listar_componentes(paths, args),
        "modulos" => pan::listar_modulos(paths, args),
        "archivos" => crate::workspace::listar_archivos(paths, args),
        "objetos" => objects::listar_objetos(paths, args),
        _ => Err(Error::Invalid(format!(
            "Valor '{valor}' no soportado para listar"
        ))),
    }
}

fn extension_mayuscula(archivo: &str) -> String {
    Path::new(archivo)
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("")
        .to_ascii_uppercase()
}
