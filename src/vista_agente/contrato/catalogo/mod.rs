//! What each tool action does and where to go next, as data. One entry per
//! `tool` × `accion`; a test fails when a declared action has no entry.

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
/// carries a placeholder.
#[derive(Debug, Clone, Copy)]
pub(crate) struct Paso {
    pub(crate) tool: &'static str,
    pub(crate) motivo: &'static str,
    pub(crate) fijos: &'static [(&'static str, &'static str)],
    pub(crate) tomar: &'static [Fuente],
}

pub(super) const fn paso(tool: &'static str, motivo: &'static str) -> Paso {
    Paso {
        tool,
        motivo,
        fijos: &[],
        tomar: &[],
    }
}

impl Paso {
    pub(super) const fn con(self, fijos: &'static [(&'static str, &'static str)]) -> Self {
        Paso { fijos, ..self }
    }

    pub(super) const fn tomando(self, tomar: &'static [Fuente]) -> Self {
        Paso { tomar, ..self }
    }
}

/// What one action reports and suggests. `pista` is the short text kept under
/// the top-level `siguiente` key for callers that already read it.
#[derive(Debug, Clone, Copy)]
pub(crate) struct Entrada {
    pub(crate) tool: &'static str,
    pub(crate) accion: &'static str,
    pub(crate) resumen: &'static str,
    pub(crate) pista: Option<&'static str>,
    pub(crate) siguiente: &'static [Paso],
}

pub(super) const fn entrada(
    tool: &'static str,
    accion: &'static str,
    resumen: &'static str,
) -> Entrada {
    Entrada {
        tool,
        accion,
        resumen,
        pista: None,
        siguiente: &[],
    }
}

impl Entrada {
    pub(super) const fn pista(self, pista: &'static str) -> Self {
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
pub(crate) const DIAGNOSTICO: Paso =
    paso("entorno", "Comprobar la instalación, la CLI y el workspace")
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
pub(super) const VER_PROYECTOS: Paso =
    paso("listar", "Ver los proyectos del workspace").con(&[("que", "proyectos")]);
pub(super) const VER_ARCHIVOS: Paso =
    paso("listar", "Ver los archivos del workspace").con(&[("que", "archivos")]);
/// The components of the kind the request or the answer names (`PAN`, `OND`...).
pub(super) const TIPO: Fuente = ("tipo", &["tipo", "kind"]);
pub(super) const VER_COMPONENTES: Paso = paso("listar", "Ver los componentes de ese tipo")
    .con(&[("que", "componentes")])
    .tomando(&[TIPO]);
pub(super) const VER_MODULOS: Paso =
    paso("listar", "Ver los módulos disponibles").con(&[("que", "modulos")]);
pub(super) const VER_HISTORIAL: Paso = paso(
    "workspace",
    "Ver el historial de cambios para poder revertir",
)
.con(&[("accion", "historial")]);
pub(super) const REVISAR: Paso = paso("proyecto", "Revisar que el proyecto es coherente")
    .con(&[("accion", "doctor")])
    .tomando(DE_PROYECTO);
pub(super) const SIMULAR: Paso = paso("simular", "Simular el proyecto")
    .con(&[("accion", "ejecutar")])
    .tomando(DE_PROYECTO);
pub(super) const VER_KPIS: Paso = paso("resultados", "Ver los indicadores de la simulación")
    .con(&[("que", "kpis")])
    .tomando(DE_PROYECTO);
pub(super) const VER_PERDIDAS: Paso = paso("resultados", "Ver el desglose de pérdidas")
    .con(&[("que", "perdidas")])
    .tomando(DE_PROYECTO);
pub(super) const P50_P90: Paso = paso("resultados", "Ver la producción P50/P90")
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
