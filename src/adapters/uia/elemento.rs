//! UIA element search, pattern invocation and geometry.

use super::arbol::hijos;
use super::{cadena, error, rol_de, Sesion};
use crate::error::{Error, Result};
use serde_json::{json, Value};
use std::ffi::c_void;
use windows::core::{Interface, BSTR};
use windows::Win32::Foundation::HWND;
use windows::Win32::UI::Accessibility::{
    IUIAutomationElement, IUIAutomationExpandCollapsePattern, IUIAutomationInvokePattern,
    IUIAutomationSelectionItemPattern, IUIAutomationTogglePattern, IUIAutomationValuePattern,
    UIA_ExpandCollapsePatternId, UIA_InvokePatternId, UIA_SelectionItemPatternId,
    UIA_TogglePatternId, UIA_ValuePatternId, UIA_PATTERN_ID,
};

/// Visit cap for name searches, keeping `invocar` bounded on very large trees.
const BUSQUEDA_MAX: u32 = 5000;

/// Matching criteria for locating a UIA element in a window hierarchy.
pub(crate) struct CriterioBusqueda<'a> {
    pub(crate) hwnd: isize,
    pub(crate) nombre: Option<&'a str>,
    pub(crate) rol: Option<&'a str>,
}

fn coincide_rol(el: &IUIAutomationElement, rol: Option<&str>) -> bool {
    match rol {
        Some(r) => {
            // SAFETY: Consulta COM de control type sobre interfaz viva.
            let tipo = unsafe { el.CachedControlType() }.map_or(0, |t| t.0);
            rol_de(tipo).eq_ignore_ascii_case(r)
        }
        None => true,
    }
}

/// Depth-first search for an element matching `nombre` (exact first, then
/// substring) and `rol`. Errors list the nearest names encountered.
fn buscar(sesion: &Sesion, criterio: &CriterioBusqueda) -> Result<IUIAutomationElement> {
    if criterio.nombre.is_none() && criterio.rol.is_none() {
        return Err(Error::Gui(
            "Indique 'nombre' o 'rol' para localizar el elemento UIA".into(),
        ));
    }
    // SAFETY: ElementFromHandleBuildCache sobre HWND válido con COM inicializado.
    let raiz = unsafe {
        sesion
            .auto
            .ElementFromHandleBuildCache(HWND(criterio.hwnd as *mut c_void), &sesion.req)
    }
    .map_err(error)?;
    // SAFETY: ControlViewWalker sobre interfaz viva.
    let walker = unsafe { sesion.auto.ControlViewWalker() }.map_err(error)?;
    let mut pila = vec![raiz];
    let mut parcial: Option<IUIAutomationElement> = None;
    let mut cercanos: Vec<String> = Vec::new();
    let mut visitados = 0u32;
    while let Some(el) = pila.pop() {
        visitados += 1;
        if visitados > BUSQUEDA_MAX {
            break;
        }
        // SAFETY: CachedName sobre interfaz viva; ignoramos errores aquí para continuar búsqueda.
        let texto = cadena(unsafe { el.CachedName() }).unwrap_or_default();
        if !texto.is_empty() && cercanos.len() < 10 && !cercanos.contains(&texto) {
            cercanos.push(texto.clone());
        }
        if coincide_rol(&el, criterio.rol) {
            match criterio.nombre {
                None => return Ok(el),
                Some(n) if texto.eq_ignore_ascii_case(n) => return Ok(el),
                Some(n) => {
                    if parcial.is_none() && texto.to_lowercase().contains(&n.to_lowercase()) {
                        parcial = Some(el.clone());
                    }
                }
            }
        }
        let (ch, _) = hijos(sesion, &walker, &el);
        pila.extend(ch.into_iter().rev());
    }
    parcial.ok_or_else(|| {
        Error::Gui(format!(
            "Elemento UIA '{}' no encontrado; cercanos: {}",
            criterio.nombre.or(criterio.rol).unwrap_or_default(),
            cercanos.join(", ")
        ))
    })
}

/// Returns the typed pattern `T` of `el`, or an error when it is not supported.
fn patron<T: Interface>(el: &IUIAutomationElement, id: UIA_PATTERN_ID) -> Result<T> {
    // SAFETY: Llamada COM sobre una interfaz viva en el hilo con COM inicializado.
    let desconocido = unsafe { el.GetCurrentPattern(id) }.map_err(error)?;
    desconocido.cast::<T>().map_err(error)
}

