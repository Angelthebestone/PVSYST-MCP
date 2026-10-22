//! Legacy (< 6.40) hybrid PAN module structure and decoding.

use crate::error::{Error, Result};
use crate::format::real48::{real48, BYTES_POR_SLOT};
use crate::format::redondear;
use serde_json::{json, Map, Value};

/// Minimum number of Real48 slots needed by `decode_legacy_slots`.
/// Highest slot accessed is index 24 (rp_exp), requiring at least 25 slots (0..=24).
pub(crate) const SLOTS_LEGACY_REQUERIDOS: usize = 25;

/// The 3rd CRLF marks the start of the binary Real48 data block.
const TERCER_CRLF: usize = 3;

/// Carriage-return byte (CR).
const BYTE_CR: u8 = 0x0D;

/// Slash character ('/') terminating the legacy Real48 block.
pub(crate) const BYTE_TERMINADOR_BLOQUE: u8 = 0x2F;

/// Locate the block terminator '/' which is either followed by CR/LF or at EOF.
pub(crate) fn find_block_terminator(bytes: &[u8], start: usize) -> Option<usize> {
    for (i, &b) in bytes[start..].iter().enumerate() {
        if b != BYTE_TERMINADOR_BLOQUE {
            continue;
        }
        let pos = start + i;
        let es_fin_o_crlf = match bytes.get(pos + 1) {
            None => true,
            Some(&sig) => sig == BYTE_CR || sig == 0x0A,
        };
        if es_fin_o_crlf {
            return Some(pos);
        }
    }
    None
}

