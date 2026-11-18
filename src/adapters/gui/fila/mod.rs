//! `gui_actuar accion:fila`: selects row `indice` of a list or tree whose rows
//! have no window of their own. By default it only selects: it never presses
//! Enter or OK, and the answer says `abierta:false`. `abrir:true` (alias
//! `confirmar:true`) then presses Enter on the row and reports whether a window
//! opened or closed.
//!
//! The click is a physical one on the declared geometry, and the result is
//! checked without OCR: the highlighted band of a capture must be the requested
//! row, and the click must not have opened or closed anything. A click that
//! PVsyst took as "open this element" is reported, never hidden. The band proves
//! a row is selected, not which element it holds: with `texto`, the row is read
//! by OCR and must hold that text.

mod banda;
mod error;
mod geometria;
mod lectura;
mod registro;
mod verificacion;

use super::acciones::exigir_escritorio;
use super::captura::rgb_de_control;
use super::clic_fisico::{clic_fisico_en_control, PuntoPedido};
use super::efecto::{Efecto, Instantanea};
use super::escritorio;
use super::ffi::{self, Hwnd, GA_ROOT};
use super::modal;
use super::primer_plano;
use super::ventanas::{class_of, hwnd_de, text_of, visible};
use crate::args::{flag, missing, opt_u32};
use crate::error::{Error, Result};
use crate::idioma::{Idioma, Texto};
use error::ErrorFila;
use geometria::Geometria;
use registro::Control;
use serde_json::{json, Value};
use std::time::Duration;
use verificacion::{
    exigir_banda_en, exigir_foco, exigir_sin_activacion, exigir_texto_pedido, ventanas_del_proceso,
};

/// Time PVsyst needs to repaint the selection after the click.
const REPINTADO: Duration = Duration::from_millis(250);
/// Time PVsyst needs to open (or close) the element after Enter.
const ESPERA_APERTURA: Duration = Duration::from_millis(700);
/// Virtual key of Enter: the default button of the form (its OK) answers it.
const VK_RETURN: u8 = 0x0D;

/// What is being selected: the control, its measured geometry and the row.
struct Pedido<'a> {
    h: Hwnd,
    geom: &'a Geometria,
    indice: usize,
}

fn indice_de(args: &Value) -> Result<usize> {
    let indice = opt_u32(args, "indice")?.ok_or_else(|| {
        missing(
            "indice",
            "fila a seleccionar, desde 0, dentro de las filas visibles",
        )
    })?;
    Ok(usize::try_from(indice).unwrap_or(usize::MAX))
}

fn exigir_escritorio_medido() -> Result<()> {
    exigir_escritorio()?;
    if escritorio::es_aislado() {
        return Err(Error::Gui(
            "fila uses a physical click, which is not measured on the isolated desktop: use accion:clic_texto or PVSYST_MCP_ESCRITORIO without isolation".into(),
        ));
    }
    Ok(())
}

/// Geometry declared for this very control: its class and the window hosting it.
fn geometria_del_control(h: Hwnd) -> Result<&'static Geometria> {
    let raiz = ffi::get_ancestor(h, GA_ROOT);
    let (clase, anfitrion, titulo) = (class_of(h), class_of(raiz), text_of(raiz));
    let control = Control {
        clase: &clase,
        anfitrion: &anfitrion,
        titulo: &titulo,
    };
    registro::geometria_de(&control).map_err(ErrorFila::a_error)
}

/// The control the request points at: `ventana` itself, or the only visible
/// child of class `clase` when the caller names the host form. A list or tree
/// has no window per row, so naming its class is how the caller picks it; with
/// no `clase`, the one control of the form whose geometry was measured.
fn control_de(args: &Value) -> Result<Hwnd> {
    let h = hwnd_de(args, "ventana")?;
    let clase = args.get("clase").and_then(Value::as_str);
    if clase.is_some_and(|c| class_of(h) == c)
        || (clase.is_none() && geometria_del_control(h).is_ok())
    {
        return Ok(h);
    }
    let hijos: Vec<(Hwnd, String)> = ffi::enum_children(h)
        .into_iter()
        .filter(|c| visible(*c))
        .map(|c| (c, class_of(c)))
        .collect();
    match clase {
        Some(clase) => unico_de_clase(&hijos, clase),
        None => Ok(unico_medido(&hijos, &class_of(h), &text_of(h))?.unwrap_or(h)),
    }
}

const VARIOS_MEDIDOS: Texto = Texto {
    es: "Hay más de un control medido en esa ventana: indica 'clase' o pasa el hwnd del control como 'ventana'",
    en: "More than one measured control in that window: pass 'clase' or the control's hwnd as 'ventana'",
};

/// The only child whose class and host form have a measured geometry; `None`
/// when there is none, so the caller reports the controls that are measured.
fn unico_medido(hijos: &[(Hwnd, String)], anfitrion: &str, titulo: &str) -> Result<Option<Hwnd>> {
    let mut medidos = hijos.iter().filter(|(_, clase)| {
        registro::geometria_de(&Control {
            clase,
            anfitrion,
            titulo,
        })
        .is_ok()
    });
    match (medidos.next(), medidos.next()) {
        (Some((h, _)), None) => Ok(Some(*h)),
        (None, _) => Ok(None),
        _ => Err(Error::Invalid(
            VARIOS_MEDIDOS.en_idioma(Idioma::de_entorno()).into(),
        )),
    }
}

