//! UIA element search, pattern invocation and geometry.

use super::com;
use super::{rol_de, Sesion};
use crate::error::{Error, Result};
use serde_json::{json, Value};
use windows::Win32::UI::Accessibility::IUIAutomationElement;

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
        Some(r) => rol_de(com::tipo_control(el).unwrap_or(0)).eq_ignore_ascii_case(r),
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
    let (raiz, walker) = com::raiz_y_walker(sesion, criterio.hwnd)?;
    let mut pila = vec![raiz];
    let mut parcial: Option<IUIAutomationElement> = None;
    let mut cercanos: Vec<String> = Vec::new();
    let mut visitados = 0u32;
    while let Some(el) = pila.pop() {
        visitados += 1;
        if visitados > BUSQUEDA_MAX {
            break;
        }
        let texto = com::nombre(&el).unwrap_or_default();
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
        let (ch, _) = com::hijos(sesion, &walker, &el);
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
    com::operar(&el, accion, valor)?;
    let nombre_str = com::nombre(&el).unwrap_or_default();
    let rol_str = com::tipo_control(&el).map_or("otro", rol_de);
    Ok(json!({
        "operacion": accion,
        "nombre": nombre_str,
        "rol": rol_str,
    }))
}

/// Index of the tab item `nombre` among its tab-strip siblings, with the strip's
/// window handle. `None` when the item has no parent strip.
pub(crate) fn posicion_pestana(hwnd: isize, nombre: &str) -> Result<Option<(usize, isize)>> {
    let sesion = Sesion::nueva()?;
    let criterio = CriterioBusqueda {
        hwnd,
        nombre: Some(nombre),
        rol: Some("tab item"),
    };
    let el = buscar(&sesion, &criterio)?;
    let (_, walker) = com::raiz_y_walker(&sesion, hwnd)?;
    let Some(padre) = com::padre(&walker, &el) else {
        return Ok(None);
    };
    let (hermanas, _) = com::hijos(&sesion, &walker, &padre);
    let pestanas: Vec<String> = hermanas
        .iter()
        .filter(|h| com::tipo_control(h).is_ok_and(|t| rol_de(t) == "tab item"))
        .map(|h| com::nombre(h).unwrap_or_default())
        .collect();
    let propio = com::nombre(&el).unwrap_or_default();
    Ok(pestanas
        .iter()
        .position(|p| *p == propio)
        .map(|i| (i, com::hwnd_nativo(&padre))))
}

/// Screen centre of the element found by name and/or role, for physical clicks.
pub(crate) fn centro(hwnd: isize, nombre: Option<&str>, rol: Option<&str>) -> Result<(i32, i32)> {
    let sesion = Sesion::nueva()?;
    let criterio = CriterioBusqueda { hwnd, nombre, rol };
    let el = buscar(&sesion, &criterio)?;
    let r = com::rectangulo(&el).map_err(super::error)?;
    if r.right <= r.left || r.bottom <= r.top {
        let nom = com::nombre(&el).unwrap_or_else(|_| "desconocido".into());
        return Err(Error::Gui(format!(
            "Elemento '{nom}' sin rectángulo visible"
        )));
    }
    Ok(((r.left + r.right) / 2, (r.top + r.bottom) / 2))
}
