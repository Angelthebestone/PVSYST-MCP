//! UI Automation bridge for PVsyst windows: element tree snapshots, pattern
//! invocation and element centres, through `IUIAutomation` on the MTA. Every
//! call owns its own COM apartment and releases it before returning.

pub(crate) mod arbol;
pub(crate) mod com;
pub(crate) mod elemento;

pub(crate) use arbol::arbol;
pub(crate) use elemento::{centro, invocar};

use crate::error::{Error, Result};
use windows::Win32::System::Com::{
    CoCreateInstance, CoInitializeEx, CoUninitialize, CLSCTX_INPROC_SERVER, COINIT_MULTITHREADED,
};
use windows::Win32::UI::Accessibility::{CUIAutomation, IUIAutomation, IUIAutomationCacheRequest};

/// COM apartment guard: uninitialises the thread's MTA when dropped.
pub(crate) struct Com;

impl Com {
    pub(crate) fn init() -> Result<Self> {
        // SAFETY: Se empareja con `CoUninitialize` en `Drop`; no recibe punteros del llamador.
        let hr = unsafe { CoInitializeEx(None, COINIT_MULTITHREADED) };
        if hr.is_err() {
            return Err(Error::Gui(format!("CoInitializeEx falló: {hr:?}")));
        }
        Ok(Self)
    }
}

impl Drop for Com {
    fn drop(&mut self) {
        // SAFETY: Solo existe un `Com` si `CoInitializeEx` tuvo éxito (ver `init`), y `Drop` corre en ese mismo hilo.
        unsafe { CoUninitialize() };
    }
}

#[expect(
    clippy::needless_pass_by_value,
    reason = "used as a `map_err` function pointer"
)]
pub(crate) fn error(e: windows::core::Error) -> Error {
    Error::Gui(e.to_string())
}

/// Maps a UIA control-type id to the role names used in tool arguments.
pub(crate) fn rol_de(tipo: i32) -> &'static str {
    match tipo {
        50000 => "button",
        50002 => "checkbox",
        50003 => "combo box",
        50004 => "edit",
        50005 => "hyperlink",
        50006 => "image",
        50007 => "list item",
        50008 => "list",
        50009 => "menu",
        50010 => "menu bar",
        50011 => "menu item",
        50013 => "radio button",
        50014 => "scroll bar",
        50015 => "slider",
        50018 => "tab",
        50019 => "tab item",
        50020 => "text",
        50021 => "tool bar",
        50023 => "tree",
        50024 => "tree item",
        50025 => "custom",
        50026 => "group",
        50028 => "data grid",
        50029 => "data item",
        50030 => "document",
        50032 => "window",
        50033 => "pane",
        50034 => "header",
        50035 => "header item",
        50036 => "table",
        50037 => "title bar",
        50038 => "separator",
        _ => "otro",
    }
}

/// One automation instance plus the cache request used for every element read.
/// Field order matters: the COM guard is declared last so it drops last.
pub(crate) struct Sesion {
    pub(crate) auto: IUIAutomation,
    pub(crate) req: IUIAutomationCacheRequest,
    _com: Com,
}

impl Sesion {
    pub(crate) fn nueva() -> Result<Self> {
        let guard = Com::init()?;
        let auto: IUIAutomation =
            // SAFETY: CLSID y contexto constantes; COM ya está inicializado en este hilo.
            unsafe { CoCreateInstance(&CUIAutomation, None, CLSCTX_INPROC_SERVER) }
                .map_err(error)?;
        let req = com::configurar_cache(&auto)?;
        Ok(Self {
            auto,
            req,
            _com: guard,
        })
    }
}

#[cfg(test)]
#[path = "../../../tests/unit/adapters/uia/mod.rs"]
mod tests;
