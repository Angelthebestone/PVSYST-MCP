//! Captions every PVsyst dialog shares: the buttons that answer, close or save.
//! Spanish measured live on 8.1.8; English, French and German inferred.

use super::Rotulo;

pub(crate) const OK: Rotulo<'static> = Rotulo::nuevo("OK", ("OK", "OK", "OK"));
pub(crate) const ACEPTAR: Rotulo<'static> = Rotulo::nuevo("Aceptar", ("OK", "Accepter", "OK"));
pub(crate) const CONTINUAR: Rotulo<'static> =
    Rotulo::nuevo("Continuar", ("Continue", "Continuer", "Weiter"));
pub(crate) const CERRAR: Rotulo<'static> =
    Rotulo::nuevo("Cerrar", ("Close", "Fermer", "Schließen"));
pub(crate) const CANCELAR: Rotulo<'static> =
    Rotulo::nuevo("Cancelar", ("Cancel", "Annuler", "Abbrechen"));
pub(crate) const SALIR: Rotulo<'static> = Rotulo::nuevo("Salir", ("Exit", "Quitter", "Beenden"));
pub(crate) const SALIDA: Rotulo<'static> = Rotulo::nuevo("Salida", ("Exit", "Sortie", "Beenden"));
pub(crate) const NO: Rotulo<'static> = Rotulo::nuevo("No", ("No", "Non", "Nein"));
pub(crate) const SI: Rotulo<'static> = Rotulo::nuevo("Sí", ("Yes", "Oui", "Ja"));
pub(crate) const GUARDAR: Rotulo<'static> =
    Rotulo::nuevo("Guardar", ("Save", "Enregistrer", "Speichern"));
pub(crate) const GUARDAR_COMO: Rotulo<'static> = Rotulo::nuevo(
    "Guardar como",
    ("Save as", "Enregistrer sous", "Speichern unter"),
);
pub(crate) const NUEVO: Rotulo<'static> = Rotulo::nuevo("Nuevo", ("New", "Nouveau", "Neu"));
/// The button of the crash dialog that starts PVsyst again.
pub(crate) const REINICIAR: Rotulo<'static> = Rotulo::nuevo(
    "Reiniciar PVsyst",
    ("Restart PVsyst", "Redémarrer PVsyst", "PVsyst neu starten"),
);

/// Buttons that abandon a form, in the order they are tried.
pub(crate) const BOTONES_CIERRE: &[Rotulo<'static>] = &[CERRAR, CANCELAR, SALIR, NO];
/// Buttons of a message box that only informs, in the order they are tried.
pub(crate) const BOTONES_MENSAJE: &[Rotulo<'static>] = &[OK, ACEPTAR, CONTINUAR, CERRAR, CANCELAR];
/// Buttons of the madExcept crash dialog that let the session go on.
pub(crate) const BOTONES_EXCEPCION: &[Rotulo<'static>] = &[CONTINUAR];
/// Buttons that leave a form of a session step without saving.
pub(crate) const BOTONES_SALIDA: &[Rotulo<'static>] = &[CANCELAR, CERRAR, SALIDA, NO];
/// Buttons that dismiss a message box of a session step (the answers it takes as consent).
pub(crate) const BOTONES_SESION: &[Rotulo<'static>] = &[OK, SI, ACEPTAR, CONTINUAR, CERRAR];
/// Buttons that abandon a form when the request says `cancelar`.
pub(crate) const BOTONES_CANCELAR: &[Rotulo<'static>] = &[CANCELAR, NO];

/// Words of the "PVsyst is unstable after a crash" warning, folded to lower case
/// and without accents, in each language.
pub(crate) const AVISO_INESTABLE: &[&str] = &["inestable", "unstable", "instable", "instabil"];

pub(super) const TABLA: &[Rotulo<'static>] = &[
    OK,
    ACEPTAR,
    CONTINUAR,
    CERRAR,
    CANCELAR,
    SALIR,
    SALIDA,
    NO,
    SI,
    GUARDAR,
    GUARDAR_COMO,
    NUEVO,
    REINICIAR,
];
