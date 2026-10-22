//! Buttons of the project-window forms that open a further dialog (second
//! level). Each needs its form open first (`requiere`), and some sit on a tab
//! page (`en_pagina`). Observed live on PVsyst 8.1.7 with a demo project; the
//! ones that opened nothing say why in `nota`.

use crate::gui_catalogo::dominio::Funcion;

pub(super) const FUNCIONES: &[Funcion] = &[
    Funcion::boton("variante.orientacion.detalles_y_grafico", "TOrientationForm", "Detalles y gráfico")
        .abre("TShedElecShadingForm")
        .requiere("variante.orientacion"),
    Funcion::boton("variante.sistema.diagrama_unifilar", "TSystemegridForm", "Diagrama unifilar")
        .abre("TSystemCircuitForm")
        .requiere("variante.sistema"),
    Funcion::boton("variante.sistema.dimensionamiento", "TSystemegridForm", "Dimensionamiento")
        .abre("TArraySizingForm")
        .requiere("variante.sistema"),
    Funcion::boton("variante.sistema.abrir", "TSystemegridForm", "Abrir")
        .abre("TGridInverterForm")
        .requiere("variante.sistema"),
    Funcion::boton("variante.perdidas_detalladas.grafico_de_perdidas", "TDetailedLossesForm", "Gráfico de pérdidas")
        .abre("TArrayLossesPlotForm")
        .requiere("variante.perdidas_detalladas")
        .en_pagina(0),
    Funcion::boton("variante.perdidas_detalladas.ver_el_tnco_de_todos_modos", "TDetailedLossesForm", "Ver el TNCO de todos modos")
        .requiere("variante.perdidas_detalladas")
        .en_pagina(0)
        .nota("no abrió ventana en el estado probado (muestra información dentro del propio formulario)"),
    Funcion::boton("variante.perdidas_detalladas.diagrama_unifilar", "TDetailedLossesForm", "Diagrama unifilar")
        .abre("TSystemCircuitForm")
        .requiere("variante.perdidas_detalladas")
        .en_pagina(1),
    Funcion::boton("variante.perdidas_detalladas.computacion_detallada", "TDetailedLossesForm", "Computación detallada")
        .abre("TWiringResistanceForm")
        .requiere("variante.perdidas_detalladas")
        .en_pagina(1),
    Funcion::boton("variante.perdidas_detalladas.estudio_detallado", "TDetailedLossesForm", "Estudio detallado")
        .abre("TMismatchStringForm")
        .requiere("variante.perdidas_detalladas")
        .en_pagina(2),
    Funcion::boton("variante.perdidas_detalladas.eleja_archivo_csv", "TDetailedLossesForm", "Eleja archivo CSV")
        .requiere("variante.perdidas_detalladas")
        .en_pagina(3)
        .nota("el botón estaba deshabilitado en el estado probado (necesita activar la pérdida de suciedad desde archivo): no se pudo observar qué abre"),
    Funcion::boton("variante.perdidas_detalladas.leer_modelo", "TDetailedLossesForm", "Leer modelo")
        .requiere("variante.perdidas_detalladas")
        .en_pagina(6)
        .nota("el botón estaba deshabilitado en el estado probado (necesita un modelo de envejecimiento activo): no se pudo observar qué abre"),
    Funcion::boton("variante.perdidas_detalladas.exportar_como_archivo", "TDetailedLossesForm", "Exportar como archivo")
        .requiere("variante.perdidas_detalladas")
        .en_pagina(7)
        .nota("el botón estaba deshabilitado en el estado probado (necesita una indisponibilidad definida): no se pudo observar qué abre"),
    Funcion::boton("variante.autoconsumo.carga", "TUserNeedsForm", "Carga")
        .abre("TChoixObjetPVForm")
        .requiere("variante.autoconsumo"),
    Funcion::boton("variante.horizonte.leer_importar", "THorizonForm", "Leer / Importar")
        .abre("TImportHorizonForm")
        .requiere("variante.horizonte")
        .en_pagina(0),
    Funcion::boton("variante.sombreados_cercanos.construccion_perspectiva", "TShadingsForm", "Construcción / Perspectiva")
        .abre("TPerspectiveForm")
        .requiere("variante.sombreados_cercanos"),
    Funcion::boton("variante.sombreados_cercanos.orientaciones", "TShadingsForm", "Orientaciones")
        .abre("TOrientationForm")
        .requiere("variante.sombreados_cercanos"),
    Funcion::boton("variante.sombreados_cercanos.grafico", "TShadingsForm", "Gráfico")
        .requiere("variante.sombreados_cercanos")
        .nota("el botón estaba deshabilitado en el estado probado (necesita una escena 3D definida): no se pudo observar qué abre"),
    Funcion::boton("variante.sombreados_cercanos.tabla", "TShadingsForm", "Tabla")
        .requiere("variante.sombreados_cercanos")
        .nota("el botón estaba deshabilitado en el estado probado (necesita una escena 3D definida): no se pudo observar qué abre"),
    Funcion::boton("variante.sombreados_cercanos.importar", "TShadingsForm", "Importar")
        .abre("TChoixObjetPVForm")
        .requiere("variante.sombreados_cercanos"),
    Funcion::boton("variante.evaluacion_economica.tasas", "TEcoEvaluationForm", "Tasas")
        .abre("TCurrenciesForm")
        .requiere("variante.evaluacion_economica")
        .en_pagina(0),
    Funcion::boton("variante.evaluacion_economica.flujo_de_caja_anual", "TEcoEvaluationForm", "Flujo de caja anual")
        .requiere("variante.evaluacion_economica")
        .en_pagina(3)
        .nota("no abrió ventana en el estado probado (el gráfico se dibuja con resultados de simulación; sin ejecutar la simulación no hay datos)"),
    Funcion::boton("variante.evaluacion_economica.resultados_detallados", "TEcoEvaluationForm", "Resultados detallados")
        .requiere("variante.evaluacion_economica")
        .en_pagina(3)
        .nota("no abrió ventana en el estado probado (el gráfico se dibuja con resultados de simulación; sin ejecutar la simulación no hay datos)"),
    Funcion::boton("variante.evaluacion_economica.asignacion_de_ingresos", "TEcoEvaluationForm", "Asignación de ingresos")
        .requiere("variante.evaluacion_economica")
        .en_pagina(3)
        .nota("no abrió ventana en el estado probado (el gráfico se dibuja con resultados de simulación; sin ejecutar la simulación no hay datos)"),
    Funcion::boton("variante.evaluacion_economica.flujo_de_caja_acumulativo", "TEcoEvaluationForm", "Flujo de caja acumulativo")
        .requiere("variante.evaluacion_economica")
        .en_pagina(3)
        .nota("no abrió ventana en el estado probado (el gráfico se dibuja con resultados de simulación; sin ejecutar la simulación no hay datos)"),
    Funcion::boton("variante.simulacion_avanzada.herramienta_de_envejecimiento", "TAdvancedSimForm", "Herramienta de envejecimiento")
        .abre("TAgingToolForm")
        .requiere("variante.simulacion_avanzada"),
    Funcion::boton("variante.simulacion_avanzada.simulacion_por_lotes", "TAdvancedSimForm", "Simulación por lotes")
        .abre("TBatchSimulForm")
        .requiere("variante.simulacion_avanzada"),
    Funcion::boton("variante.simulacion_avanzada.graficos_especiales", "TAdvancedSimForm", "Gráficos especiales")
        .abre("TSimulPlotForm")
        .requiere("variante.simulacion_avanzada"),
    Funcion::boton("variante.simulacion_avanzada.almacen_de_datos_por_hora", "TAdvancedSimForm", "Almacen. de datos por hora")
        .abre("TSimulValHorForm")
        .requiere("variante.simulacion_avanzada"),
    Funcion::boton("variante.simulacion_avanzada.archivo_de_salida", "TAdvancedSimForm", "Archivo de salida")
        .abre("TSimulFichForm")
        .requiere("variante.simulacion_avanzada"),
    Funcion::boton("variante.simulacion_avanzada.comparaciones", "TAdvancedSimForm", "Comparaciones")
        .requiere("variante.simulacion_avanzada")
        .nota("el botón estaba deshabilitado en el estado probado (necesita resultados de simulación): no se pudo observar qué abre"),
    Funcion::boton("variante.simulacion_avanzada.herramienta_de_optimizacion", "TAdvancedSimForm", "Herramienta de optimización")
        .abre("TOptimizationToolForm")
        .requiere("variante.simulacion_avanzada"),
    Funcion::boton("variante.resultados_detallados.diagrama_de_perdida", "TResultForm", "Diagrama de pérdida")
        .abre("TPlotLossDiagramF")
        .requiere("variante.resultados_detallados"),
    Funcion::boton("variante.resultados_detallados.tablas", "TResultForm", "Tablas")
        .abre("TResultTabForm")
        .requiere("variante.resultados_detallados"),
    Funcion::boton("variante.resultados_detallados.graficos_predefinidos", "TResultForm", "Gráficos predefinidos")
        .abre("TResultPlotForm")
        .requiere("variante.resultados_detallados"),
    Funcion::boton("variante.resultados_detallados.graficos_por_hora", "TResultForm", "Gráficos por hora")
        .abre("TResultPlotHorForm")
        .requiere("variante.resultados_detallados"),
];
