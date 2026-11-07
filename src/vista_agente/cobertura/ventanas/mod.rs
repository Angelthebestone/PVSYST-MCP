//! The registry: every window class of the map, grouped by feature.

mod aplicacion;
mod componentes;
mod escena_3d;
mod meteo;
mod resultados;
mod variante;

use super::dominio::Ventana;

const GRUPOS: &[&[Ventana]] = &[
    componentes::VENTANAS,
    variante::VENTANAS,
    escena_3d::VENTANAS,
    meteo::VENTANAS,
    resultados::VENTANAS,
    aplicacion::VENTANAS,
];

/// Every registered window class.
pub(crate) fn todas() -> impl Iterator<Item = &'static Ventana> {
    GRUPOS.iter().flat_map(|g| g.iter())
}

/// The registration of `clase`, if any. O(n) over 85 entries.
pub(crate) fn de_clase(clase: &str) -> Option<&'static Ventana> {
    todas().find(|v| v.clase == clase)
}
