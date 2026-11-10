//! `simular` and `resultados`.

use super::{
    entrada, paso, Entrada, Paso, DE_PROYECTO, P50_P90, SIMULAR, SIMULAR_BATCH, VER_KPIS,
    VER_PERDIDAS, VER_PROYECTOS,
};
use crate::idioma::Texto;

const VER_ESTADO: Paso = paso(
    "simular",
    Texto {
        es: "Consultar el estado del trabajo en segundo plano",
        en: "Check the background job status",
    },
)
.con(&[("accion", "estado")])
.tomando(&[("trabajo", &["trabajo", "job_id"])]);
const VER_MENSUAL: Paso = paso(
    "resultados",
    Texto {
        es: "Ver la producción mes a mes",
        en: "See the month-by-month production",
    },
)
.con(&[("que", "mensual")])
.tomando(DE_PROYECTO);
const COMPARAR: Paso = paso(
    "resultados",
    Texto {
        es: "Comparar variantes del proyecto",
        en: "Compare the project variants",
    },
)
.con(&[("que", "comparar")])
.tomando(DE_PROYECTO);

/// What follows a result that changed a project: its indicators, or the project list.
const TRAS_RESULTADO: &[Paso] = &[VER_KPIS, VER_PROYECTOS];

pub(super) const ENTRADAS: &[Entrada] = &[
    entrada(
        "simular",
        "ejecutar",
        Texto {
            es: "Simulación ejecutada con PVsystCLI.",
            en: "Simulation run with PVsystCLI.",
        },
    )
    .pista(Texto {
        es: "resultados que:perdidas",
        en: "resultados que:perdidas",
    })
    .luego(&[VER_KPIS, VER_PERDIDAS, VER_ESTADO, VER_PROYECTOS]),
    entrada(
        "simular",
        "preparar",
        Texto {
            es: "Simulación preparada sin ejecutarla.",
            en: "Simulation prepared without running it.",
        },
    )
    .luego(&[SIMULAR_BATCH, VER_ESTADO, VER_PROYECTOS]),
    entrada(
        "simular",
        "estado",
        Texto {
            es: "Estado de las simulaciones consultado.",
            en: "Simulation status checked.",
        },
    )
    .luego(TRAS_RESULTADO),
    entrada(
        "simular",
        "preestimar",
        Texto {
            es: "Producción estimada antes de simular.",
            en: "Production estimated before simulating.",
        },
    )
    .luego(&[SIMULAR, VER_ESTADO, VER_PROYECTOS]),
    entrada(
        "simular",
        "barrido",
        Texto {
            es: "Barrido de simulaciones ejecutado.",
            en: "Sweep of simulations run.",
        },
    )
    .luego(&[COMPARAR, VER_ESTADO, VER_PROYECTOS]),
    entrada(
        "resultados",
        "kpis",
        Texto {
            es: "Indicadores de la simulación leídos del informe.",
            en: "Simulation indicators read from the report.",
        },
    )
    .pista(Texto {
        es: "resultados que:perdidas",
        en: "resultados que:perdidas",
    })
    .luego(&[VER_PERDIDAS, VER_PROYECTOS]),
    entrada(
        "resultados",
        "csv",
        Texto {
            es: "Resultados horarios leídos del CSV.",
            en: "Hourly results read from the CSV.",
        },
    )
    .luego(TRAS_RESULTADO),
    entrada(
        "resultados",
        "perdidas",
        Texto {
            es: "Desglose de pérdidas del sistema leído.",
            en: "System loss breakdown read.",
        },
    )
    .luego(&[VER_MENSUAL, VER_PROYECTOS]),
    entrada(
        "resultados",
        "mensual",
        Texto {
            es: "Producción mensual leída.",
            en: "Monthly production read.",
        },
    )
    .luego(&[P50_P90, VER_PROYECTOS]),
    entrada(
        "resultados",
        "comparar",
        Texto {
            es: "Variantes del proyecto comparadas.",
            en: "Project variants compared.",
        },
    )
    .luego(TRAS_RESULTADO),
    entrada(
        "resultados",
        "medido_vs_simulado",
        Texto {
            es: "Datos medidos comparados con la simulación.",
            en: "Measured data compared with the simulation.",
        },
    )
    .luego(TRAS_RESULTADO),
    entrada(
        "resultados",
        "p50_p90",
        Texto {
            es: "Producción P50/P90 calculada de la simulación.",
            en: "P50/P90 production computed from the simulation.",
        },
    )
    .luego(TRAS_RESULTADO),
    entrada(
        "resultados",
        "carbono",
        Texto {
            es: "Balance de carbono calculado.",
            en: "Carbon balance computed.",
        },
    )
    .luego(TRAS_RESULTADO),
    entrada(
        "resultados",
        "economia",
        Texto {
            es: "Resultados económicos leídos.",
            en: "Economic results read.",
        },
    )
    .luego(TRAS_RESULTADO),
    entrada(
        "resultados",
        "entregable",
        Texto {
            es: "Entregable del proyecto generado.",
            en: "Project deliverable generated.",
        },
    )
    .luego(TRAS_RESULTADO),
    entrada(
        "resultados",
        "resumen",
        Texto {
            es: "Resumen de resultados del proyecto.",
            en: "Summary of the project results.",
        },
    )
    .luego(&[VER_PERDIDAS, VER_PROYECTOS]),
];
