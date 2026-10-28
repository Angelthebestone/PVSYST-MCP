//! `simular` and `resultados`.

use super::{
    entrada, paso, Entrada, Paso, DE_PROYECTO, P50_P90, SIMULAR, VER_KPIS, VER_PERDIDAS,
    VER_PROYECTOS,
};

const VER_ESTADO: Paso = paso(
    "simular",
    "Consultar el estado del trabajo en segundo plano",
)
.con(&[("accion", "estado")])
.tomando(&[("trabajo", &["trabajo", "job_id"])]);
const VER_MENSUAL: Paso = paso("resultados", "Ver la producción mes a mes")
    .con(&[("que", "mensual")])
    .tomando(DE_PROYECTO);
const COMPARAR: Paso = paso("resultados", "Comparar variantes del proyecto")
    .con(&[("que", "comparar")])
    .tomando(DE_PROYECTO);

/// What follows a result that changed a project: its indicators, or the project list.
const TRAS_RESULTADO: &[Paso] = &[VER_KPIS, VER_PROYECTOS];

pub(super) const ENTRADAS: &[Entrada] = &[
    entrada("simular", "ejecutar", "Simulación ejecutada con PVsystCLI.")
        .pista("resultados que:perdidas")
        .luego(&[VER_KPIS, VER_PERDIDAS, VER_ESTADO, VER_PROYECTOS]),
    entrada(
        "simular",
        "preparar",
        "Simulación preparada sin ejecutarla.",
    )
    .luego(&[SIMULAR, VER_ESTADO, VER_PROYECTOS]),
    entrada(
        "simular",
        "estado",
        "Estado de las simulaciones consultado.",
    )
    .luego(TRAS_RESULTADO),
    entrada(
        "simular",
        "preestimar",
        "Producción estimada antes de simular.",
    )
    .luego(&[SIMULAR, VER_ESTADO, VER_PROYECTOS]),
    entrada("simular", "barrido", "Barrido de simulaciones ejecutado.").luego(&[
        COMPARAR,
        VER_ESTADO,
        VER_PROYECTOS,
    ]),
    entrada(
        "resultados",
        "kpis",
        "Indicadores de la simulación leídos del informe.",
    )
    .pista("resultados que:perdidas")
    .luego(&[VER_PERDIDAS, VER_PROYECTOS]),
    entrada("resultados", "csv", "Resultados horarios leídos del CSV.").luego(TRAS_RESULTADO),
    entrada(
        "resultados",
        "perdidas",
        "Desglose de pérdidas del sistema leído.",
    )
    .luego(&[VER_MENSUAL, VER_PROYECTOS]),
    entrada("resultados", "mensual", "Producción mensual leída.").luego(&[P50_P90, VER_PROYECTOS]),
    entrada(
        "resultados",
        "comparar",
        "Variantes del proyecto comparadas.",
    )
    .luego(TRAS_RESULTADO),
    entrada(
        "resultados",
        "medido_vs_simulado",
        "Datos medidos comparados con la simulación.",
    )
    .luego(TRAS_RESULTADO),
    entrada(
        "resultados",
        "p50_p90",
        "Producción P50/P90 calculada de la simulación.",
    )
    .luego(TRAS_RESULTADO),
    entrada("resultados", "carbono", "Balance de carbono calculado.").luego(TRAS_RESULTADO),
    entrada("resultados", "economia", "Resultados económicos leídos.").luego(TRAS_RESULTADO),
    entrada(
        "resultados",
        "entregable",
        "Entregable del proyecto generado.",
    )
    .luego(TRAS_RESULTADO),
    entrada(
        "resultados",
        "resumen",
        "Resumen de resultados del proyecto.",
    )
    .luego(&[VER_PERDIDAS, VER_PROYECTOS]),
];
