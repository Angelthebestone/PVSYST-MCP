//! Argument descriptions of `simular`.

use super::{entrada, Entrada};

pub(super) const TABLA: &[Entrada] = &[
    entrada(
        "abrir_gui",
        "Abre PVsyst con el proyecto al terminar; sin PVsystCLI, abre la GUI en vez de simular.",
        "Opens PVsyst with the project afterwards; without PVsystCLI it opens the GUI instead of simulating.",
    ),
    entrada(
        "archivo_rvt",
        "Plantilla .RVT de accion:preparar; equivale a batch_rvt_file.",
        "RVT template of accion:preparar; same as batch_rvt_file.",
    ),
    entrada(
        "azimut",
        "Azimut en grados (-180 a 180; 0 = sur, este negativo).",
        "Azimuth in degrees (-180 to 180; 0 = south, east negative).",
    ),
    entrada(
        "confirmar",
        "true acepta el gasto de ejecuciones de prueba que el aviso de presupuesto retuvo.",
        "true accepts the trial-run spend that the budget warning held back.",
    ),
    entrada(
        "descripciones",
        "Texto por parámetro para la columna de descripción de Param_Modif.dat.",
        "Text per parameter for the description column of Param_Modif.dat.",
    ),
    entrada(
        "end_date",
        "Fin del periodo simulado (-ed): YYYY-MM-DD, YY-MM-DD o MM-DD.",
        "End of the simulated period (-ed): YYYY-MM-DD, YY-MM-DD or MM-DD.",
    ),
    entrada(
        "extra_options",
        "Opciones de PVsystCLI que se añaden tal cual; no pueden redefinir las que pone simular.",
        "PVsystCLI options appended verbatim; they may not redefine the ones simular sets.",
    ),
    entrada(
        "idioma_reporte",
        "Idioma del informe PDF (-rl): en, fr, de, es, it, pt, tr, ko, zh, ja, pl o ru.",
        "PDF report language (-rl): en, fr, de, es, it, pt, tr, ko, zh, ja, pl or ru.",
    ),
    entrada(
        "iniciales",
        "Valor inicial por parámetro en Param_Modif.dat; por defecto 0.0000.",
        "Initial value per parameter in Param_Modif.dat; defaults to 0.0000.",
    ),
    entrada(
        "kwp",
        "Potencia de la planta en kWp, mayor que 0.",
        "Plant power in kWp, greater than 0.",
    ),
    entrada(
        "log_level",
        "Nivel de log de PVsystCLI (-ll): 0 OFF, 1 ERROR, 2 WARNING, 3 INFO.",
        "PVsystCLI log level (-ll): 0 OFF, 1 ERROR, 2 WARNING, 3 INFO.",
    ),
    entrada(
        "n",
        "Cuántas de las mejores orientaciones se simulan (por defecto 3).",
        "How many of the best orientations are simulated (default 3).",
    ),
    entrada(
        "operacion",
        "Con accion:preparar sobre un .RVT: listar, anadir, eliminar, reordenar, crear o catalogo.",
        "With accion:preparar on an .RVT: listar, anadir, eliminar, reordenar, crear or catalogo.",
    ),
    entrada(
        "overwrite_default_csv",
        "Bandera -odc: sobrescribe el CSV de resultados por defecto del proyecto.",
        "-odc flag: overwrites the project default results CSV.",
    ),
    entrada(
        "paginas_reporte",
        "Páginas del informe PDF (-rp), separadas por coma: results, losses, p50p90...",
        "PDF report pages (-rp), comma separated: results, losses, p50p90...",
    ),
    entrada(
        "recalcular_sombreado",
        "Recalcula las tablas de factores de sombreado (-rst: 0 o 1); PVsyst usa 1.",
        "Recomputes the shading factor tables (-rst: 0 or 1); PVsyst uses 1.",
    ),
    entrada(
        "segundo_plano",
        "Lanza la simulación en segundo plano y devuelve un id para accion:estado.",
        "Runs the simulation in the background and returns an id for accion:estado.",
    ),
    entrada(
        "sin_cache",
        "No usa la caché de resultados: ejecuta PVsyst y lo marca en la respuesta.",
        "Skips the result cache: runs PVsyst and marks it in the response.",
    ),
    entrada(
        "start_date",
        "Inicio del periodo simulado (-sd): YYYY-MM-DD, YY-MM-DD o MM-DD.",
        "Start of the simulated period (-sd): YYYY-MM-DD, YY-MM-DD or MM-DD.",
    ),
    entrada(
        "synthetic_weather",
        "Genera la meteo horaria sintética con Meteonorm (-swg); solo con sitio (-s).",
        "Generates synthetic hourly weather with Meteonorm (-swg); only with a site (-s).",
    ),
    entrada(
        "tilt",
        "Inclinación en grados (0 a 90); con solo azimut vale |latitud|.",
        "Tilt in degrees (0 to 90); with only azimut it is |latitude|.",
    ),
    entrada(
        "time_step",
        "Paso de cálculo (-ts): hour o subhour; subhour exige meteo subhoraria.",
        "Calculation step (-ts): hour or subhour; subhour needs sub-hourly weather.",
    ),
    entrada(
        "unidades",
        "Unidad por parámetro para Param_Modif.dat; texto sin punto y coma.",
        "Unit per parameter for Param_Modif.dat; text without semicolons.",
    ),
    entrada(
        "variables",
        "Variables de resultado .RVT: con filas genera el lote; con accion:preparar edita la plantilla.",
        "RVT result variables: with filas it builds the batch; with accion:preparar it edits the template.",
    ),
];
