//! Boxes that overlap. PVsyst's anchor for a position is not proven, so the
//! check assumes the position is the lower corner and says so.

use super::lectura::ObjetoPedido;

/// Overlap smaller than this is touching, not colliding (meters).
const TOLERANCIA_M: f64 = 1e-6;

type Caja = [(f64, f64); 3];

/// The box of an object whose geometry is proven; `None` for the other shapes.
fn caja(objeto: &ObjetoPedido) -> Option<Caja> {
    let p = objeto.posicion;
    let [dx, dy, dz] = (objeto.forma.extension?)(&objeto.valores);
    Some([(p.x, p.x + dx), (p.y, p.y + dy), (p.z, p.z + dz)])
}

fn se_solapan(a: &Caja, b: &Caja) -> bool {
    a.iter()
        .zip(b)
        .all(|(ea, eb)| ea.0 < eb.1 - TOLERANCIA_M && eb.0 < ea.1 - TOLERANCIA_M)
}

/// Pairs of object indices whose boxes share volume. O(n²) in the objects of a scene.
/// Shapes without a proven box are left out: see [`sin_caja`].
pub(crate) fn colisiones(objetos: &[ObjetoPedido]) -> Vec<(usize, usize)> {
    let cajas: Vec<Option<Caja>> = objetos.iter().map(caja).collect();
    cajas
        .iter()
        .enumerate()
        .flat_map(|(i, a)| {
            cajas
                .iter()
                .enumerate()
                .skip(i + 1)
                .filter(move |(_, b)| matches!((a, b), (Some(a), Some(b)) if se_solapan(a, b)))
                .map(move |(j, _)| (i, j))
        })
        .collect()
}

/// Indices of the objects the collision check cannot see (shape without a proven box).
pub(crate) fn sin_caja(objetos: &[ObjetoPedido]) -> Vec<usize> {
    let vacias = |o: &ObjetoPedido| o.forma.extension.is_none();
    objetos
        .iter()
        .enumerate()
        .filter(|(_, o)| vacias(o))
        .map(|(i, _)| i)
        .collect()
}
