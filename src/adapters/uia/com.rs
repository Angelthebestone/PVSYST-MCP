//! Safe wrappers for the UI Automation COM methods used by this adapter.
//!
//! Every wrapper documents the lifetime and apartment assumptions once, so the
//! tree and element modules stay free of `unsafe`.

use super::{error, Sesion};
use crate::error::{Error, Result};
use std::ffi::c_void;
use windows::core::{Interface, BSTR};
use windows::Win32::Foundation::{HWND, RECT};
use windows::Win32::UI::Accessibility::{
    IUIAutomation, IUIAutomationCacheRequest, IUIAutomationElement,
    IUIAutomationExpandCollapsePattern, IUIAutomationInvokePattern,
    IUIAutomationSelectionItemPattern, IUIAutomationTogglePattern, IUIAutomationTreeWalker,
    IUIAutomationValuePattern, UIA_AutomationIdPropertyId, UIA_BoundingRectanglePropertyId,
    UIA_ClassNamePropertyId, UIA_ControlTypePropertyId, UIA_ExpandCollapsePatternId,
    UIA_InvokePatternId, UIA_IsEnabledPropertyId, UIA_LocalizedControlTypePropertyId,
    UIA_NamePropertyId, UIA_NativeWindowHandlePropertyId, UIA_SelectionItemPatternId,
    UIA_TogglePatternId, UIA_ValuePatternId,
};

/// Cached properties of one element, each carrying its own COM error.
pub(crate) struct PropiedadesCom {
    pub(crate) nombre: windows::core::Result<String>,
    pub(crate) tipo: windows::core::Result<i32>,
    pub(crate) automation_id: windows::core::Result<String>,
    pub(crate) clase: windows::core::Result<String>,
    pub(crate) rect: windows::core::Result<RECT>,
    pub(crate) hwnd: windows::core::Result<isize>,
}

/// Builds the cache request used for every element read.
pub(crate) fn configurar_cache(auto: &IUIAutomation) -> Result<IUIAutomationCacheRequest> {
    // SAFETY: Llamadas COM sobre una interfaz viva en el hilo con COM inicializado.
    unsafe {
        let req = auto.CreateCacheRequest().map_err(error)?;
        for propiedad in [
            UIA_NamePropertyId,
            UIA_ControlTypePropertyId,
            UIA_LocalizedControlTypePropertyId,
            UIA_AutomationIdPropertyId,
            UIA_ClassNamePropertyId,
            UIA_NativeWindowHandlePropertyId,
            UIA_BoundingRectanglePropertyId,
            UIA_IsEnabledPropertyId,
        ] {
            req.AddProperty(propiedad).map_err(error)?;
        }
        for patron in [
            UIA_InvokePatternId,
            UIA_TogglePatternId,
            UIA_ValuePatternId,
            UIA_SelectionItemPatternId,
            UIA_ExpandCollapsePatternId,
        ] {
            req.AddPattern(patron).map_err(error)?;
        }
        Ok(req)
    }
}

/// Returns the cached root element for `hwnd` and the control-view walker.
pub(crate) fn raiz_y_walker(
    sesion: &Sesion,
    hwnd: isize,
) -> Result<(IUIAutomationElement, IUIAutomationTreeWalker)> {
    // SAFETY: `hwnd` identifica una ventana viva y COM está inicializado en este hilo.
    unsafe {
        let raiz = sesion
            .auto
            .ElementFromHandleBuildCache(HWND(hwnd as *mut c_void), &sesion.req)
            .map_err(error)?;
        let walker = sesion.auto.ControlViewWalker().map_err(error)?;
        Ok((raiz, walker))
    }
}

/// Cached accessible name of `el`.
pub(crate) fn nombre(el: &IUIAutomationElement) -> windows::core::Result<String> {
    // SAFETY: Lectura COM en caché sobre una interfaz viva.
    unsafe { el.CachedName() }.map(|s| s.to_string())
}

/// Cached control type id of `el`.
pub(crate) fn tipo_control(el: &IUIAutomationElement) -> windows::core::Result<i32> {
    // SAFETY: Lectura COM en caché sobre una interfaz viva.
    unsafe { el.CachedControlType() }.map(|t| t.0)
}

/// Cached bounding rectangle of `el`.
pub(crate) fn rectangulo(el: &IUIAutomationElement) -> windows::core::Result<RECT> {
    // SAFETY: Lectura COM en caché sobre una interfaz viva.
    unsafe { el.CachedBoundingRectangle() }
}

/// Reads every cached property used by the tree snapshot in one COM block.
pub(crate) fn propiedades(el: &IUIAutomationElement) -> PropiedadesCom {
    // SAFETY: Lecturas COM en caché sobre una interfaz viva; cada error se conserva.
    unsafe {
        PropiedadesCom {
            nombre: el.CachedName().map(|s| s.to_string()),
            tipo: el.CachedControlType().map(|t| t.0),
            automation_id: el.CachedAutomationId().map(|s| s.to_string()),
            clase: el.CachedClassName().map(|s| s.to_string()),
            rect: el.CachedBoundingRectangle(),
            hwnd: el.CachedNativeWindowHandle().map(|h| h.0 as isize),
        }
    }
}

