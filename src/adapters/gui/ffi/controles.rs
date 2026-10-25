//! Control-level Win32 calls: text in and out of edit controls, combo boxes,
//! tab pages and keyboard navigation.

use super::ventanas::{es_ventana, send_message_timeout, Mensaje, TIMEOUT_MENSAJE_MS};
use super::Hwnd;
use crate::error::{Error, Result};

const WM_GETTEXT: u32 = 0x000D;
const WM_CHAR: u32 = 0x0102;
const EM_SETSEL: u32 = 0x00B1;
const CB_GETCOUNT: u32 = 0x0146;
const CB_GETLBTEXT: u32 = 0x0148;
const CB_GETLBTEXTLEN: u32 = 0x0149;
const CB_FINDSTRING: u32 = 0x014C;
const CB_SETCURSEL: u32 = 0x014E;
const LB_FINDSTRING: u32 = 0x018F;
const LB_SETCURSEL: u32 = 0x0186;
const LB_SETSEL: u32 = 0x0185;
const GWL_STYLE: i32 = -16;
/// `LBS_MULTIPLESEL | LBS_EXTENDEDSEL`: the list keeps several selected items.
const ESTILO_MULTISELECCION: isize = 0x0008 | 0x0800;
pub(crate) const WM_COMMAND: u32 = 0x0111;
const CBN_SELCHANGE: usize = 1;
/// `CB_FINDSTRING` result when no item matches.
const CB_ERR: isize = -1;
/// `CB_FINDSTRING` start index meaning "search the whole list".
const DESDE_EL_PRINCIPIO: usize = usize::MAX;
/// VCL class of a page control (the tab strip and its pages).
pub(crate) const CLASE_PESTANAS: &str = "TPageControl";
const TCM_GETITEMCOUNT: u32 = 0x1304;
const TCM_GETCURSEL: u32 = 0x130B;
const TCM_SETCURFOCUS: u32 = 0x1330;
/// Longest text read back from a PVsyst edit control.
const LARGO_TEXTO: usize = 1024;

#[link(name = "user32")]
extern "system" {
    fn GetParent(h: Hwnd) -> Hwnd;
    fn GetDlgCtrlID(h: Hwnd) -> i32;
    fn GetWindowLongPtrW(h: Hwnd, indice: i32) -> isize;
}

/// Text of a control in another process (`WM_GETTEXT` is marshalled by Windows).
pub(crate) fn texto_control(h: Hwnd) -> String {
    let mut buf = vec![0u16; LARGO_TEXTO];
    let Some(res) = send_message_timeout(
        h,
        Mensaje::nuevo(WM_GETTEXT, buf.len(), buf.as_mut_ptr() as isize),
        TIMEOUT_MENSAJE_MS,
    ) else {
        return String::new();
    };
    let n = usize::try_from(res).unwrap_or(0).min(buf.len());
    let slice = buf.get(..n).unwrap_or(&[]);
    String::from_utf16_lossy(slice)
}

/// Text of item `indice` of a combo box.
fn texto_de_item(h: Hwnd, indice: usize) -> Option<String> {
    let res_len = send_message_timeout(
        h,
        Mensaje::nuevo(CB_GETLBTEXTLEN, indice, 0),
        TIMEOUT_MENSAJE_MS,
    )?;
    if res_len < 0 {
        return None;
    }
    let largo = usize::try_from(res_len).ok()?;
    let mut buf = vec![0u16; largo + 1];
    let res_txt = send_message_timeout(
        h,
        Mensaje::nuevo(CB_GETLBTEXT, indice, buf.as_mut_ptr() as isize),
        TIMEOUT_MENSAJE_MS,
    )?;
    if res_txt < 0 {
        return None;
    }
    let n = usize::try_from(res_txt).ok()?.min(largo);
    let slice = buf.get(..n).unwrap_or(&[]);
    Some(String::from_utf16_lossy(slice))
}

/// Items of a combo box in order: what a user could pick. Complexity: O(items).
pub(crate) fn opciones_de_combo(h: Hwnd) -> Vec<String> {
    let total_res =
        send_message_timeout(h, Mensaje::simple(CB_GETCOUNT), TIMEOUT_MENSAJE_MS).unwrap_or(0);
    if total_res <= 0 {
        return Vec::new();
    }
    (0..usize::try_from(total_res).unwrap_or(0))
        .filter_map(|i| texto_de_item(h, i))
        .collect()
}

/// Replaces the content of a focused edit control as if typed, so the owning
/// form sees every change (VCL recalculates on each keystroke).
pub(crate) fn teclear_en(h: Hwnd, texto: &str) -> Result<()> {
    send_message_timeout(h, Mensaje::nuevo(EM_SETSEL, 0, -1), TIMEOUT_MENSAJE_MS)
        .ok_or_else(|| Error::Gui("PVsyst no respondió en 3 s al seleccionar texto".into()))?;
    for unidad in texto.encode_utf16() {
        send_message_timeout(
            h,
            Mensaje::nuevo(WM_CHAR, usize::from(unidad), 0),
            TIMEOUT_MENSAJE_MS,
        )
        .ok_or_else(|| Error::Gui("PVsyst no respondió en 3 s al enviar carácter".into()))?;
    }
    Ok(())
}

/// Shows tab page `indice` of a page control.
pub(crate) fn mostrar_pagina(h: Hwnd, indice: usize) -> Result<()> {
    send_message_timeout(
        h,
        Mensaje::nuevo(TCM_SETCURFOCUS, indice, 0),
        TIMEOUT_MENSAJE_MS,
    )
    .ok_or_else(|| Error::Gui("PVsyst no respondió en 3 s al cambiar de pestaña".into()))?;
    Ok(())
}

