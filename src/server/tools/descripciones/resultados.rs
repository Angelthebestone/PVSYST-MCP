//! Argument descriptions of `resultados`.

use super::{entrada, Entrada};

pub(super) const TABLA: &[Entrada] = &[
    entrada(
        "count",
        "Cuántos CSV devuelve que:csv, del más reciente atrás (por defecto 1).",
        "How many CSVs que:csv returns, newest first (default 1).",
    ),
    entrada(
        "dir",
        "Con que:csv, carpeta donde se buscan los resultados (por defecto Results).",
        "With que:csv, folder searched for the results (default Results).",
    ),
    entrada(
        "include_text",
        "true añade el texto del CSV, hasta 32768 bytes; solo con que:csv.",
        "true adds the CSV text, up to 32768 bytes; only with que:csv.",
    ),
    entrada(
        "internet",
        "true: sigma_meteo_pct sale de PVGIS por internet (p50_p90, entregable).",
        "true: sigma_meteo_pct comes from PVGIS over the internet (p50_p90, entregable).",
    ),
    entrada(
        "kwp",
        "Potencia del campo en kWp para las cifras por kWp; si falta, se lee del informe.",
        "Array power in kWp for per-kWp figures; when absent it is read from the report.",
    ),
    entrada(
        "medido_csv",
        "CSV con la energía mensual medida (medido_vs_simulado); alternativa a medido_kwh.",
        "CSV with the measured monthly energy (medido_vs_simulado); alternative to medido_kwh.",
    ),
    entrada(
        "medido_kwh",
        "12 energías mensuales medidas en kWh, enero a diciembre (medido_vs_simulado).",
        "12 measured monthly energies in kWh, January to December (medido_vs_simulado).",
    ),
    entrada(
        "sigma_meteo_pct",
        "Variabilidad interanual del recurso en % (0-100) para p50_p90 y entregable; 5 por defecto o PVGIS.",
        "Resource interannual variability in % (0-100) for p50_p90 and entregable; default 5 or PVGIS.",
    ),
    entrada(
        "sigma_modelo_pct",
        "Incertidumbre del modelo en % (0-100) para p50_p90 y entregable; por defecto 3.",
        "Model uncertainty in % (0-100) for p50_p90 and entregable; default 3.",
    ),
    entrada(
        "umbral_pct",
        "Desviación mensual en % desde la que un mes entra en fuera_de_umbral; 10 por defecto.",
        "Monthly deviation in % above which a month enters fuera_de_umbral; default 10.",
    ),
];
