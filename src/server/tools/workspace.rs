//! Tools for workspace operations, file reading/writing, editing and listing.

use super::declaracion::{con, Acceso, Discriminador, Herramienta, TipoDato};
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

pub(crate) const TOOL_WORKSPACE: Herramienta = Herramienta {
    nombre: "workspace",
    descripcion:
        "Workspace: crear, exportar, importar, exportar proyecto, historial de cambios y restaurar.",
    acceso: Acceso::Normal,
    propiedades: &[
        ("ruta", TipoDato::String),
        ("sobrescribir", TipoDato::Boolean),
        ("destino", TipoDato::String),
        ("zip", TipoDato::String),
        ("proyecto", TipoDato::String),
        ("workspace", TipoDato::String),
        ("n", TipoDato::Integer),
        ("id", TipoDato::String),
        ("permitir_fuera_del_workspace", TipoDato::Boolean),
    ],
    requeridos: &[],
    discriminador: Some(Discriminador {
        campo: "accion",
        defecto: None,
        valores: &[
            "crear",
            "exportar",
            "importar",
            "exportar_proyecto",
            "historial",
            "restaurar",
        ],
    }),
    requeridos_por_accion: &[("restaurar", &["id"])],
    ejecutar: ejecutar_workspace,
};

pub(crate) const TOOL_ESCRIBIR_ARCHIVO: Herramienta = Herramienta {
    nombre: "escribir_archivo",
    descripcion: "Escribe un archivo dentro del workspace (con historial).",
    acceso: Acceso::Normal,
    propiedades: &[
        ("archivo", TipoDato::String),
        ("contenido", TipoDato::String),
        ("base64", TipoDato::Boolean),
        ("permitir_fuera_del_workspace", TipoDato::Boolean),
        ("workspace", TipoDato::String),
    ],
    requeridos: &["archivo", "contenido"],
    discriminador: None,
    requeridos_por_accion: &[],
    ejecutar: ejecutar_escribir_archivo,
};

pub(crate) const TOOL_LEER: Herramienta = Herramienta {
    nombre: "leer",
    descripcion: "Lee cualquier archivo PVsyst: proyecto, modulo, sitio, meteo, componente, objeto o bytes crudos (detectado por archivo).",
    acceso: Acceso::Normal,
    propiedades: &[
        ("archivo", TipoDato::String),
        ("objeto", TipoDato::String),
        ("plano", TipoDato::Boolean),
        ("tipo", TipoDato::String),
        ("offset", TipoDato::Integer),
        ("max_bytes", TipoDato::Integer),
        ("base64", TipoDato::Boolean),
        ("workspace", TipoDato::String),
        ("permitir_fuera_del_workspace", TipoDato::Boolean),
    ],
    requeridos: &["archivo"],
    discriminador: None,
    requeridos_por_accion: &[],
    ejecutar: ejecutar_leer,
};

pub(crate) const TOOL_EDITAR: Herramienta = Herramienta {
    nombre: "editar",
    descripcion:
        "Edita campos, inserta o elimina objetos PVObject (byte-preserving, con historial).",
    acceso: Acceso::Normal,
    propiedades: &[
        ("archivo", TipoDato::String),
        ("tipo", TipoDato::String),
        ("objeto", TipoDato::String),
        ("campos", TipoDato::Object),
        ("ediciones", TipoDato::Array),
        ("proyecto", TipoDato::String),
        ("variante", TipoDato::String),
        ("padre", TipoDato::String),
        ("desde", TipoDato::Object),
        ("reemplazar", TipoDato::Boolean),
        ("workspace", TipoDato::String),
        ("permitir_fuera_del_workspace", TipoDato::Boolean),
    ],
    requeridos: &[],
    discriminador: Some(Discriminador {
        campo: "accion",
        defecto: Some("campos"),
        valores: &["campos", "insertar_objeto", "eliminar_objeto"],
    }),
    requeridos_por_accion: &[
        ("insertar_objeto", &["archivo", "padre", "desde"]),
        ("eliminar_objeto", &["archivo", "objeto"]),
    ],
    ejecutar: ejecutar_editar,
};

