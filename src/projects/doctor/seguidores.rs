//! Tracker geometry checks for `proyecto doctor`.

use crate::format::object;
use crate::projects::doctor::Informe;

fn numero(nodo: &object::PvNode, clave: &str) -> Option<f64> {
    nodo.find_deep(clave)?.trim().parse().ok()
}

fn es_seguidor(field_type: &str) -> bool {
    field_type.starts_with("Track") || field_type.eq_ignore_ascii_case("UnlimTracking")
}

/// Errors for impossible tracker geometry and a warning when backtracking is
/// enabled without the pitch/width that it needs.
pub(crate) fn revisar_seguidor(var_txt: &str, inf: &mut Informe) {
    let Some(raiz) = object::parse_first(var_txt) else {
        return;
    };
    let Some(field_type) = raiz.find_deep("FieldType") else {
        return;
    };
    if !es_seguidor(field_type) {
        return;
    }
    let pitch = numero(&raiz, "Pitch");
    let ancho = numero(&raiz, "CollectorWidth");
    let (min_phi, max_phi) = (numero(&raiz, "MinPhi"), numero(&raiz, "MaxPhi"));
    let backtracking = raiz
        .find_deep("UsesBacktracking")
        .is_some_and(|v| v.eq_ignore_ascii_case("True"));

    if let (Some(pitch), Some(ancho)) = (pitch, ancho) {
        if pitch <= ancho {
            inf.errores.push(format!(
                "Seguidor: Pitch {pitch} debe superar CollectorWidth {ancho}"
            ));
        }
    }
    if let (Some(min), Some(max)) = (min_phi, max_phi) {
        if min >= max {
            inf.errores.push(format!(
                "Seguidor: MinPhi {min} debe ser menor que MaxPhi {max}"
            ));
        }
    }
    if backtracking && (pitch.is_none() || ancho.is_none()) {
        inf.avisos
            .push("Seguidor con backtracking sin Pitch o CollectorWidth".into());
    }
}

#[cfg(test)]
#[path = "../../../tests/unit/projects/doctor/seguidores.rs"]
mod tests;
