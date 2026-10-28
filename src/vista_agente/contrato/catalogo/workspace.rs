//! `workspace`, `listar`, `leer`, `editar` and `escribir_archivo`.

use super::{entrada, paso, Entrada, Paso, DE_ARCHIVO, DE_PROYECTO};

const VER_PROYECTOS: Paso =
    paso("listar", "Ver los proyectos del workspace").con(&[("que", "proyectos")]);
const VER_ARCHIVOS: Paso =
    paso("listar", "Ver los archivos del workspace").con(&[("que", "archivos")]);
const VER_MODULOS: Paso = paso("listar", "Ver los módulos disponibles").con(&[("que", "modulos")]);
const VER_HISTORIAL: Paso =
    paso("workspace", "Ver el historial de cambios").con(&[("accion", "historial")]);
const LEER: Paso = paso("leer", "Leer el archivo").tomando(DE_ARCHIVO);
const VER_OBJETOS: Paso = paso("listar", "Ver los objetos del archivo")
    .con(&[("que", "objetos")])
    .tomando(DE_ARCHIVO);
const REVISAR: Paso = paso("proyecto", "Revisar que el proyecto es coherente")
    .con(&[("accion", "doctor")])
    .tomando(DE_PROYECTO);

pub(super) const ENTRADAS: &[Entrada] = &[
    entrada("workspace", "crear", "Workspace creado.").luego(&[VER_PROYECTOS, VER_HISTORIAL]),
    entrada("workspace", "exportar", "Workspace exportado a un archivo.")
        .luego(&[VER_PROYECTOS, VER_HISTORIAL]),
    entrada("workspace", "importar", "Workspace importado.").luego(&[VER_PROYECTOS, VER_HISTORIAL]),
    entrada(
        "workspace",
        "exportar_proyecto",
        "Proyecto exportado a un archivo.",
    )
    .luego(&[VER_PROYECTOS, VER_HISTORIAL]),
    entrada(
        "workspace",
        "historial",
        "Historial de cambios del workspace.",
    )
    .luego(&[VER_PROYECTOS]),
    entrada(
        "workspace",
        "restaurar",
        "Cambio restaurado desde el historial.",
    )
    .luego(&[VER_HISTORIAL, VER_PROYECTOS]),
    entrada("listar", "proyectos", "Proyectos del workspace listados.")
        .luego(&[VER_ARCHIVOS, VER_MODULOS]),
    entrada(
        "listar",
        "componentes",
        "Componentes del tipo pedido listados.",
    )
    .luego(&[VER_MODULOS, VER_PROYECTOS]),
    entrada("listar", "modulos", "Módulos fotovoltaicos listados.")
        .luego(&[VER_ARCHIVOS, VER_PROYECTOS]),
    entrada("listar", "archivos", "Archivos del workspace listados.")
        .luego(&[VER_PROYECTOS, VER_MODULOS]),
    entrada("listar", "objetos", "Objetos del archivo PVsyst listados.")
        .luego(&[LEER, VER_PROYECTOS]),
    entrada("leer", "", "Archivo de PVsyst leído.").luego(&[VER_OBJETOS, VER_PROYECTOS]),
    entrada(
        "editar",
        "campos",
        "Campos editados con copia en el historial.",
    )
    .pista("proyecto doctor")
    .luego(&[REVISAR, LEER, VER_HISTORIAL]),
    entrada(
        "editar",
        "insertar_objeto",
        "Objeto insertado en el archivo.",
    )
    .luego(&[LEER, VER_HISTORIAL]),
    entrada("editar", "eliminar_objeto", "Objeto eliminado del archivo.")
        .luego(&[LEER, VER_HISTORIAL]),
    entrada("editar", "rvt", "Operación sobre el archivo RVT aplicada.").luego(&[VER_HISTORIAL]),
    entrada("editar", "plantilla_rvt", "Plantilla RVT generada.").luego(&[VER_HISTORIAL]),
    entrada(
        "escribir_archivo",
        "",
        "Archivo escrito en el workspace con copia en el historial.",
    )
    .luego(&[LEER, VER_HISTORIAL]),
];
