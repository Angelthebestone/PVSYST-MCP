//! What each tool action does and where to go next, as data. One entry per
//! `tool` × `accion`; a test fails when a declared action has no entry.

use crate::idioma::Texto;

mod componentes;
mod entorno;
mod gui;
mod proyectos;
mod resultados;
mod sitios;
mod workspace;

/// Where a next-step argument is taken from: the request first, then the answer.
pub(crate) type Fuente = (&'static str, &'static [&'static str]);

/// A suggested call. `fijos` are always sent; every `tomar` value must be found
/// in the request or the answer, otherwise the step is not offered: a step never
/// carries a placeholder. `si_esta` values are added only when they are found.
#[derive(Debug, Clone, Copy)]
pub(crate) struct Paso {
    pub(crate) tool: &'static str,
    pub(crate) motivo: Texto,
    pub(crate) fijos: &'static [(&'static str, &'static str)],
    pub(crate) tomar: &'static [Fuente],
    pub(crate) si_esta: &'static [Fuente],
}

pub(super) const fn paso(tool: &'static str, motivo: Texto) -> Paso {
    Paso {
        tool,
        motivo,
        fijos: &[],
        tomar: &[],
        si_esta: &[],
    }
}

impl Paso {
    pub(super) const fn con(self, fijos: &'static [(&'static str, &'static str)]) -> Self {
        Paso { fijos, ..self }
    }

    pub(super) const fn tomando(self, tomar: &'static [Fuente]) -> Self {
        Paso { tomar, ..self }
    }

    /// Values the step takes only when the request or the answer carries them.
    pub(super) const fn tomando_si_esta(self, si_esta: &'static [Fuente]) -> Self {
        Paso { si_esta, ..self }
    }
}

/// What one action reports and suggests. `pista` is the short text kept under
/// the top-level `siguiente` key for callers that already read it.
#[derive(Debug, Clone, Copy)]
pub(crate) struct Entrada {
    pub(crate) tool: &'static str,
    pub(crate) accion: &'static str,
    pub(crate) resumen: Texto,
    pub(crate) pista: Option<Texto>,
    pub(crate) siguiente: &'static [Paso],
}

pub(super) const fn entrada(tool: &'static str, accion: &'static str, resumen: Texto) -> Entrada {
    Entrada {
        tool,
        accion,
        resumen,
        pista: None,
        siguiente: &[],
    }
}

impl Entrada {
    pub(super) const fn pista(self, pista: Texto) -> Self {
        Entrada {
            pista: Some(pista),
            ..self
        }
    }

    pub(super) const fn luego(self, siguiente: &'static [Paso]) -> Self {
        Entrada { siguiente, ..self }
    }
}

/// Step offered when a call did not finish; it needs no arguments.
pub(crate) const DIAGNOSTICO: Paso = paso(
    "entorno",
    Texto {
        es: "Comprobar la instalación, la CLI y el workspace",
        en: "Check the installation, the CLI and the workspace",
    },
)
.con(&[("accion", "diagnostico")]);

pub(super) const PROYECTO: Fuente = (
    "proyecto",
    &["proyecto", "nombre_nuevo", "proyecto_creado", "project"],
);
pub(super) const ARCHIVO: Fuente = ("archivo", &["archivo", "file", "sit_file"]);
pub(super) const VENTANA: Fuente = ("ventana", &["ventana"]);
pub(super) const DE_PROYECTO: &[Fuente] = &[PROYECTO];
pub(super) const DE_ARCHIVO: &[Fuente] = &[ARCHIVO];
pub(super) const DE_VENTANA: &[Fuente] = &[VENTANA];

// Next steps several features share; one definition each.
pub(super) const VER_PROYECTOS: Paso = paso(
    "listar",
    Texto {
        es: "Ver los proyectos del workspace",
        en: "See the workspace projects",
    },
)
.con(&[("que", "proyectos")]);
pub(super) const VER_ARCHIVOS: Paso = paso(
    "listar",
    Texto {
        es: "Ver los archivos del workspace",
        en: "See the workspace files",
    },
)
.con(&[("que", "archivos")]);
/// The components of the kind the request or the answer names (`PAN`, `OND`...).
pub(super) const TIPO: Fuente = ("tipo", &["tipo", "kind"]);
pub(super) const VER_COMPONENTES: Paso = paso(
    "listar",
    Texto {
        es: "Ver los componentes de ese tipo",
        en: "See the components of that kind",
    },
)
.con(&[("que", "componentes")])
.tomando(&[TIPO]);
pub(super) const VER_MODULOS: Paso = paso(
    "listar",
    Texto {
        es: "Ver los módulos disponibles",
        en: "See the available modules",
    },
)
.con(&[("que", "modulos")]);
pub(super) const VER_HISTORIAL: Paso = paso(
    "workspace",
    Texto {
        es: "Ver el historial de cambios para poder revertir",
        en: "See the change history so it can be reverted",
    },
)
.con(&[("accion", "historial")]);
pub(super) const REVISAR: Paso = paso(
    "proyecto",
    Texto {
        es: "Revisar que el proyecto es coherente",
        en: "Check that the project is coherent",
    },
)
.con(&[("accion", "doctor")])
.tomando(DE_PROYECTO);
pub(super) const SIMULAR: Paso = paso(
    "simular",
    Texto {
        es: "Simular el proyecto",
        en: "Simulate the project",
    },
)
.con(&[("accion", "ejecutar")])
.tomando(DE_PROYECTO);
/// The batch files `simular preparar` wrote, when it wrote them.
pub(super) const BATCH_PARAMS: Fuente = ("batch_params_file", &["batch_params_file"]);
pub(super) const BATCH_RVT: Fuente = ("batch_rvt_file", &["batch_rvt_file"]);
/// Running the batch a `preparar` call left ready: without its files the run
/// would be a plain simulation and the batch would be lost.
pub(super) const SIMULAR_BATCH: Paso = paso(
    "simular",
    Texto {
        es: "Ejecutar el lote preparado",
        en: "Run the prepared batch",
    },
)
.con(&[("accion", "ejecutar")])
.tomando(DE_PROYECTO)
.tomando_si_esta(&[BATCH_PARAMS, BATCH_RVT]);
pub(super) const VER_KPIS: Paso = paso(
    "resultados",
    Texto {
        es: "Ver los indicadores de la simulación",
        en: "See the simulation indicators",
    },
)
.con(&[("que", "kpis")])
.tomando(DE_PROYECTO);
pub(super) const VER_PERDIDAS: Paso = paso(
    "resultados",
    Texto {
        es: "Ver el desglose de pérdidas",
        en: "See the loss breakdown",
    },
)
.con(&[("que", "perdidas")])
.tomando(DE_PROYECTO);
pub(super) const P50_P90: Paso = paso(
    "resultados",
    Texto {
        es: "Ver la producción P50/P90",
        en: "See the P50/P90 production",
    },
)
.con(&[("que", "p50_p90")])
.tomando(DE_PROYECTO);

/// Every entry, grouped by feature.
const GRUPOS: &[&[Entrada]] = &[
    entorno::ENTRADAS,
    proyectos::ENTRADAS,
    componentes::ENTRADAS,
    sitios::ENTRADAS,
    resultados::ENTRADAS,
    workspace::ENTRADAS,
    gui::ENTRADAS,
];

/// All entries. O(n) over ~110 entries per lookup, once per tool call.
pub(crate) fn todas() -> impl Iterator<Item = &'static Entrada> {
    GRUPOS.iter().flat_map(|g| g.iter())
}

/// Entry of `tool` with discriminator value `accion` (`""` for tools without one).
pub(crate) fn buscar(tool: &str, accion: &str) -> Option<&'static Entrada> {
    todas().find(|e| e.tool == tool && e.accion == accion)
}
