//! Control-level Win32 calls: text in and out of edit controls, combo boxes,
//! tab pages and keyboard navigation.

use super::ventanas::{send_message, Mensaje};
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
/// Longest text read back from a PVsyst edit control.
const LARGO_TEXTO: usize = 1024;

#[link(name = "user32")]
extern "system" {
    fn SendMessageW(h: Hwnd, msg: u32, w: usize, l: isize) -> isize;
    fn GetParent(h: Hwnd) -> Hwnd;
    fn GetDlgCtrlID(h: Hwnd) -> i32;
    fn GetWindowLongPtrW(h: Hwnd, indice: i32) -> isize;
}

/// Text of a control in another process (`WM_GETTEXT` is marshalled by Windows).
pub(crate) fn texto_control(h: Hwnd) -> String {
    let mut buf = vec![0u16; LARGO_TEXTO];
    // SAFETY: `buf` vive durante la llamada síncrona en este marco de pila y su longitud
    // se pasa en `wparam`; `SendMessageW` no escribe más allá de `buf.len()` unidades.
    let n = unsafe { SendMessageW(h, WM_GETTEXT, buf.len(), buf.as_mut_ptr() as isize) };
    let n = usize::try_from(n).unwrap_or(0).min(buf.len());
    let slice = buf.get(..n).unwrap_or(&[]);
    String::from_utf16_lossy(slice)
}

/// Text of item `indice` of a combo box.
fn texto_de_item(h: Hwnd, indice: usize) -> Option<String> {
    // SAFETY: Consulta síncrona a un control COMBOBOX.
    let largo = unsafe { SendMessageW(h, CB_GETLBTEXTLEN, indice, 0) };
    let largo = usize::try_from(largo).ok()?;
    let mut buf = vec![0u16; largo + 1];
    // SAFETY: `buf` tiene capacidad para `largo + 1` caracteres UTF-16; la API escribe como máximo
    // la longitud reportada por `CB_GETLBTEXTLEN` más el terminador nulo.
    let n = unsafe { SendMessageW(h, CB_GETLBTEXT, indice, buf.as_mut_ptr() as isize) };
    let n = usize::try_from(n).ok()?.min(largo);
    let slice = buf.get(..n).unwrap_or(&[]);
    Some(String::from_utf16_lossy(slice))
}

/// Items of a combo box in order: what a user could pick. Complexity: O(items).
pub(crate) fn opciones_de_combo(h: Hwnd) -> Vec<String> {
    // SAFETY: Consulta síncrona pasando HWND.
    let total = unsafe { SendMessageW(h, CB_GETCOUNT, 0, 0) };
    (0..usize::try_from(total).unwrap_or(0))
        .filter_map(|i| texto_de_item(h, i))
        .collect()
}

/// Replaces the content of a focused edit control as if typed, so the owning
/// form sees every change (VCL recalculates on each keystroke).
pub(crate) fn teclear_en(h: Hwnd, texto: &str) {
    send_message(h, Mensaje::nuevo(EM_SETSEL, 0, -1));
    for unidad in texto.encode_utf16() {
        send_message(h, Mensaje::nuevo(WM_CHAR, usize::from(unidad), 0));
    }
}

/// Shows tab page `indice` of a page control.
pub(crate) fn mostrar_pagina(h: Hwnd, indice: usize) {
    send_message(h, Mensaje::nuevo(TCM_SETCURFOCUS, indice, 0));
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
    // SAFETY: Consulta de solo lectura del estilo de un HWND; devuelve un entero.
    unsafe { GetWindowLongPtrW(h, GWL_STYLE) & ESTILO_MULTISELECCION != 0 }
}

/// Makes item `indice` the selection. In a multi-select list `LB_SETCURSEL` is
/// ignored, so the previous selection is cleared and the item set explicitly.
fn marcar_indice(h: Hwnd, mensajes: &MensajesLista, indice: usize) {
    if mensajes.elegir == LB_SETCURSEL && es_multiseleccion(h) {
        send_message(h, Mensaje::nuevo(LB_SETSEL, 0, -1));
        send_message(
            h,
            Mensaje::nuevo(LB_SETSEL, 1, isize::try_from(indice).unwrap_or(0)),
        );
        return;
    }
    send_message(h, Mensaje::nuevo(mensajes.elegir, indice, 0));
}

fn elegir_item(h: Hwnd, mensajes: &MensajesLista, prefijo: &str) -> bool {
    let texto: Vec<u16> = prefijo.encode_utf16().chain(std::iter::once(0)).collect();
    // SAFETY: `texto` es una cadena UTF-16 terminada en nulo viva durante la llamada;
    // Windows la serializa entre procesos de forma síncrona.
    let indice = unsafe {
        SendMessageW(
            h,
            mensajes.buscar,
            DESDE_EL_PRINCIPIO,
            texto.as_ptr() as isize,
        )
    };
    if indice == CB_ERR {
        return false;
    }
    marcar_indice(h, mensajes, usize::try_from(indice).unwrap_or(0));
    // SAFETY: Llamadas auxiliares sobre ventanas; enteros por valor.
    let (padre, id) = unsafe {
        let padre = GetParent(h);
        let id = usize::try_from(GetDlgCtrlID(h)).unwrap_or(0) & 0xFFFF;
        (padre, id)
    };
    send_message(
        padre,
        Mensaje::nuevo(WM_COMMAND, (CBN_SELCHANGE << 16) | id, h),
    );
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
