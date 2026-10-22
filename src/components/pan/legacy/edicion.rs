//! Structured field editing for legacy (< 6.40) Delphi Real48 PAN modules.
//!
//! Patches only the 6-byte Real48 slot corresponding to proven numeric fields,
//! keeping every other byte in the file byte-for-byte identical.
use super::dominio::{block1_start, find_block_terminator};
use crate::error::{Error, Result};
use crate::format::edit::ChangeOutcome;
use crate::format::real48::{encode_real48, real48, BYTES_POR_SLOT};

/// Proven numeric field slot index and canonical identifier in legacy PAN block 1.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct SlotDef {
    pub(crate) index: usize,
    pub(crate) canonical: &'static str,
}

const fn slot(index: usize, canonical: &'static str) -> SlotDef {
    SlotDef { index, canonical }
}

/// Known and verified field slots in legacy PAN block 1.
///
/// Verified against CASSYS binary PAN reader and PVsyst legacy module files.
/// Fields outside this verified mapping are rejected.
pub(crate) fn slot_de_campo(nombre: &str) -> Option<SlotDef> {
    let lower = nombre.to_ascii_lowercase();
    let norm = lower.trim().replace('-', "_");
    match norm.as_str() {
        "pnom" | "pnom_w" | "p_nom" => Some(slot(0, "pnom_w")),
        "vmax_iec" | "vmax_iec_v" | "vmaxiec" => Some(slot(1, "vmax_iec_v")),
        "tolerance" => Some(slot(2, "tolerance")),
        "area" | "area_m" | "area_m2" | "aream" => Some(slot(3, "area_m2")),
        "cell_area" | "cell_area_cm2" | "cellarea" => Some(slot(4, "cell_area_cm2")),
        "gref" | "gref_wm2" => Some(slot(5, "gref_wm2")),
        "tref" | "tref_c" => Some(slot(6, "tref_c")),
        "absorb" => Some(slot(7, "absorb")),
        "isc" | "isc_a" => Some(slot(8, "isc_a")),
        "mu_isc" | "mu_isc_ma_c" | "muisc" => Some(slot(9, "mu_isc_ma_c")),
        "voc" | "voc_v" => Some(slot(10, "voc_v")),
        "mu_voc" | "mu_voc_mv_c" | "muvoc" | "muvocspec" => Some(slot(11, "mu_voc_mv_c")),
        "imp" | "impp" | "imp_a" | "impp_a" => Some(slot(12, "impp_a")),
        "vmp" | "vmpp" | "vmp_v" | "vmpp_v" => Some(slot(13, "vmpp_v")),
        "vrev_diode" | "vrev_diode_v" | "vrevdiode" => Some(slot(14, "vrev_diode_v")),
        "rshunt" | "rshunt_ohm" => Some(slot(17, "rshunt_ohm")),
        "rserie" | "rserie_ohm" => Some(slot(18, "rserie_ohm")),
        "rp0" | "rp0_ohm" | "rp_0" => Some(slot(23, "rp0_ohm")),
        "rp_exp" | "rpexp" => Some(slot(24, "rp_exp")),
        _ => None,
    }
}

/// Parse a numeric string or reject if absent or non-numeric.
fn parse_float_val(key: &str, opt_val: Option<&str>) -> Result<f64> {
    let val_str = opt_val.ok_or_else(|| {
        Error::Invalid(format!(
            "PAN legacy no permite eliminar campos (recibido null para '{key}')"
        ))
    })?;
    val_str.trim().parse::<f64>().map_err(|_| {
        Error::Invalid(format!(
            "Valor no numérico '{val_str}' para campo '{key}' en PAN legacy"
        ))
    })
}

/// Patch one 6-byte slot in the legacy buffer.
fn parchear_un_slot(
    bytes: &mut [u8],
    start: usize,
    n_slots: usize,
    def: SlotDef,
    nuevo_valor: f64,
) -> Result<ChangeOutcome> {
    if def.index >= n_slots {
        return Err(Error::Invalid(format!(
            "Slot {} para '{}' fuera de rango (el archivo tiene {} slots)",
            def.index, def.canonical, n_slots
        )));
    }
    let offset = start + def.index * BYTES_POR_SLOT;
    let mut old_slot_bytes = [0u8; BYTES_POR_SLOT];
    old_slot_bytes.copy_from_slice(&bytes[offset..offset + BYTES_POR_SLOT]);
    let old_f = real48(&old_slot_bytes);

    let new_slot_bytes = encode_real48(nuevo_valor)?;
    bytes[offset..offset + BYTES_POR_SLOT].copy_from_slice(&new_slot_bytes);

    Ok(ChangeOutcome {
        key: def.canonical.to_string(),
        action: "modificado".into(),
        old_value: Some(old_f.to_string()),
        new_value: Some(nuevo_valor.to_string()),
    })
}

/// Patch proven numeric fields in a legacy PAN byte buffer in place.
///
/// Guaranteed to be strictly byte-preserving for every byte outside the edited slots.
pub(crate) fn parchear_campos_legacy(
    bytes: &[u8],
    cambios: &[(String, Option<String>)],
) -> Result<(Vec<u8>, Vec<ChangeOutcome>)> {
    let start = block1_start(bytes)
        .ok_or_else(|| Error::Format("Legacy PAN: 3rd CRLF not found".into()))?;
    let term = find_block_terminator(bytes, start)
        .ok_or_else(|| Error::Format("Legacy PAN: block terminator not found".into()))?;

    let n_bytes = term - start;
    let n_slots = n_bytes / BYTES_POR_SLOT;

    // Local copy to ensure all-or-nothing transactional edits.
    let mut nuevo = bytes.to_vec();
    let mut outcomes = Vec::with_capacity(cambios.len());

    for (key, opt_val) in cambios {
        let def = slot_de_campo(key).ok_or_else(|| {
            Error::Invalid(format!(
                "Campo no comprobado o desconocido en PAN legacy: '{key}'"
            ))
        })?;
        let valor = parse_float_val(key, opt_val.as_deref())?;
        let outcome = parchear_un_slot(&mut nuevo, start, n_slots, def, valor)?;
        outcomes.push(outcome);
    }

    Ok((nuevo, outcomes))
}
