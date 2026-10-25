//! Tools for workspace operations, file reading/writing, editing and listing.

use super::declaracion::{Anotaciones, Discriminador, Ejecutor, Herramienta, Sinonimos, TipoDato};
use super::workspace_ejecucion::{
    ejecutar_editar, ejecutar_escribir_archivo, ejecutar_leer, ejecutar_listar, ejecutar_workspace,
};

pub(crate) const TOOL_WORKSPACE: Herramienta = Herramienta {
    nombre: "workspace",
    descripcion: "Workspace: crear, exportar, importar, exportar proyecto, historial de cambios y restaurar. / EN: Workspace: create, export, import, export project, change history and restore.",
    anotaciones: Anotaciones {
        titulo: "Workspace",
        es_solo_lectura: false,
        es_destructiva: true,
        es_idempotente: false,
        usa_red: false,
    },
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
    sinonimos: SINONIMOS_WORKSPACE,
    ejecutar: Ejecutor::Simple(ejecutar_workspace),
};

const SINONIMOS_WORKSPACE: Sinonimos = Sinonimos {
    argumentos: &[
        ("action", "accion"),
        ("path", "ruta"),
        ("overwrite", "sobrescribir"),
        ("target", "destino"),
        ("destination", "destino"),
        ("archive", "zip"),
        ("project", "proyecto"),
    ],
    valores: &[
        ("create", "crear"),
        ("export", "exportar"),
        ("import", "importar"),
        ("export_project", "exportar_proyecto"),
        ("history", "historial"),
        ("restore", "restaurar"),
    ],
};

pub(crate) const TOOL_ESCRIBIR_ARCHIVO: Herramienta = Herramienta {
    nombre: "escribir_archivo",
    descripcion: "Escribe un archivo dentro del workspace (con historial). / EN: Writes a file inside the workspace (with history).",
    anotaciones: Anotaciones {
        titulo: "Escribir archivo / Write file",
        es_solo_lectura: false,
        es_destructiva: true,
        es_idempotente: true,
        usa_red: false,
    },
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
    sinonimos: SINONIMOS_ESCRIBIR_ARCHIVO,
    ejecutar: Ejecutor::Simple(ejecutar_escribir_archivo),
};

const SINONIMOS_ESCRIBIR_ARCHIVO: Sinonimos = Sinonimos {
    argumentos: &[("file", "archivo"), ("content", "contenido")],
    valores: &[],
};

pub(crate) const TOOL_LEER: Herramienta = Herramienta {
    nombre: "leer",
    descripcion: "Lee cualquier archivo PVsyst: proyecto, modulo, sitio, meteo, componente, objeto o bytes crudos (detectado por archivo). / EN: Reads any PVsyst file: project, module, site, weather, component, object or raw bytes (detected from the file).",
    anotaciones: Anotaciones {
        titulo: "Leer / Read",
        es_solo_lectura: true,
        es_destructiva: false,
        es_idempotente: true,
        usa_red: false,
    },
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
        ("serie", TipoDato::Boolean),
        ("limite", TipoDato::Integer),
    ],
    requeridos: &["archivo"],
    discriminador: None,
    requeridos_por_accion: &[],
    sinonimos: SINONIMOS_LEER,
    ejecutar: Ejecutor::Simple(ejecutar_leer),
};

const SINONIMOS_LEER: Sinonimos = Sinonimos {
    argumentos: &[
        ("file", "archivo"),
        ("object", "objeto"),
        ("flat", "plano"),
        ("type", "tipo"),
        ("series", "serie"),
        ("limit", "limite"),
    ],
    valores: &[],
};

pub(crate) const TOOL_EDITAR: Herramienta = Herramienta {
    nombre: "editar",
    descripcion: "Edita campos, inserta o elimina objetos PVObject (byte-preserving, con historial). / EN: Edits fields, inserts or removes PVObject objects (byte-preserving, with history).",
    anotaciones: Anotaciones {
        titulo: "Editar / Edit",
        es_solo_lectura: false,
        es_destructiva: true,
        es_idempotente: false,
        usa_red: false,
    },
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
        ("operacion", TipoDato::String),
        ("variables", TipoDato::Array),
        ("batch_rvt_file", TipoDato::String),
    ],
    requeridos: &[],
    discriminador: Some(Discriminador {
        campo: "accion",
        defecto: Some("campos"),
        valores: &["campos", "insertar_objeto", "eliminar_objeto", "rvt", "plantilla_rvt"],
    }),
    requeridos_por_accion: &[
        ("insertar_objeto", &["archivo", "padre", "desde"]),
        ("eliminar_objeto", &["archivo", "objeto"]),
        ("rvt", &["operacion"]),
        ("plantilla_rvt", &["operacion"]),
    ],
    sinonimos: SINONIMOS_EDITAR,
    ejecutar: Ejecutor::Simple(ejecutar_editar),
};

const SINONIMOS_EDITAR: Sinonimos = Sinonimos {
    argumentos: &[
        ("action", "accion"),
        ("file", "archivo"),
        ("type", "tipo"),
        ("object", "objeto"),
        ("fields", "campos"),
        ("edits", "ediciones"),
        ("project", "proyecto"),
        ("variant", "variante"),
        ("parent", "padre"),
        ("from", "desde"),
        ("replace", "reemplazar"),
        ("operation", "operacion"),
    ],
    valores: &[
        ("fields", "campos"),
        ("insert_object", "insertar_objeto"),
        ("delete_object", "eliminar_objeto"),
        ("remove_object", "eliminar_objeto"),
    ],
};

pub(crate) const TOOL_LISTAR: Herramienta = Herramienta {
    nombre: "listar",
    descripcion: "Lista proyectos, componentes, modulos, archivos u objetos. / EN: Lists projects, components, modules, files or objects.",
    anotaciones: Anotaciones {
        titulo: "Listar / List",
        es_solo_lectura: true,
        es_destructiva: false,
        es_idempotente: true,
        usa_red: false,
    },
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
        ("todos", TipoDato::Boolean),
        ("permitir_fuera_del_workspace", TipoDato::Boolean),
    ],
    requeridos: &[],
    discriminador: Some(Discriminador {
        campo: "que",
        defecto: Some("proyectos"),
        valores: &["proyectos", "componentes", "modulos", "archivos", "objetos"],
    }),
    requeridos_por_accion: &[("componentes", &["tipo"]), ("objetos", &["archivo"])],
    sinonimos: SINONIMOS_LISTAR,
    ejecutar: Ejecutor::Simple(ejecutar_listar),
};

const SINONIMOS_LISTAR: Sinonimos = Sinonimos {
    argumentos: &[
        ("what", "que"),
        ("type", "tipo"),
        ("text", "texto"),
        ("folder", "carpeta"),
        ("file", "archivo"),
        ("recursive", "recursivo"),
    ],
    valores: &[
        ("projects", "proyectos"),
        ("components", "componentes"),
        ("modules", "modulos"),
        ("files", "archivos"),
        ("objects", "objetos"),
    ],
};
