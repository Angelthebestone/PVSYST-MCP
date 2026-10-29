//! One object of the scene: PVsyst's elementary-object dialog, filled field by
//! field and closed with «Cerrar objeto», which adds it to the scene.

use super::pasos::{abrir, campos, escribir, hijo_n, ventana};
use crate::adapters::gui::conduccion::{esperar_cierre, Formulario};
use crate::adapters::gui::ffi;
use crate::error::{Error, Result};
use crate::escena_3d::ObjetoPedido;
use std::time::Duration;

const DIALOGO: &str = "TElementaryObjectForm";
/// The dialog rebuilds its fields when the shape changes.
const RECONSTRUCCION: Duration = Duration::from_millis(900);
const ESPERA_CIERRE: Duration = Duration::from_secs(10);

/// Creates `objeto` at the origin of the scene; its position is set afterwards.
pub(super) fn crear(objeto: &ObjetoPedido) -> Result<()> {
    abrir("escena_3d.crear.objeto_elemental")?;
    let dialogo = Formulario::de(ventana(DIALOGO)?);
    elegir_forma(&dialogo, objeto)?;
    rellenar(&dialogo, objeto)?;
    dialogo.pulsar("Cerrar objeto")?;
    if esperar_cierre(dialogo.h, ESPERA_CIERRE) {
        return Ok(());
    }
    Err(Error::Gui(format!(
        "PVsyst no cerró '{DIALOGO}' tras «Cerrar objeto»: ¿hay un valor inválido?"
    )))
}

fn elegir_forma(dialogo: &Formulario, objeto: &ObjetoPedido) -> Result<()> {
    let lista = hijo_n(dialogo.h, "TComboBox", 0)?;
    if !ffi::elegir_en_combo(lista, objeto.forma.nombre_pvsyst)? {
        return Err(Error::Gui(format!(
            "PVsyst no ofrece la forma '{}'",
            objeto.forma.nombre_pvsyst
        )));
    }
    std::thread::sleep(RECONSTRUCCION);
    Ok(())
}

/// The name field is the first edit, the shape's values follow in `Params` order.
/// Thin shapes (cable) add a last edit, the «Objeto delgado» percentage, which is left as is.
fn rellenar(dialogo: &Formulario, objeto: &ObjetoPedido) -> Result<()> {
    let campos = campos(dialogo.h);
    let esperados = objeto.forma.parametros.len() + 1;
    if !(esperados..=esperados + 1).contains(&campos.len()) {
        return Err(Error::Gui(format!(
            "'{}' tiene {} campos y se esperaban {esperados}: ¿cambió la versión de PVsyst?",
            objeto.forma.tipo,
            campos.len()
        )));
    }
    if let (Some(nombre), Some(campo)) = (&objeto.nombre, campos.first()) {
        escribir(dialogo.h, *campo, nombre)?;
    }
    for (campo, p) in campos.iter().skip(1).zip(objeto.forma.parametros) {
        let valor = objeto.valores.get(p.clave).copied().unwrap_or(p.defecto);
        escribir(dialogo.h, *campo, &valor.to_string())?;
    }
    Ok(())
}
