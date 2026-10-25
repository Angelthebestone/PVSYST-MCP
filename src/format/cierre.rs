//! Closing-marker recognition for the PVsyst text format: which line closes
//! which object type, including PVsyst's known closing-type aliases.

/// Known PVsyst type aliases where closing tag name differs from opening type.
const EQUIVALENCIAS_CIERRE: &[(&str, &str)] = &[
    ("pvageing", "tageing"),
    ("pvsimulfile", "tsimulfich"),
    ("pvorient", "torientgroup"),
    ("pvdomeorient", "tdomeorientgroup"),
];

pub(super) fn coincide_tipo_cierre(esperado: &str, encontrado: &str) -> bool {
    let esp = esperado.to_ascii_lowercase();
    let enc = encontrado.to_ascii_lowercase();
    esp == enc
        || EQUIVALENCIAS_CIERRE
            .iter()
            .any(|&(e, c)| e == esp && c == enc)
}

/// The type named by a closing marker; `Some(None)` for a bare `End of PVObject`.
pub(super) fn extraer_tipo_cierre(linea: &str) -> Option<Option<&str>> {
    let trimmed = linea.trim_end();
    if let Some(rest) = trimmed.strip_prefix("End of PVObject") {
        let r = rest.trim();
        Some(if r.is_empty() { None } else { Some(r) })
    } else {
        match trimmed.to_ascii_lowercase().as_str() {
            "end of tageing" => Some(Some("TAgeing")),
            "end of tsimulfich" => Some(Some("TSimulFich")),
            "end of torientgroup" => Some(Some("TOrientGroup")),
            "end of tdomeorientgroup" => Some(Some("TDomeOrientGroup")),
            _ => None,
        }
    }
}

/// True when `trimmed` closes an object of type `ty`.
pub(crate) fn cierra_objeto(trimmed: &str, ty: &str) -> bool {
    extraer_tipo_cierre(trimmed)
        .is_some_and(|opt| opt.is_none_or(|enc| coincide_tipo_cierre(ty, enc)))
}
