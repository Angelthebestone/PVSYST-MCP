//! Control-level Win32 calls: text in and out of edit controls, combo boxes,
//! tab pages and keyboard navigation.

use super::ventanas::{es_ventana, send_message_timeout, Mensaje};
use super::Hwnd;

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
const TCM_SETCURFOCUS: u32 = 0x1330;
const SMTO_ABORTIFHUNG: u32 = 0x0002;
const TIMEOUT_MENSAJE_MS: u32 = 3000;
/// Longest text read back from a PVsyst edit control.
const LARGO_TEXTO: usize = 1024;

#[link(name = "user32")]
extern "system" {
    fn SendMessageTimeoutW(
        h: Hwnd,
        msg: u32,
        w: usize,
        l: isize,
        flags: u32,
        timeout: u32,
        res: *mut isize,
    ) -> isize;
    fn GetParent(h: Hwnd) -> Hwnd;
    fn GetDlgCtrlID(h: Hwnd) -> i32;
    fn GetWindowLongPtrW(h: Hwnd, indice: i32) -> isize;
}

/// Text of a control in another process (`WM_GETTEXT` is marshalled by Windows).
pub(crate) fn texto_control(h: Hwnd) -> String {
    if !es_ventana(h) {
        return String::new();
    }
    let mut buf = vec![0u16; LARGO_TEXTO];
    let mut res = 0isize;
    // SAFETY: `buf` vive durante la llamada síncrona en este marco de pila y su longitud
    // se pasa en `wparam`; `SendMessageTimeoutW` no escribe más allá de `buf.len()` unidades.
    let ok = unsafe {
        SendMessageTimeoutW(
            h,
            WM_GETTEXT,
            buf.len(),
            buf.as_mut_ptr() as isize,
            SMTO_ABORTIFHUNG,
            TIMEOUT_MENSAJE_MS,
            &mut res,
        )
    };
    if ok == 0 {
        return String::new();
    }
    let n = usize::try_from(res).unwrap_or(0).min(buf.len());
    let slice = buf.get(..n).unwrap_or(&[]);
    String::from_utf16_lossy(slice)
}

/// Text of item `indice` of a combo box.
fn texto_de_item(h: Hwnd, indice: usize) -> Option<String> {
    if !es_ventana(h) {
        return None;
    }
    let mut res_len = 0isize;
    // SAFETY: Consulta síncrona a un control COMBOBOX validado con tiempo límite.
    let ok_len = unsafe {
        SendMessageTimeoutW(
            h,
            CB_GETLBTEXTLEN,
            indice,
            0,
            SMTO_ABORTIFHUNG,
            TIMEOUT_MENSAJE_MS,
            &mut res_len,
        )
    };
    if ok_len == 0 {
        return None;
    }
    let largo = usize::try_from(res_len).ok()?;
    let mut buf = vec![0u16; largo + 1];
    let mut res_txt = 0isize;
    // SAFETY: `buf` tiene capacidad para `largo + 1` caracteres UTF-16; la API escribe como máximo
    // la longitud reportada por `CB_GETLBTEXTLEN` más el terminador nulo.
    let ok_txt = unsafe {
        SendMessageTimeoutW(
            h,
            CB_GETLBTEXT,
            indice,
            buf.as_mut_ptr() as isize,
            SMTO_ABORTIFHUNG,
            TIMEOUT_MENSAJE_MS,
            &mut res_txt,
        )
    };
    if ok_txt == 0 {
        return None;
    }
    let n = usize::try_from(res_txt).ok()?.min(largo);
    let slice = buf.get(..n).unwrap_or(&[]);
    Some(String::from_utf16_lossy(slice))
}

/// Items of a combo box in order: what a user could pick. Complexity: O(items).
pub(crate) fn opciones_de_combo(h: Hwnd) -> Vec<String> {
    if !es_ventana(h) {
        return Vec::new();
    }
    let mut total_res = 0isize;
    // SAFETY: Consulta síncrona pasando HWND validado; timeout previene bloqueos si la GUI cuelga.
    let ok = unsafe {
        SendMessageTimeoutW(
            h,
            CB_GETCOUNT,
            0,
            0,
            SMTO_ABORTIFHUNG,
            TIMEOUT_MENSAJE_MS,
            &mut total_res,
        )
    };
    if ok == 0 {
        return Vec::new();
    }
    (0..usize::try_from(total_res).unwrap_or(0))
        .filter_map(|i| texto_de_item(h, i))
        .collect()
}

