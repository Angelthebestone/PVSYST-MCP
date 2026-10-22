//! Windows opened from the start page's Utilidades: Herramientas, Bases de
//! datos and Datos medidos. Each button needs its parent window open first.

use crate::gui_catalogo::dominio::Funcion;

const HERRAMIENTAS: &str = "TToolsForm";
const BASES: &str = "TDatabaseForm";
const MEDIDOS: &str = "TDamMainForm";
const ELEGIR_OBJETO: &str = "TChoixObjetPVForm";
const TABLAS_METEO: &str = "TMeteoTablesGraphsForm";

pub(super) const FUNCIONES: &[Funcion] = &[
    Funcion::boton(
        "herramientas.optimizacion_voltaje",
        HERRAMIENTAS,
        "Optimización voltaje de funcionamiento",
    )
    .requiere("inicio.herramientas")
    .abre("TVoltageOptimiseForm"),
    Funcion::boton(
        "herramientas.comportamiento_electrico",
        HERRAMIENTAS,
        "Comportamiento eléctrico de los conjuntos FV",
    )
    .requiere("inicio.herramientas")
    .abre("TArrayCharactForm"),
    Funcion::boton(
        "herramientas.factor_transposicion",
        HERRAMIENTAS,
        "Factor de transposición",
    )
    .requiere("inicio.herramientas")
    .abre("TTranspositionForm"),
    Funcion::boton(
        "herramientas.calculo_mensual_meteo",
        HERRAMIENTAS,
        "Cálculo mensual de datos meteo",
    )
    .requiere("inicio.herramientas")
    .abre("TMeteoMensForm"),
    Funcion::boton(
        "herramientas.parametros_solares",
        HERRAMIENTAS,
        "Tablas/gráficos de parámetros solares",
    )
    .requiere("inicio.herramientas")
    .abre("TParamSolForm"),
    Funcion::boton("bases_de_datos.modulos_pv", BASES, "Módulos PV")
        .requiere("inicio.bases_de_datos")
        .abre(ELEGIR_OBJETO),
    Funcion::boton("bases_de_datos.inversores_red", BASES, "Inversor de red")
        .requiere("inicio.bases_de_datos")
        .abre(ELEGIR_OBJETO),
    Funcion::boton("bases_de_datos.baterias", BASES, "Baterías")
        .requiere("inicio.bases_de_datos")
        .abre(ELEGIR_OBJETO),
    Funcion::boton(
        "bases_de_datos.controladores_independientes",
        BASES,
        "Controladores para independientes",
    )
    .requiere("inicio.bases_de_datos")
    .abre(ELEGIR_OBJETO),
    Funcion::boton("bases_de_datos.bombas", BASES, "Bombas")
        .requiere("inicio.bases_de_datos")
        .abre(ELEGIR_OBJETO),
    Funcion::boton(
        "bases_de_datos.controladores_bombeo",
        BASES,
        "Controladores para bombeo",
    )
    .requiere("inicio.bases_de_datos")
    .abre(ELEGIR_OBJETO),
    Funcion::boton("bases_de_datos.generadores", BASES, "Generadores")
        .requiere("inicio.bases_de_datos")
        .abre(ELEGIR_OBJETO),
    Funcion::boton(
        "bases_de_datos.fabricantes_minoristas",
        BASES,
        "Fabricantes y minoristas",
    )
    .requiere("inicio.bases_de_datos")
    .abre(ELEGIR_OBJETO),
    Funcion::boton(
        "bases_de_datos.sitios_geograficos",
        BASES,
        "Sitios geográficos",
    )
    .requiere("inicio.bases_de_datos")
    .abre(ELEGIR_OBJETO),
    Funcion::boton("bases_de_datos.precios", BASES, "Precios")
        .requiere("inicio.bases_de_datos")
        .abre("TEcoPriceListForm"),
    Funcion::boton(
        "bases_de_datos.importar_archivo_personalizado",
        BASES,
        "Importar un archivo personalizado",
    )
    .requiere("inicio.bases_de_datos")
    .abre("TMetWizard"),
    Funcion::boton(
        "bases_de_datos.importar_formato_conocido",
        BASES,
        "Importar un formato conocido",
    )
    .requiere("inicio.bases_de_datos")
    .abre("TMeteoImportForm"),
    Funcion::boton(
        "bases_de_datos.comparar_meteo",
        BASES,
        "Comparar datos meteo",
    )
    .requiere("inicio.bases_de_datos")
    .abre("TMeteoCompareForm"),
    Funcion::boton(
        "bases_de_datos.tablas_graficos_meteo",
        BASES,
        "Tablas y gráficos de datos meteo",
    )
    .requiere("inicio.bases_de_datos")
    .abre(TABLAS_METEO),
    Funcion::boton(
        "bases_de_datos.datos_sinteticos",
        BASES,
        "Generación de datos sintéticos",
    )
    .requiere("inicio.bases_de_datos")
    .abre("TGenerSyntheForm"),
    Funcion::boton("bases_de_datos.generacion_tmy", BASES, "Generación de TMY")
        .requiere("inicio.bases_de_datos"),
    Funcion::boton(
        "datos_medidos.importar_archivo_personalizado",
        MEDIDOS,
        "Importar archivo personalizado",
    )
    .requiere("inicio.datos_medidos")
    .abre("TMeteoConversionForm"),
    Funcion::boton(
        "datos_medidos.tablas_graficos",
        MEDIDOS,
        "Tablas de datos y gráficos",
    )
    .requiere("inicio.datos_medidos")
    .abre(TABLAS_METEO),
    Funcion::boton(
        "datos_medidos.transformar_archivo",
        MEDIDOS,
        "Transformación archivo",
    )
    .requiere("inicio.datos_medidos")
    .abre("TDatamesFusionForm"),
    Funcion::boton("datos_medidos.analisis", MEDIDOS, "Análisis datos medidos")
        .requiere("inicio.datos_medidos")
        .abre("TProjectForm"),
];