/// Runs one UIA pattern operation on `el`.
fn operar(el: &IUIAutomationElement, accion: &str, valor: Option<&str>) -> Result<()> {
    match accion {
        "invocar" => {
            let p = patron::<IUIAutomationInvokePattern>(el, UIA_InvokePatternId)?;
            // SAFETY: Llamada COM sobre interfaz viva.
            unsafe { p.Invoke() }.map_err(error)
        }
        "alternar" => {
            let p = patron::<IUIAutomationTogglePattern>(el, UIA_TogglePatternId)?;
            // SAFETY: Llamada COM sobre interfaz viva.
            unsafe { p.Toggle() }.map_err(error)
        }
        "seleccionar" => {
            let p = patron::<IUIAutomationSelectionItemPattern>(el, UIA_SelectionItemPatternId)?;
            // SAFETY: Llamada COM sobre interfaz viva.
            unsafe { p.Select() }.map_err(error)
        }
        "expandir" => {
            let p = patron::<IUIAutomationExpandCollapsePattern>(el, UIA_ExpandCollapsePatternId)?;
            // SAFETY: Llamada COM sobre interfaz viva.
            unsafe { p.Expand() }.map_err(error)
        }
        "contraer" => {
            let p = patron::<IUIAutomationExpandCollapsePattern>(el, UIA_ExpandCollapsePatternId)?;
            // SAFETY: Llamada COM sobre interfaz viva.
            unsafe { p.Collapse() }.map_err(error)
        }
        "fijar_valor" => {
            let v = valor.ok_or_else(|| Error::Gui("Indique 'valor' para fijar_valor".into()))?;
            let p = patron::<IUIAutomationValuePattern>(el, UIA_ValuePatternId)?;
            let bstr = BSTR::from(v);
            // SAFETY: Llamada COM sobre interfaz viva con BSTR válido.
            unsafe { p.SetValue(&bstr) }.map_err(error)
        }
        other => Err(Error::Gui(format!(
            "Operación UIA desconocida '{other}' (invocar|fijar_valor|alternar|seleccionar|expandir|contraer)"
        ))),
    }
}

/// Finds an element under `hwnd` by name and/or role and applies `accion`.
pub(crate) fn invocar(
    hwnd: isize,
    nombre: Option<&str>,
    rol: Option<&str>,
    accion: &str,
    valor: Option<&str>,
) -> Result<Value> {
    let sesion = Sesion::nueva()?;
    let criterio = CriterioBusqueda { hwnd, nombre, rol };
    let el = buscar(&sesion, &criterio)?;
    operar(&el, accion, valor)?;
    // SAFETY: Llamada COM para extraer el nombre del elemento invocado.
    let nombre_str = cadena(unsafe { el.CachedName() }).unwrap_or_default();
    // SAFETY: Llamada COM para extraer el tipo de control del elemento invocado.
    let rol_str = unsafe { el.CachedControlType() }.map_or("otro", |t| rol_de(t.0));
    Ok(json!({
        "operacion": accion,
        "nombre": nombre_str,
        "rol": rol_str,
    }))
}

/// Screen centre of the element found by name and/or role, for physical clicks.
pub(crate) fn centro(hwnd: isize, nombre: Option<&str>, rol: Option<&str>) -> Result<(i32, i32)> {
    let sesion = Sesion::nueva()?;
    let criterio = CriterioBusqueda { hwnd, nombre, rol };
    let el = buscar(&sesion, &criterio)?;
    // SAFETY: Llamada COM sobre una interfaz viva con COM inicializado.
    let r = unsafe { el.CachedBoundingRectangle() }.map_err(error)?;
    if r.right <= r.left || r.bottom <= r.top {
        // SAFETY: Llamada COM sobre una interfaz viva con COM inicializado.
        let nom = cadena(unsafe { el.CachedName() }).unwrap_or_else(|_| "desconocido".into());
        return Err(Error::Gui(format!(
            "Elemento '{nom}' sin rectángulo visible"
        )));
    }
    Ok(((r.left + r.right) / 2, (r.top + r.bottom) / 2))
}
