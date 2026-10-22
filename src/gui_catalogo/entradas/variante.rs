//! Project window (`TProjectForm`): the buttons that open each part of a
//! simulation variant. They need a project open (`proyecto.cargar`, or a
//! recent project through `gui_actuar accion:menu`).

use crate::gui_catalogo::dominio::{Efecto, Funcion};

const PROYECTO: &str = "TProjectForm";

pub(super) const FUNCIONES: &[Funcion] = &[
    Funcion::boton("variante.orientacion", PROYECTO, "Orientación").abre("TOrientationForm"),
    Funcion::boton("variante.sistema", PROYECTO, "Sistema").abre("TSystemegridForm"),
    Funcion::boton(
        "variante.perdidas_detalladas",
        PROYECTO,
        "Pérdidas detalladas",
    )
    .abre("TDetailedLossesForm"),
    Funcion::boton("variante.autoconsumo", PROYECTO, "Autoconsumo").abre("TUserNeedsForm"),
    Funcion::boton("variante.almacenamiento", PROYECTO, "Almacenamiento").abre("TGridStorageForm"),
    Funcion::boton("variante.horizonte", PROYECTO, "Horizonte").abre("THorizonForm"),
    Funcion::boton(
        "variante.sombreados_cercanos",
        PROYECTO,
        "Sombreados cercanos",
    )
    .abre("TShadingsForm"),
    Funcion::boton(
        "variante.gestion_energia",
        PROYECTO,
        "Gestión de la energía",
    )
    .abre("TSystemeAddForm"),
    Funcion::boton(
        "variante.evaluacion_economica",
        PROYECTO,
        "Evaluación económica",
    )
    .abre("TEcoEvaluationForm"),
    Funcion::boton(
        "variante.simulacion_avanzada",
        PROYECTO,
        "Simulación avanzada",
    )
    .abre("TAdvancedSimForm"),
    Funcion::boton(
        "variante.resultados_detallados",
        PROYECTO,
        "Resultados detallados",
    )
    .abre("TResultForm"),
    Funcion::boton("variante.resumen_sistema", PROYECTO, "Resumen del sistema")
        .abre("TSummaryForm"),
    Funcion::boton(
        "variante.ejecutar_simulacion",
        PROYECTO,
        "Ejecutar simulación",
    )
    .con_efecto(Efecto::Modifica),
    Funcion::boton("variante.informe", PROYECTO, "Informe").con_efecto(Efecto::Modifica),
];
