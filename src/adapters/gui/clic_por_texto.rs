//! `gui_actuar accion:clic_texto`: clicks the text a person reads on screen.
//! VCL speed buttons, grid rows and tree nodes have no window handle, so they
//! cannot be addressed any other way; the click is posted as mouse messages to
//! the window under the text, which works on a private desktop too.

use super::ffi::{self, FalloClic};
use super::ocr::captura_ocr::lineas_vivas;
use super::ocr::segmentacion::segmentar_lineas;
use super::ocr::ubicacion::{elegir_caja, ubicar_texto};
use super::ventanas::{class_of, hwnd_de};
use crate::args::{opt_u32, req_str};
use crate::error::{Error, Result};
use serde_json::{json, Value};

/// Clicks the on-screen text `texto` inside window `ventana`; `indice` picks
/// among several matches.
pub(crate) fn clic_texto(args: &Value) -> Result<Value> {
    let texto = req_str(args, &["texto"], "texto visible sobre el que pulsar")?;
    let indice = opt_u32(args, "indice")?.map(|i| i as usize);
    let h = hwnd_de(args, "ventana")?;
    let lineas = lineas_vivas(h)?;
    let cajas = ubicar_texto(&segmentar_lineas(&lineas, &[]), texto);
    let caja = elegir_caja(&cajas, indice, texto)?;
    let punto = ((caja.left + caja.right) / 2, (caja.top + caja.bottom) / 2);
    let objetivo = ffi::control_en_punto(h, punto);
    ffi::clic_en_pantalla(objetivo, punto).map_err(|fallo| match fallo {
        FalloClic::SinConversion => Error::Gui(format!(
            "Windows no pudo convertir el punto de pantalla al control 0x{objetivo:X}"
        )),
        FalloClic::SinEnvio => {
            Error::Gui(format!("PostMessage falló para el control 0x{objetivo:X}"))
        }
    })?;
    Ok(json!({
        "accion": "clic_texto",
        "texto": texto,
        "coincidencias": cajas.len(),
        "punto": { "x": punto.0, "y": punto.1 },
        "control": { "hwnd": format!("0x{objetivo:X}"), "clase": class_of(objetivo) },
    }))
}
