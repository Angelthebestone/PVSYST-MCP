//! `simular` and `resultados`.

use super::{entrada, paso, Entrada, Paso, DE_PROYECTO};

const VER_PROYECTOS: Paso =
    paso("listar", "Ver los proyectos del workspace").con(&[("que", "proyectos")]);
const VER_ESTADO: Paso = paso(
    "simular",
    "Consultar el estado del trabajo en segundo plano",
)
.con(&[("accion", "estado")])
.tomando(&[("trabajo", &["trabajo", "job_id"])]);
const SIMULAR: Paso = paso("simular", "Simular el proyecto")
    .con(&[("accion", "ejecutar")])
    .tomando(DE_PROYECTO);
const VER_KPIS: Paso = paso("resultados", "Ver los indicadores de la simulación")
    .con(&[("que", "kpis")])
    .tomando(DE_PROYECTO);
const VER_PERDIDAS: Paso = paso("resultados", "Ver el desglose de pérdidas")
    .con(&[("que", "perdidas")])
    .tomando(DE_PROYECTO);
const VER_MENSUAL: Paso = paso("resultados", "Ver la producción mes a mes")
    .con(&[("que", "mensual")])
    .tomando(DE_PROYECTO);
const VER_P50_P90: Paso = paso("resultados", "Ver la producción P50/P90")
    .con(&[("que", "p50_p90")])
    .tomando(DE_PROYECTO);
const COMPARAR: Paso = paso("resultados", "Comparar variantes del proyecto")
    .con(&[("que", "comparar")])
    .tomando(DE_PROYECTO);

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
    .luego(&[VER_KPIS, VER_PROYECTOS]),
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
    entrada("resultados", "csv", "Resultados horarios leídos del CSV.")
        .luego(&[VER_KPIS, VER_PROYECTOS]),
    entrada(
        "resultados",
        "perdidas",
        "Desglose de pérdidas del sistema leído.",
    )
    .luego(&[VER_MENSUAL, VER_PROYECTOS]),
    entrada("resultados", "mensual", "Producción mensual leída.")
        .luego(&[VER_P50_P90, VER_PROYECTOS]),
    entrada(
        "resultados",
        "comparar",
        "Variantes del proyecto comparadas.",
    )
    .luego(&[VER_KPIS, VER_PROYECTOS]),
    entrada(
        "resultados",
        "medido_vs_simulado",
        "Datos medidos comparados con la simulación.",
    )
    .luego(&[VER_KPIS, VER_PROYECTOS]),
    entrada(
        "resultados",
        "p50_p90",
        "Producción P50/P90 calculada de la simulación.",
    )
    .luego(&[VER_KPIS, VER_PROYECTOS]),
    entrada("resultados", "carbono", "Balance de carbono calculado.")
        .luego(&[VER_KPIS, VER_PROYECTOS]),
    entrada("resultados", "economia", "Resultados económicos leídos.")
        .luego(&[VER_KPIS, VER_PROYECTOS]),
    entrada(
        "resultados",
        "entregable",
        "Entregable del proyecto generado.",
    )
    .luego(&[VER_KPIS, VER_PROYECTOS]),
    entrada(
        "resultados",
        "resumen",
        "Resumen de resultados del proyecto.",
    )
    .luego(&[VER_PERDIDAS, VER_PROYECTOS]),
];