/// Index of the selected tab page of a page control; `None` without a selection
/// or when the control does not answer.
pub(crate) fn pagina_seleccionada(h: Hwnd) -> Option<usize> {
    let indice = send_message_timeout(h, Mensaje::nuevo(TCM_GETCURSEL, 0, 0), TIMEOUT_MENSAJE_MS)?;
    usize::try_from(indice).ok()
}

/// Number of tab pages of a page control; `None` when the control does not answer.
pub(crate) fn paginas_de(h: Hwnd) -> Option<usize> {
    let total = send_message_timeout(h, Mensaje::nuevo(TCM_GETITEMCOUNT, 0, 0), TIMEOUT_MENSAJE_MS)?;
    usize::try_from(total).ok()
}

/// Window messages of a list control: find an item, select it.
struct MensajesLista {
    buscar: u32,
    elegir: u32,
}

const COMBO: MensajesLista = MensajesLista {
    buscar: CB_FINDSTRING,
    elegir: CB_SETCURSEL,
};
const LISTA: MensajesLista = MensajesLista {
    buscar: LB_FINDSTRING,
    elegir: LB_SETCURSEL,
};

/// Whether the list box `h` keeps several items selected at once.
fn es_multiseleccion(h: Hwnd) -> bool {
    if !es_ventana(h) {
        return false;
    }
    // SAFETY: Consulta de solo lectura del estilo de un HWND validado; devuelve un entero.
    unsafe { GetWindowLongPtrW(h, GWL_STYLE) & ESTILO_MULTISELECCION != 0 }
}

/// Makes item `indice` the selection. In a multi-select list `LB_SETCURSEL` is
/// ignored, so the previous selection is cleared and the item set explicitly.
fn marcar_indice(h: Hwnd, mensajes: &MensajesLista, indice: usize) -> Result<()> {
    if !es_ventana(h) {
        return Err(Error::Gui("La ventana ya no existe".into()));
    }
    if mensajes.elegir == LB_SETCURSEL && es_multiseleccion(h) {
        send_message_timeout(h, Mensaje::nuevo(LB_SETSEL, 0, -1), TIMEOUT_MENSAJE_MS)
            .ok_or_else(|| Error::Gui("PVsyst no respondió en 3 s".into()))?;
        send_message_timeout(
            h,
            Mensaje::nuevo(LB_SETSEL, 1, isize::try_from(indice).unwrap_or(0)),
            TIMEOUT_MENSAJE_MS,
        )
        .ok_or_else(|| Error::Gui("PVsyst no respondió en 3 s".into()))?;
        return Ok(());
    }
    send_message_timeout(
        h,
        Mensaje::nuevo(mensajes.elegir, indice, 0),
        TIMEOUT_MENSAJE_MS,
    )
    .ok_or_else(|| Error::Gui("PVsyst no respondió en 3 s".into()))?;
    Ok(())
}

fn elegir_item(h: Hwnd, mensajes: &MensajesLista, prefijo: &str) -> Result<bool> {
    if !es_ventana(h) {
        return Ok(false);
    }
    let texto: Vec<u16> = prefijo.encode_utf16().chain(std::iter::once(0)).collect();
    let indice_res = send_message_timeout(
        h,
        Mensaje::nuevo(mensajes.buscar, DESDE_EL_PRINCIPIO, texto.as_ptr() as isize),
        TIMEOUT_MENSAJE_MS,
    )
    .ok_or_else(|| Error::Gui("PVsyst no respondió en 3 s al buscar elemento".into()))?;
    if indice_res == CB_ERR {
        return Ok(false);
    }
    marcar_indice(h, mensajes, usize::try_from(indice_res).unwrap_or(0))?;
    // SAFETY: Consultas sobre ventana padre e id del control hijo.
    let (padre, id) = unsafe {
        let padre = GetParent(h);
        let id = usize::try_from(GetDlgCtrlID(h)).unwrap_or(0) & 0xFFFF;
        (padre, id)
    };
    if es_ventana(padre) {
        send_message_timeout(
            padre,
            Mensaje::nuevo(WM_COMMAND, (CBN_SELCHANGE << 16) | id, h),
            TIMEOUT_MENSAJE_MS,
        )
        .ok_or_else(|| Error::Gui("PVsyst no respondió en 3 s al notificar cambio".into()))?;
    }
    Ok(true)
}

/// Selects the first combo-box item starting with `prefijo` and notifies the
/// owning form as a user choice would. `false` when no item matches.
pub(crate) fn elegir_en_combo(h: Hwnd, prefijo: &str) -> Result<bool> {
    elegir_item(h, &COMBO, prefijo)
}

/// Same for a list box (`TListBox`): the first item starting with `prefijo`.
pub(crate) fn elegir_en_lista(h: Hwnd, prefijo: &str) -> Result<bool> {
    elegir_item(h, &LISTA, prefijo)
}

/// Picks an item by prefix in a combo box or a list box, by control class.
pub(crate) fn elegir_item_por_clase(h: Hwnd, clase: &str, prefijo: &str) -> Result<bool> {
    if clase == "TListBox" {
        elegir_en_lista(h, prefijo)
    } else {
        elegir_en_combo(h, prefijo)
    }
}

#[cfg(test)]
#[path = "../../../../tests/unit/adapters/gui/ffi/controles.rs"]
mod tests;