/// Replaces the content of a focused edit control as if typed, so the owning
/// form sees every change (VCL recalculates on each keystroke).
pub(crate) fn teclear_en(h: Hwnd, texto: &str) {
    if !es_ventana(h) {
        return;
    }
    send_message_timeout(h, Mensaje::nuevo(EM_SETSEL, 0, -1), TIMEOUT_MENSAJE_MS);
    for unidad in texto.encode_utf16() {
        send_message_timeout(
            h,
            Mensaje::nuevo(WM_CHAR, usize::from(unidad), 0),
            TIMEOUT_MENSAJE_MS,
        );
    }
}

/// Shows tab page `indice` of a page control.
pub(crate) fn mostrar_pagina(h: Hwnd, indice: usize) {
    if !es_ventana(h) {
        return;
    }
    send_message_timeout(
        h,
        Mensaje::nuevo(TCM_SETCURFOCUS, indice, 0),
        TIMEOUT_MENSAJE_MS,
    );
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
fn marcar_indice(h: Hwnd, mensajes: &MensajesLista, indice: usize) {
    if !es_ventana(h) {
        return;
    }
    if mensajes.elegir == LB_SETCURSEL && es_multiseleccion(h) {
        send_message_timeout(h, Mensaje::nuevo(LB_SETSEL, 0, -1), TIMEOUT_MENSAJE_MS);
        send_message_timeout(
            h,
            Mensaje::nuevo(LB_SETSEL, 1, isize::try_from(indice).unwrap_or(0)),
            TIMEOUT_MENSAJE_MS,
        );
        return;
    }
    send_message_timeout(
        h,
        Mensaje::nuevo(mensajes.elegir, indice, 0),
        TIMEOUT_MENSAJE_MS,
    );
}

fn elegir_item(h: Hwnd, mensajes: &MensajesLista, prefijo: &str) -> bool {
    if !es_ventana(h) {
        return false;
    }
    let texto: Vec<u16> = prefijo.encode_utf16().chain(std::iter::once(0)).collect();
    let mut indice_res = 0isize;
    // SAFETY: `texto` es una cadena UTF-16 terminada en nulo viva durante la llamada;
    // Windows la serializa entre procesos de forma síncrona con tiempo límite.
    let ok = unsafe {
        SendMessageTimeoutW(
            h,
            mensajes.buscar,
            DESDE_EL_PRINCIPIO,
            texto.as_ptr() as isize,
            SMTO_ABORTIFHUNG,
            TIMEOUT_MENSAJE_MS,
            &mut indice_res,
        )
    };
    if ok == 0 || indice_res == CB_ERR {
        return false;
    }
    marcar_indice(h, mensajes, usize::try_from(indice_res).unwrap_or(0));
    // SAFETY: Llamadas auxiliares sobre ventanas validadas; enteros por valor.
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
        );
    }
    true
}

/// Selects the first combo-box item starting with `prefijo` and notifies the
/// owning form as a user choice would. `false` when no item matches.
pub(crate) fn elegir_en_combo(h: Hwnd, prefijo: &str) -> bool {
    elegir_item(h, &COMBO, prefijo)
}

/// Same for a list box (`TListBox`): the first item starting with `prefijo`.
pub(crate) fn elegir_en_lista(h: Hwnd, prefijo: &str) -> bool {
    elegir_item(h, &LISTA, prefijo)
}

/// Picks an item by prefix in a combo box or a list box, by control class.
pub(crate) fn elegir_item_por_clase(h: Hwnd, clase: &str, prefijo: &str) -> bool {
    if clase == "TListBox" {
        elegir_en_lista(h, prefijo)
    } else {
        elegir_en_combo(h, prefijo)
    }
}

#[cfg(test)]
#[path = "../../../../tests/unit/adapters/gui/ffi/controles.rs"]
mod tests;
