//! Binary decoding primitives for Delphi Real48 geometry and position blocks in 3D scene files.

use super::dominio::{GeometriaElemental, Posicion3d, TipoFormaElemental};
use crate::format::real48::{real48, BYTES_POR_SLOT};

pub(crate) const LONGITUD_BLOQUE_POSICION: usize = 44;
pub(crate) const LONGITUD_BLOQUE_GEOMETRIA_ELEMENTAL: usize = 39;
pub(crate) const TERMINADOR_SLASH: u8 = b'/';

const OFFSET_POS_X: usize = 12;
const OFFSET_POS_Y: usize = 18;
const OFFSET_POS_Z: usize = 24;
const OFFSET_ROT_AZIMUT: usize = 30;
const OFFSET_ROT_INCLINACION: usize = 36;
const OFFSET_FORMA_GEOM: usize = 0;
const OFFSET_DIMS_GEOM: usize = 2;
const MAX_DIMENSIONES_FORMA: usize = 6;

pub(crate) fn parsear_bloque_posicion(raw: &[u8]) -> Option<Posicion3d> {
    if raw.len() < LONGITUD_BLOQUE_POSICION {
        return None;
    }
    let x = leer_slot_real48(raw, OFFSET_POS_X)?;
    let y = leer_slot_real48(raw, OFFSET_POS_Y)?;
    let z = leer_slot_real48(raw, OFFSET_POS_Z)?;
    let rot1 = leer_slot_real48(raw, OFFSET_ROT_AZIMUT)?;
    let rot2 = leer_slot_real48(raw, OFFSET_ROT_INCLINACION)?;
    Some(Posicion3d {
        x,
        y,
        z,
        rotacion_azimut: rot1,
        rotacion_inclinacion: rot2,
    })
}

pub(crate) fn parsear_bloque_geometria(raw: &[u8]) -> Option<GeometriaElemental> {
    if raw.len() < LONGITUD_BLOQUE_GEOMETRIA_ELEMENTAL {
        return None;
    }
    let forma_bytes = raw.get(OFFSET_FORMA_GEOM..OFFSET_FORMA_GEOM + 2)?;
    let forma_codigo = u16::from_le_bytes([*forma_bytes.first()?, *forma_bytes.get(1)?]);
    let mut dims = Vec::with_capacity(MAX_DIMENSIONES_FORMA);
    let mut off = OFFSET_DIMS_GEOM;
    while off + BYTES_POR_SLOT < raw.len() {
        if let Some(val) = leer_slot_real48(raw, off) {
            dims.push(val);
        }
        off += BYTES_POR_SLOT;
    }
    Some(GeometriaElemental {
        forma: TipoFormaElemental::de_codigo(forma_codigo),
        dimensiones: dims,
    })
}

pub(crate) fn leer_slot_real48(raw: &[u8], offset: usize) -> Option<f64> {
    if offset + BYTES_POR_SLOT > raw.len() {
        return None;
    }
    let slice = raw.get(offset..offset + BYTES_POR_SLOT)?;
    let mut slot = [0u8; BYTES_POR_SLOT];
    slot.copy_from_slice(slice);
    Some(real48(&slot))
}
