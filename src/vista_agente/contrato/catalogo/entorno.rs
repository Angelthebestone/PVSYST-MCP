//! `entorno` and `disenar_sistema`.

use super::{entrada, paso, Entrada, Paso, DIAGNOSTICO, VER_KPIS, VER_PERDIDAS, VER_PROYECTOS};

const VER_ENTORNO: Paso = DIAGNOSTICO;
const VER_LICENCIA: Paso = paso("entorno", "Comprobar la licencia").con(&[("accion", "licencia")]);
const VER_PREFERENCIAS: Paso =
    paso("entorno", "Leer las preferencias").con(&[("accion", "preferencias")]);

pub(super) const ENTRADAS: &[Entrada] = &[
    entrada(
        "entorno",
        "diagnostico",
        "Diagnóstico del entorno: PVsyst, CLI, workspace y herramientas.",
    )
    .pista("disenar_sistema {latitud,longitud,kwp}")
    .luego(&[VER_PROYECTOS, VER_LICENCIA]),
    entrada("entorno", "licencia", "Estado de la licencia de PVsystCLI.").luego(&[VER_ENTORNO]),
    entrada(
        "entorno",
        "licencia_sync",
        "Licencia sincronizada con el servidor de PVsyst.",
    )
    .luego(&[VER_LICENCIA]),
    entrada(
        "entorno",
        "licencia_activar",
        "Comando de activación preparado; el MCP no activa licencias.",
    )
    .luego(&[VER_LICENCIA]),
    entrada(
        "entorno",
        "licencia_desactivar",
        "Comando de desactivación preparado; el MCP no desactiva licencias.",
    )
    .luego(&[VER_LICENCIA]),
    entrada(
        "entorno",
        "exportar_logs",
        "Registros de PVsyst exportados.",
    )
    .luego(&[VER_ENTORNO]),
    entrada(
        "entorno",
        "logs_cli",
        "Últimas líneas del registro de PVsystCLI.",
    )
    .luego(&[VER_ENTORNO]),
    entrada("entorno", "preferencias", "Preferencias de PVsyst leídas.")
        .pista("entorno preferencias {seccion:'Preferences'}")
        .luego(&[VER_ENTORNO]),
    entrada(
        "entorno",
        "preferencias_editar",
        "Preferencia de PVsyst modificada.",
    )
    .luego(&[VER_PREFERENCIAS]),
    entrada(
        "disenar_sistema",
        "",
        "Sistema diseñado de extremo a extremo: sitio, proyecto, dimensionado y simulación.",
    )
    .pista("resultados que:perdidas")
    .luego(&[VER_PERDIDAS, VER_KPIS, VER_ENTORNO]),
];