/// Pattern labels cached by the session and supported by `el`.
pub(crate) fn patrones_soportados(el: &IUIAutomationElement) -> Vec<&'static str> {
    // SAFETY: Lecturas COM en caché sobre una interfaz viva; un error solo omite la etiqueta.
    unsafe {
        [
            (UIA_InvokePatternId, "invoke"),
            (UIA_ValuePatternId, "valor"),
            (UIA_TogglePatternId, "alternar"),
            (UIA_SelectionItemPatternId, "seleccion"),
            (UIA_ExpandCollapsePatternId, "expandir"),
        ]
        .into_iter()
        .filter(|(id, _)| {
            el.GetCachedPattern(*id)
                .is_ok_and(|p| !p.as_raw().is_null())
        })
        .map(|(_, etiqueta)| etiqueta)
        .collect()
    }
}

fn es_fin_de_lista(e: &windows::core::Error) -> bool {
    e.code().is_ok()
}

/// Direct children of `el` in control view, with the session cache applied.
pub(crate) fn hijos(
    sesion: &Sesion,
    walker: &IUIAutomationTreeWalker,
    el: &IUIAutomationElement,
) -> (Vec<IUIAutomationElement>, Option<windows::core::Error>) {
    let mut out = Vec::new();
    // SAFETY: Llamadas COM sobre interfaces vivas; el fin de lista llega como S_OK.
    unsafe {
        let mut actual = match walker.GetFirstChildElementBuildCache(el, &sesion.req) {
            Ok(c) => c,
            Err(e) if es_fin_de_lista(&e) => return (out, None),
            Err(e) => return (out, Some(e)),
        };
        loop {
            out.push(actual.clone());
            match walker.GetNextSiblingElementBuildCache(&actual, &sesion.req) {
                Ok(n) => actual = n,
                Err(e) if es_fin_de_lista(&e) => break,
                Err(e) => return (out, Some(e)),
            }
        }
    }
    (out, None)
}

const OPERACIONES: &str = "invocar|fijar_valor|alternar|seleccionar|expandir|contraer";

fn desconocida(accion: &str) -> Error {
    Error::Gui(format!(
        "Operación UIA desconocida '{accion}' ({OPERACIONES})"
    ))
}

/// Runs one UIA pattern operation on `el`.
pub(crate) fn operar(el: &IUIAutomationElement, accion: &str, valor: Option<&str>) -> Result<()> {
    match accion {
        "invocar" | "alternar" | "seleccionar" => accion_simple(el, accion),
        "expandir" | "contraer" | "fijar_valor" => accion_estado(el, accion, valor),
        other => Err(desconocida(other)),
    }
}

fn accion_simple(el: &IUIAutomationElement, accion: &str) -> Result<()> {
    // SAFETY: Patrones COM obtenidos y usados sobre una interfaz viva en este hilo.
    unsafe {
        match accion {
            "invocar" => el
                .GetCurrentPattern(UIA_InvokePatternId)
                .map_err(error)?
                .cast::<IUIAutomationInvokePattern>()
                .map_err(error)?
                .Invoke()
                .map_err(error),
            "alternar" => el
                .GetCurrentPattern(UIA_TogglePatternId)
                .map_err(error)?
                .cast::<IUIAutomationTogglePattern>()
                .map_err(error)?
                .Toggle()
                .map_err(error),
            "seleccionar" => el
                .GetCurrentPattern(UIA_SelectionItemPatternId)
                .map_err(error)?
                .cast::<IUIAutomationSelectionItemPattern>()
                .map_err(error)?
                .Select()
                .map_err(error),
            other => Err(desconocida(other)),
        }
    }
}

fn accion_estado(el: &IUIAutomationElement, accion: &str, valor: Option<&str>) -> Result<()> {
    // SAFETY: Patrones COM obtenidos y usados sobre una interfaz viva en este hilo.
    unsafe {
        match accion {
            "expandir" => el
                .GetCurrentPattern(UIA_ExpandCollapsePatternId)
                .map_err(error)?
                .cast::<IUIAutomationExpandCollapsePattern>()
                .map_err(error)?
                .Expand()
                .map_err(error),
            "contraer" => el
                .GetCurrentPattern(UIA_ExpandCollapsePatternId)
                .map_err(error)?
                .cast::<IUIAutomationExpandCollapsePattern>()
                .map_err(error)?
                .Collapse()
                .map_err(error),
            "fijar_valor" => {
                let v =
                    valor.ok_or_else(|| Error::Gui("Indique 'valor' para fijar_valor".into()))?;
                let bstr = BSTR::from(v);
                el.GetCurrentPattern(UIA_ValuePatternId)
                    .map_err(error)?
                    .cast::<IUIAutomationValuePattern>()
                    .map_err(error)?
                    .SetValue(&bstr)
                    .map_err(error)
            }
            other => Err(desconocida(other)),
        }
    }
}
