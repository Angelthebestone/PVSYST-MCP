//! `gui_actuar accion:clic_texto`: clicks the text a person reads on screen.
//! VCL speed buttons, grid rows and tree nodes have no window handle, so they
//! cannot be addressed any other way; the click is posted as mouse messages to
//! the window under the text, which works on a private desktop too.

use super::ffi::{self, FalloClic, Hwnd};
use super::modal;
use super::ocr::captura_ocr::lineas_vivas;
use super::ocr::segmentacion::segmentar_lineas;
use super::ocr::ubicacion::{elegir_caja, ubicar_texto};
use super::ventanas::{class_of, hwnd_de};
use crate::args::{opt_u32, req_str};
use crate::error::{Error, Result};
use serde_json::{json, Value};
use std::time::Duration;

/// Lets the form process a posted click before the tab is read back.
const ASENTAMIENTO_CLIC: Duration = Duration::from_millis(400);

/// Clicks the on-screen text `texto` inside window `ventana`; `indice` picks
/// among several matches.
pub(crate) fn clic_texto(args: &Value) -> Result<Value> {
    let texto = req_str(args, &["texto"], "texto visible sobre el que pulsar")?;
    let indice = opt_u32(args, "indice")?.map(|i| i as usize);
    let h = hwnd_de(args, "ventana")?;
    modal::exigir_habilitada(h, modal::PARA_ACTUAR)?;
    let lineas = lineas_vivas(h)?;
    let cajas = ubicar_texto(&segmentar_lineas(&lineas, &[]), texto);
    let caja = elegir_caja(&cajas, indice, texto)?;
    let punto = ((caja.left + caja.right) / 2, (caja.top + caja.bottom) / 2);
    let objetivo = ffi::control_en_punto(h, punto);
    let pestana_antes = pestana_seleccionada(objetivo);
    ffi::clic_en_pantalla(objetivo, punto).map_err(|fallo| match fallo {
        FalloClic::SinConversion => Error::Gui(format!(
            "Windows no pudo convertir el punto de pantalla al control 0x{objetivo:X}"
        )),
        FalloClic::SinEnvio => {
            Error::Gui(format!("PostMessage falló para el control 0x{objetivo:X}"))
        }
    })?;
    let mut resultado = json!({
        "accion": "clic_texto",
        "texto": texto,
        "coincidencias": cajas.len(),
        "punto": { "x": punto.0, "y": punto.1 },
        "control": { "hwnd": format!("0x{objetivo:X}"), "clase": class_of(objetivo) },
    });
    let aviso = pestana_sin_cambio(objetivo, pestana_antes);
    if let (Some(aviso), Some(obj)) = (aviso, resultado.as_object_mut()) {
        obj.insert("advertencias".into(), json!([aviso]));
    }
    Ok(resultado)
}

/// Tab selected in `control` when it is a page control, which a click may switch.
fn pestana_seleccionada(control: Hwnd) -> Option<usize> {
    if class_of(control) != ffi::CLASE_PESTANAS {
        return None;
    }
    ffi::pagina_seleccionada(control)
}

/// Warns when a click on a page control left the selected tab where it was:
/// PVsyst vetoes the change while the current page has invalid fields.
fn pestana_sin_cambio(control: Hwnd, antes: Option<usize>) -> Option<String> {
    let antes = antes?;
    std::thread::sleep(ASENTAMIENTO_CLIC);
    (ffi::pagina_seleccionada(control)? == antes).then(|| {
        format!(
            "La pestaña seleccionada no cambió (sigue en la {antes}): PVsyst la vetó por validación del formulario o ya estaba seleccionada; el motivo está en gui_observar que:vista ocr:true (textos)"
        )
    })
}
