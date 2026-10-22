//! Declarative tables of PVsyst GUI captions and window classes for project features.

use crate::adapters::gui::rotulos::Idioma;

/// Captions and classes for project pre-sizing.
pub(crate) struct RotulosDimensionado {
    pub(crate) sistema: &'static str,
    pub(crate) formulario_sistema: &'static str,
    pub(crate) ok: &'static str,
}

pub(crate) const fn rotulos_dimensionado(idioma: Idioma) -> RotulosDimensionado {
    match idioma {
        Idioma::Es => RotulosDimensionado {
            sistema: "Sistema",
            formulario_sistema: "TSystemegridForm",
            ok: "OK",
        },
    }
}

/// Captions and classes for horizon import.
pub(crate) struct RotulosHorizonte {
    pub(crate) horizonte: &'static str,
    pub(crate) formulario_horizonte: &'static str,
    pub(crate) leer_importar: &'static str,
    pub(crate) formulario_importar: &'static str,
    pub(crate) opcion_pvgis: &'static str,
    pub(crate) importar_desde_web: &'static str,
    pub(crate) ok: &'static str,
}

pub(crate) const fn rotulos_horizonte(idioma: Idioma) -> RotulosHorizonte {
    match idioma {
        Idioma::Es => RotulosHorizonte {
            horizonte: "Horizonte",
            formulario_horizonte: "THorizonForm",
            leer_importar: "Leer / Importar",
            formulario_importar: "TImportHorizonForm",
            opcion_pvgis: "PVGIS",
            importar_desde_web: "Importar desde web",
            ok: "OK",
        },
    }
}

/// Captions and classes for P50-P90 energy management setup.
pub(crate) struct RotulosP50 {
    pub(crate) gestion_energia: &'static str,
    pub(crate) formulario_energia: &'static str,
    pub(crate) pestana: &'static str,
    pub(crate) ok: &'static str,
}

pub(crate) const fn rotulos_p50(idioma: Idioma) -> RotulosP50 {
    match idioma {
        Idioma::Es => RotulosP50 {
            gestion_energia: "Gestión de la energía",
            formulario_energia: "TSystemeAddForm",
            pestana: "Estimación P50 - P90",
            ok: "OK",
        },
    }
}

/// Captions and classes for project window lifecycle.
pub(crate) struct RotulosProyectoGui {
    pub(crate) formulario_cargar: &'static str,
    pub(crate) ok: &'static str,
    pub(crate) formulario_proyecto: &'static str,
    pub(crate) formulario_guardar: &'static str,
    pub(crate) guardar: &'static str,
    pub(crate) inicio_red: &'static str,
    pub(crate) inicio_aislado: &'static str,
    pub(crate) inicio_bombeo: &'static str,
    pub(crate) inicio_red_dc: &'static str,
}

pub(crate) const fn rotulos_proyecto_gui(idioma: Idioma) -> RotulosProyectoGui {
    match idioma {
        Idioma::Es => RotulosProyectoGui {
            formulario_cargar: "TProjectLoadForm",
            ok: "OK",
            formulario_proyecto: "TProjectForm",
            formulario_guardar: "TSaveObjPVForm",
            guardar: "Guardar",
            inicio_red: "Conectado a la red",
            inicio_aislado: "Independiente",
            inicio_bombeo: "Bombeo",
            inicio_red_dc: "Red DC",
        },
    }
}
