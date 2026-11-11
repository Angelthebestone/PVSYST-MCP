//! `entorno` and `disenar_sistema`.

use super::{entrada, paso, Entrada, Paso, DIAGNOSTICO, VER_KPIS, VER_PERDIDAS, VER_PROYECTOS};
use crate::idioma::Texto;

const VER_ENTORNO: Paso = DIAGNOSTICO;
const VER_LICENCIA: Paso = paso(
    "entorno",
    Texto {
        es: "Comprobar la licencia",
        en: "Check the licence",
    },
)
.con(&[("accion", "licencia")]);
const VER_PREFERENCIAS: Paso = paso(
    "entorno",
    Texto {
        es: "Leer las preferencias",
        en: "Read the preferences",
    },
)
.con(&[("accion", "preferencias")]);

pub(super) const ENTRADAS: &[Entrada] = &[
    entrada(
        "entorno",
        "diagnostico",
        Texto {
            es: "Diagnóstico del entorno: PVsyst, CLI, workspace y herramientas.",
            en: "Environment diagnosis: PVsyst, CLI, workspace and tools.",
        },
    )
    .pista(Texto {
        es: "disenar_sistema {latitud,longitud,kwp}",
        en: "disenar_sistema {latitud,longitud,kwp}",
    })
    .luego(&[VER_PROYECTOS, VER_LICENCIA]),
    entrada(
        "entorno",
        "licencia",
        Texto {
            es: "Estado de la licencia de PVsystCLI.",
            en: "PVsystCLI licence status.",
        },
    )
    .luego(&[VER_ENTORNO]),
    entrada(
        "entorno",
        "licencia_sync",
        Texto {
            es: "Comando de sincronización preparado; el MCP no sincroniza licencias.",
            en: "Synchronisation command prepared; the MCP does not synchronise licences.",
        },
    )
    .luego(&[VER_LICENCIA]),
    entrada(
        "entorno",
        "licencia_activar",
        Texto {
            es: "Comando de activación preparado; el MCP no activa licencias.",
            en: "Activation command prepared; the MCP does not activate licences.",
        },
    )
    .luego(&[VER_LICENCIA]),
    entrada(
        "entorno",
        "licencia_desactivar",
        Texto {
            es: "Comando de desactivación preparado; el MCP no desactiva licencias.",
            en: "Deactivation command prepared; the MCP does not deactivate licences.",
        },
    )
    .luego(&[VER_LICENCIA]),
    entrada(
        "entorno",
        "exportar_logs",
        Texto {
            es: "Registros de PVsyst exportados.",
            en: "PVsyst logs exported.",
        },
    )
    .luego(&[VER_ENTORNO]),
    entrada(
        "entorno",
        "logs_cli",
        Texto {
            es: "Últimas líneas del registro de PVsystCLI.",
            en: "Last lines of the PVsystCLI log.",
        },
    )
    .luego(&[VER_ENTORNO]),
    entrada(
        "entorno",
        "preferencias",
        Texto {
            es: "Preferencias de PVsyst leídas.",
            en: "PVsyst preferences read.",
        },
    )
    .pista(Texto {
        es: "entorno preferencias {seccion:'Preferences'}",
        en: "entorno preferencias {seccion:'Preferences'}",
    })
    .luego(&[VER_ENTORNO]),
    entrada(
        "entorno",
        "preferencias_editar",
        Texto {
            es: "Preferencia de PVsyst modificada.",
            en: "PVsyst preference changed.",
        },
    )
    .luego(&[VER_PREFERENCIAS]),
    entrada(
        "disenar_sistema",
        "",
        Texto {
            es: "Sistema diseñado de extremo a extremo: sitio, proyecto, dimensionado y simulación.",
            en: "System designed end to end: site, project, sizing and simulation.",
        },
    )
    .pista(Texto {
        es: "resultados que:perdidas",
        en: "resultados que:perdidas",
    })
    .luego(&[VER_PERDIDAS, VER_KPIS, VER_ENTORNO]),
];
