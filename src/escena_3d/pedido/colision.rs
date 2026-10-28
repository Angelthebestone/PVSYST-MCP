//! Boxes that overlap. PVsyst's anchor for a position is not proven, so the
//! check assumes the position is the lower corner and says so.

use super::lectura::ObjetoPedido;

/// Overlap smaller than this is touching, not colliding (meters).
const TOLERANCIA_M: f64 = 1e-6;

type Caja = [(f64, f64); 3];

fn caja(objeto: &ObjetoPedido) -> Caja {
    let p = objeto.posicion;
    let e = (objeto.forma.extension)(&objeto.valores);
    [(p.x, p.x + e[0]), (p.y, p.y + e[1]), (p.z, p.z + e[2])]
}

fn se_solapan(a: &Caja, b: &Caja) -> bool {
    a.iter()
        .zip(b)
        .all(|(ea, eb)| ea.0 < eb.1 - TOLERANCIA_M && eb.0 < ea.1 - TOLERANCIA_M)
}

/// Pairs of object indices whose boxes share volume. O(n²) in the objects of a scene.
pub(crate) fn colisiones(objetos: &[ObjetoPedido]) -> Vec<(usize, usize)> {
    let cajas: Vec<Caja> = objetos.iter().map(caja).collect();
    cajas
        .iter()
        .enumerate()
        .flat_map(|(i, a)| {
            cajas
                .iter()
                .enumerate()
                .skip(i + 1)
                .filter(move |(_, b)| se_solapan(a, b))
                .map(move |(j, _)| (i, j))
        })
        .collect()
}