/// Exactly one visible child of `clase`; several are ambiguous and none names
/// the classes that are there, so the caller can correct the request.
fn unico_de_clase(hijos: &[(Hwnd, String)], clase: &str) -> Result<Hwnd> {
    let mut encontrados = hijos.iter().filter(|(_, c)| c == clase);
    match (encontrados.next(), encontrados.next()) {
        (Some((h, _)), None) => Ok(*h),
        (None, _) => Err(Error::Invalid(format!(
            "No visible child of class '{clase}' in that window. Visible classes: {}",
            clases_visibles(hijos)
        ))),
        (Some(_), Some(_)) => Err(Error::Invalid(format!(
            "More than one '{clase}' in that window: pass the control's hwnd as 'ventana'"
        ))),
    }
}

fn clases_visibles(hijos: &[(Hwnd, String)]) -> String {
    let mut clases: Vec<&str> = hijos.iter().map(|(_, c)| c.as_str()).collect();
    clases.sort_unstable();
    clases.dedup();
    if clases.is_empty() {
        "none".to_string()
    } else {
        clases.join(", ")
    }
}

/// `abrir:true` and `confirmar:true` both press Enter on the selected row.
fn pide_abrir(args: &Value) -> bool {
    flag(args, &["abrir", "confirmar"])
}

/// Presses Enter on the selected row and reports what it did: whether a window
/// opened or closed (`abierta`) and the windows involved. Nothing opening is
/// reported, not hidden: the row stays selected.
fn abrir_fila(p: &Pedido<'_>, idioma: Idioma) -> Result<Value> {
    let antes = Instantanea::de(p.h);
    exigir_foco(p.h)?;
    ffi::pulsar_combinacion(VK_RETURN, &[]);
    std::thread::sleep(ESPERA_APERTURA);
    let efecto = Efecto::entre(&antes, &antes.de_nuevo());
    let cerrado = !ffi::es_ventana(p.h) || !visible(p.h);
    let abierta = !efecto.abiertas.is_empty() || !efecto.cerradas.is_empty() || cerrado;
    let mut respuesta = json!({ "abierta": abierta, "efecto": efecto.a_json() });
    if let (false, Some(mapa)) = (abierta, respuesta.as_object_mut()) {
        let nada = Texto {
            es: "Enter no abrió ni cerró ninguna ventana (abierta:false): la fila {indice} sigue seleccionada; comprueba con gui_observar que:ventanas",
            en: "Enter opened and closed no window (abierta:false): row {indice} stays selected; check with gui_observar que:ventanas",
        };
        let aviso = nada.texto_con(idioma, &[("indice", &p.indice.to_string())]);
        mapa.insert("advertencias".into(), json!([aviso]));
    }
    Ok(respuesta)
}

pub(crate) fn fila(args: &Value, idioma: Idioma) -> Result<Value> {
    exigir_escritorio_medido()?;
    let h = control_de(args)?;
    let indice = indice_de(args)?;
    modal::exigir_habilitada(h, modal::PARA_ACTUAR)?;
    // The click and the captures read screen coordinates: a minimised window has none.
    primer_plano::restaurar_al_frente(h, idioma)?;
    let geom = geometria_del_control(h)?;
    let antes = rgb_de_control(h)?;
    let (x, dy) = geom
        .punto_de_fila(indice, (antes.ancho_px(), antes.alto_px()))
        .map_err(ErrorFila::a_error)?;
    let ventanas_antes = ventanas_del_proceso(h);
    exigir_foco(h)?;
    clic_fisico_en_control(h, PuntoPedido::EnControl(x, dy), idioma)?;
    std::thread::sleep(REPINTADO);
    let pedido = Pedido { h, geom, indice };
    exigir_sin_activacion(&pedido, &ventanas_antes)?;
    exigir_banda_en(&pedido, &antes)?;
    let texto = exigir_texto_pedido(args, &pedido)?;
    let mut resultado = json!({
        "fila": indice,
        "leida": indice,
        "hwnd": format!("0x{h:X}"),
        "clase": geom.clase,
        "dy_px96": dy,
        "x_px96": x,
        "dpi": ffi::dpi_de(h),
        "verificado": "banda_resaltada",
        "texto_verificado": texto.is_some(),
        "texto_fila": texto,
        "abierta": false,
    });
    if let (true, Some(mapa)) = (pide_abrir(args), resultado.as_object_mut()) {
        if let Value::Object(abierta) = abrir_fila(&pedido, idioma)? {
            mapa.extend(abierta);
        }
    }
    Ok(resultado)
}

#[cfg(test)]
#[path = "../../../../tests/unit/adapters/gui/fila/seleccion.rs"]
mod tests;
