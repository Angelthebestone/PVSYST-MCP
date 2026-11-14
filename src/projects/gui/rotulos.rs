//! Window classes and captions of the project steps PVsyst's own windows perform.
//!
//! A class is the same in every language of PVsyst. A caption is a
//! [`Rotulo`](crate::adapters::gui::rotulos::Rotulo) that matches in any of
//! them; the captions themselves live in `adapters::gui::rotulos`.

/// Pre-sizing in the System window.
pub(crate) const FORMULARIO_SISTEMA: &str = "TSystemegridForm";

/// Horizon import.
pub(crate) const FORMULARIO_HORIZONTE: &str = "THorizonForm";
pub(crate) const FORMULARIO_IMPORTAR_HORIZONTE: &str = "TImportHorizonForm";

/// P50-P90 energy management set-up.
pub(crate) const FORMULARIO_ENERGIA: &str = "TSystemeAddForm";

/// Project window lifecycle.
pub(crate) const FORMULARIO_CARGAR: &str = "TProjectLoadForm";
pub(crate) const FORMULARIO_PROYECTO: &str = "TProjectForm";
pub(crate) const FORMULARIO_GUARDAR: &str = "TSaveObjPVForm";