/// Decodes required Real48 slots into named module parameters.
pub(crate) fn decode_legacy_slots(slots: &[f64]) -> Result<Map<String, Value>> {
    if slots.len() < SLOTS_LEGACY_REQUERIDOS {
        return Err(Error::Format(format!(
            "PAN legacy incompleto: {} slots, se esperaban {}",
            slots.len(),
            SLOTS_LEGACY_REQUERIDOS
        )));
    }
    let mut m = Map::new();
    m.insert("pnom_w".into(), json!(redondear(slots[0], 4)));
    m.insert("vmax_iec_v".into(), json!(redondear(slots[1], 4)));
    m.insert("tolerance".into(), json!(redondear(slots[2], 4)));
    m.insert("area_m2".into(), json!(redondear(slots[3], 4)));
    m.insert("cell_area_cm2".into(), json!(redondear(slots[4], 4)));
    m.insert("gref_wm2".into(), json!(redondear(slots[5], 4)));
    m.insert("tref_c".into(), json!(redondear(slots[6], 4)));
    m.insert("absorb".into(), json!(redondear(slots[7], 4)));
    m.insert("isc_a".into(), json!(redondear(slots[8], 5)));
    m.insert("mu_isc_ma_c".into(), json!(redondear(slots[9], 5)));
    m.insert("voc_v".into(), json!(redondear(slots[10], 4)));
    m.insert("mu_voc_mv_c".into(), json!(redondear(slots[11], 4)));
    m.insert("impp_a".into(), json!(redondear(slots[12], 5)));
    m.insert("vmpp_v".into(), json!(redondear(slots[13], 4)));
    m.insert("vrev_diode_v".into(), json!(redondear(slots[14], 4)));
    m.insert("rshunt_ohm".into(), json!(redondear(slots[17], 4)));
    m.insert("rserie_ohm".into(), json!(redondear(slots[18], 5)));
    m.insert("rp0_ohm".into(), json!(redondear(slots[23], 4)));
    m.insert("rp_exp".into(), json!(redondear(slots[24], 4)));
    Ok(m)
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct LegacyHeader {
    pub(crate) vendor: String,
    pub(crate) model: String,
    pub(crate) data_source: String,
    pub(crate) version: String,
    pub(crate) technology: String,
    pub(crate) cells_series: i64,
    pub(crate) cells_parallel: i64,
    pub(crate) diodes: i64,
}

fn parse_legacy_int(val: Option<&str>, campo: &str) -> Result<i64> {
    let s =
        val.ok_or_else(|| Error::Format(format!("Campo faltante en encabezado PAN: {campo}")))?;
    let t = s.trim();
    t.parse::<i64>().map_err(|_| {
        Error::Format(format!(
            "Valor no numérico para {campo} en encabezado PAN: {t}"
        ))
    })
}

pub(crate) fn parse_legacy_header(text: &str) -> Result<LegacyHeader> {
    let mut h = LegacyHeader {
        vendor: String::new(),
        model: String::new(),
        data_source: String::new(),
        version: String::new(),
        technology: String::new(),
        cells_series: 0,
        cells_parallel: 0,
        diodes: 0,
    };
    let mut lines = text.lines();

    if let Some(line1) = lines.next() {
        let parts: Vec<&str> = line1.split('\t').collect();
        h.vendor = parts
            .first()
            .map(|s| s.trim_start_matches("pvModule;").trim())
            .unwrap_or("")
            .to_string();
        h.model = parts.get(1).unwrap_or(&"").trim().to_string();
        h.data_source = parts
            .get(2)
            .map(|s| s.trim_end_matches(';').trim())
            .unwrap_or("")
            .to_string();
    }
    if let Some(line2) = lines.next() {
        h.version = line2
            .split(';')
            .next()
            .map(|s| s.trim_start_matches("Version").trim())
            .unwrap_or("")
            .to_string();
    }
    if let Some(line3) = lines.next() {
        let parts: Vec<&str> = line3.split(';').collect();
        h.technology = parts.first().unwrap_or(&"").trim().to_string();
        h.cells_series = parse_legacy_int(parts.get(1).copied(), "cells_series")?;
        h.cells_parallel = parse_legacy_int(parts.get(2).copied(), "cells_parallel")?;
        h.diodes = parse_legacy_int(parts.get(3).copied(), "diodes")?;
    } else {
        return Err(Error::Format(
            "Encabezado PAN incompleto: falta línea de especificación".into(),
        ));
    }
    Ok(h)
}

/// Find the byte offset just past the 3rd CRLF (start of the legacy block).
pub(crate) fn block1_start(bytes: &[u8]) -> Option<usize> {
    // Local accumulator in window scanner: counts CRLF pairs to find binary start.
    let mut cr = 0;
    for (i, &b) in bytes.iter().enumerate() {
        if b != BYTE_CR || bytes.get(i + 1) != Some(&0x0A) {
            continue;
        }
        cr += 1;
        if cr == TERCER_CRLF {
            return Some(i + 2);
        }
    }
    None
}

/// Detect whether the byte buffer belongs to a legacy (<6.40) PAN module.
pub(crate) fn es_pan_legacy(bytes: &[u8]) -> bool {
    let text = String::from_utf8_lossy(bytes);
    if !text.lines().next().unwrap_or("").starts_with("pvModule;") {
        return false;
    }
    let Some(start) = block1_start(bytes) else {
        return false;
    };
    find_block_terminator(bytes, start).is_some()
}

pub(crate) fn parse_legacy(bytes: &[u8], text: &str) -> Result<Value> {
    let start = block1_start(bytes)
        .ok_or_else(|| Error::Format("Legacy PAN: 3rd CRLF not found".into()))?;
    let term = find_block_terminator(bytes, start)
        .ok_or_else(|| Error::Format("Legacy PAN: block terminator not found".into()))?;

    let n_bytes = term - start;
    let n_slots = n_bytes / BYTES_POR_SLOT;
    if n_bytes % BYTES_POR_SLOT != 0 || n_slots < SLOTS_LEGACY_REQUERIDOS {
        return Err(Error::Format(format!(
            "PAN legacy incompleto: {n_bytes} bytes / {n_slots} slots, se esperaban {SLOTS_LEGACY_REQUERIDOS}"
        )));
    }

    let slots: Vec<f64> = bytes[start..term]
        .as_chunks::<BYTES_POR_SLOT>()
        .0
        .iter()
        .map(real48)
        .collect();

    let h = parse_legacy_header(text)?;
    let fields = decode_legacy_slots(&slots)?;

    let mut out = Map::new();
    out.insert("vendor".into(), json!(h.vendor));
    out.insert("model".into(), json!(h.model));
    out.insert("data_source".into(), json!(h.data_source));
    out.insert("technology".into(), json!(h.technology));
    out.insert("version".into(), json!(h.version));
    out.insert("cells_series".into(), json!(h.cells_series));
    out.insert("cells_parallel".into(), json!(h.cells_parallel));
    out.insert("diodes".into(), json!(h.diodes));
    out.insert(
        "pnom".into(),
        fields.get("pnom_w").cloned().unwrap_or(json!(null)),
    );
    out.insert(
        "voc".into(),
        fields.get("voc_v").cloned().unwrap_or(json!(null)),
    );
    out.insert(
        "isc".into(),
        fields.get("isc_a").cloned().unwrap_or(json!(null)),
    );
    out.insert("parameters".into(), Value::Object(fields));
    out.insert(
        "raw_slots".into(),
        Value::Array(slots.iter().map(|v| json!(redondear(*v, 6))).collect()),
    );
    Ok(Value::Object(out))
}