pub(crate) const TOOL_LISTAR: Herramienta = Herramienta {
    nombre: "listar",
    descripcion: "Lista proyectos, componentes, modulos, archivos u objetos.",
    acceso: Acceso::Normal,
    propiedades: &[
        ("include_variants", TipoDato::Boolean),
        ("workspace", TipoDato::String),
        ("tipo", TipoDato::String),
        ("pnom_min", TipoDato::Number),
        ("pnom_max", TipoDato::Number),
        ("texto", TipoDato::String),
        ("carpeta", TipoDato::String),
        ("extension", TipoDato::String),
        ("recursivo", TipoDato::Boolean),
        ("archivo", TipoDato::String),
        ("permitir_fuera_del_workspace", TipoDato::Boolean),
    ],
    requeridos: &[],
    discriminador: Some(Discriminador {
        campo: "que",
        defecto: Some("proyectos"),
        valores: &["proyectos", "componentes", "modulos", "archivos", "objetos"],
    }),
    requeridos_por_accion: &[("componentes", &["tipo"]), ("objetos", &["archivo"])],
    ejecutar: ejecutar_listar,
};

fn ejecutar_workspace(paths: &Paths, args: &Value) -> Result<Value> {
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

fn ejecutar_escribir_archivo(paths: &Paths, args: &Value) -> Result<Value> {
    crate::workspace::escribir_archivo(paths, args)
}

fn ejecutar_leer(paths: &Paths, args: &Value) -> Result<Value> {
    if ["offset", "max_bytes", "base64"]
        .iter()
        .any(|k| args.get(*k).is_some())
    {
        return crate::workspace::leer_archivo(paths, args);
    }
    if args.get("objeto").is_some() || args.get("plano").is_some() {
        return objects::leer_objeto(paths, args);
    }
    let archivo = args
        .get("archivo")
        .and_then(Value::as_str)
        .unwrap_or_default();
    let ext = Path::new(archivo)
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("")
        .to_ascii_uppercase();
    if es_proyecto(paths, archivo, &ext) {
        let completo = args.get("detalle").and_then(Value::as_str) == Some("completo");
        let con_proyecto = con(args, "proyecto", json!(archivo));
        return projects::leer_proyecto(paths, &con(&con_proyecto, "completo", json!(completo)));
    }
    match ext.as_str() {
        "PAN" => pan::leer_modulo(paths, &con(args, "modulo", json!(archivo))),
        "SIT" => readers::leer_sitio(paths, &con(args, "sitio", json!(archivo))),
        "MET" => readers::leer_meteo(paths, &con(args, "meteo", json!(archivo))),
        "OND" | "BTR" | "PMP" | "RLT" | "RLP" | "GEN" | "REV" | "CLI" => {
            let con_nombre = con(args, "componente", json!(archivo));
            components::leer_componente(paths, &con(&con_nombre, "tipo", json!(ext)))
        }
        _ => objects::leer_objeto(paths, args)
            .or_else(|_| crate::workspace::leer_archivo(paths, args)),
    }
}

fn es_proyecto(paths: &Paths, archivo: &str, ext: &str) -> bool {
    let variante = ext.len() == 3 && (ext.starts_with("VC") || ext.starts_with("CM"));
    let nombre = ext.is_empty() && paths.projects_dir().join(format!("{archivo}.PRJ")).exists();
    ext == "PRJ" || variante || nombre
}

fn ejecutar_editar(paths: &Paths, args: &Value) -> Result<Value> {
    let valor = args
        .get("accion")
        .and_then(Value::as_str)
        .unwrap_or("campos");
    match valor {
        "campos" if args.get("ediciones").is_some() => lifecycle::editar_variante(paths, args),
        "campos" => objects::editar_objeto(paths, args),
        "insertar_objeto" => projects::anadir::insertar_objeto_archivo(paths, args),
        "eliminar_objeto" => projects::anadir::eliminar_objeto_archivo(paths, args),
        _ => Err(Error::Invalid(format!(
            "Valor '{valor}' no soportado para editar"
        ))),
    }
}

fn ejecutar_listar(paths: &Paths, args: &Value) -> Result<Value> {
    let valor = args
        .get("que")
        .and_then(Value::as_str)
        .unwrap_or("proyectos");
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
