//! Hybrid GUI view fusing Win32 window handles with the UI Automation tree.

use super::ffi::enum_children;
use super::uia_paso::raiz_por_defecto;
use super::ventanas::{hwnd_de, ventanas_de, window_brief};
use crate::adapters::uia;
use crate::args::flag;
use crate::error::{Error, Result};
use serde_json::{json, Value};

const DEFAULT_UIA_DEPTH: u32 = 4;
const DEFAULT_UIA_MAX_NODES: u32 = 150;

pub(crate) fn aplanar<'a>(nodo: &'a Value, out: &mut Vec<&'a Value>) {
    for hijo in nodo
        .get("hijos")
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
    {
        out.push(hijo);
        aplanar(hijo, out);
    }
}

pub(crate) fn resumen_uia(n: &Value) -> Value {
    let sin_hijos = match n.get("hijos").and_then(Value::as_array) {
        Some(h) => h.is_empty(),
        None => true,
    };
    let rol = n.get("rol").and_then(Value::as_str).unwrap_or("");
    let no_vacio = n
        .get("rect")
        .and_then(Value::as_array)
        .is_some_and(|r| match r.as_slice() {
            [x0, y0, x1, y1] => {
                x1.as_f64().unwrap_or(0.0) > x0.as_f64().unwrap_or(0.0)
                    && y1.as_f64().unwrap_or(0.0) > y0.as_f64().unwrap_or(0.0)
            }
            _ => false,
        });
    let solo_vision = sin_hijos && matches!(rol, "pane" | "custom" | "image") && no_vacio;
    json!({
        "rol": n.get("rol"),
        "nombre": n.get("nombre"),
        "rect": n.get("rect"),
        "patrones": n.get("patrones"),
        "solo_vision": solo_vision,
    })
}

fn asociar_nodo_uia(v: &mut Value, plano: &[&Value]) {
    let Some(n) = plano
        .iter()
        .find(|n| n.get("hwnd").is_some() && n.get("hwnd") == v.get("hwnd"))
    else {
        return;
    };
    let Some(obj) = v.as_object_mut() else {
        return;
    };
    obj.insert(
        "uia".into(),
        json!({
            "nombre": n.get("nombre"),
            "rol": n.get("rol"),
            "patrones": n.get("patrones"),
            "rect": n.get("rect"),
        }),
    );
}

fn nodos_sin_hwnd(plano: &[&Value]) -> Vec<Value> {
    plano
        .iter()
        .copied()
        .filter(|n| n.get("hwnd").is_none())
        .map(resumen_uia)
        .collect()
}

pub(crate) fn fusionar(ventanas_win: Vec<Value>, arbol: &Value) -> Value {
    let mut plano: Vec<&Value> = Vec::new();
    aplanar(arbol, &mut plano);
    let ventanas: Vec<Value> = ventanas_win
        .into_iter()
        .map(|mut v| {
            asociar_nodo_uia(&mut v, &plano);
            v
        })
        .collect();
    let sin_hwnd = nodos_sin_hwnd(&plano);
    json!({ "ventanas": ventanas, "uia_sin_hwnd": sin_hwnd })
}

pub(crate) fn controles_con_uia(paths: &crate::config::Paths, args: &Value) -> Result<Value> {
    let mut r = super::ventanas::controles(args)?;
    if flag(args, &["incluir_uia"]) {
        let h = match args.get("ventana").and_then(Value::as_str) {
            Some(_) => hwnd_de(args, "ventana")?,
            None => raiz_por_defecto(&ventanas_de(args)?)
                .ok_or_else(|| Error::Gui("No hay una ventana visible para inspeccionar".into()))?,
        };
        let arbol = uia::arbol(&paths.estado, h, DEFAULT_UIA_DEPTH, DEFAULT_UIA_MAX_NODES)?;
        let mut plano = Vec::new();
        aplanar(&arbol, &mut plano);
        let uia_sin = nodos_sin_hwnd(&plano);
        if let Some(obj) = r.as_object_mut() {
            obj.insert("uia".into(), json!(uia_sin));
        }
    }
    Ok(r)
}

pub(crate) fn observar_uia(paths: &crate::config::Paths, args: &Value) -> Result<Value> {
    let h = hwnd_de(args, "ventana")?;
    let profundidad = args
        .get("profundidad")
        .and_then(Value::as_u64)
        .unwrap_or(u64::from(DEFAULT_UIA_DEPTH)) as u32;
    let max_nodos = args
        .get("max_nodos")
        .and_then(Value::as_u64)
        .unwrap_or(u64::from(DEFAULT_UIA_MAX_NODES)) as u32;
    Ok(json!({
        "hwnd": format!("0x{:X}", h),
        "arbol": uia::arbol(&paths.estado, h, profundidad, max_nodos)?,
    }))
}

pub(crate) fn observar_arbol(paths: &crate::config::Paths, args: &Value) -> Result<Value> {
    let h = hwnd_de(args, "ventana")?;
    let win: Vec<Value> = enum_children(h).into_iter().map(window_brief).collect();
    let arbol = uia::arbol(&paths.estado, h, DEFAULT_UIA_DEPTH, DEFAULT_UIA_MAX_NODES)?;
    Ok(json!({
        "hwnd": format!("0x{:X}", h),
        "arbol": fusionar(win, &arbol),
    }))
}

#[cfg(test)]
#[path = "../../../tests/unit/adapters/gui/uia_vista.rs"]
mod tests;
