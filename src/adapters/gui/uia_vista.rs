//! Hybrid GUI view fusing Win32 window handles with the UI Automation tree.

use super::acciones::{clic_fisico_en, escritorio_interactivo};
use super::esperas::{esperar_ventana, EsperarParams};
use super::ffi::{self, enum_children, Hwnd, BM_CLICK};
use super::teclado::teclas;
use super::ventanas::{class_of, hwnd_de, resolve, ventanas_de, visible, window_brief};
use crate::adapters::uia;
use crate::args::{flag, req_str};
use crate::error::{Error, Result};
use serde_json::{json, Value};

const DEFAULT_UIA_DEPTH: u32 = 4;
const DEFAULT_UIA_MAX_NODES: u32 = 150;
const STEP_VERIFY_TIMEOUT_MS: u64 = 5000;

pub(crate) fn aplanar<'a>(nodo: &'a Value, out: &mut Vec<&'a Value>) {
    for hijo in nodo["hijos"].as_array().into_iter().flatten() {
        out.push(hijo);
        aplanar(hijo, out);
    }
}

pub(crate) fn resumen_uia(n: &Value) -> Value {
    let sin_hijos = match n["hijos"].as_array() {
        Some(h) => h.is_empty(),
        None => true,
    };
    let rol = n["rol"].as_str().unwrap_or("");
    let no_vacio = n["rect"].as_array().is_some_and(|r| {
        r.len() == 4
            && r[2].as_f64().unwrap_or(0.0) > r[0].as_f64().unwrap_or(0.0)
            && r[3].as_f64().unwrap_or(0.0) > r[1].as_f64().unwrap_or(0.0)
    });
    let solo_vision = sin_hijos && matches!(rol, "pane" | "custom" | "image") && no_vacio;
    json!({
        "rol": n["rol"],
        "nombre": n["nombre"],
        "rect": n["rect"],
        "patrones": n["patrones"],
        "solo_vision": solo_vision,
    })
}

pub(crate) fn fusionar(ventanas_win: Vec<Value>, arbol: &Value) -> Value {
    let mut plano: Vec<&Value> = Vec::new();
    aplanar(arbol, &mut plano);
    let ventanas: Vec<Value> = ventanas_win
        .into_iter()
        .map(|mut v| {
            let par = plano
                .iter()
                .find(|n| n.get("hwnd").is_some() && n["hwnd"] == v["hwnd"]);
            if let Some(n) = par {
                v["uia"] = json!({
                    "nombre": n["nombre"],
                    "rol": n["rol"],
                    "patrones": n["patrones"],
                    "rect": n["rect"],
                });
            }
            v
        })
        .collect();
    let sin_hwnd: Vec<Value> = plano
        .iter()
        .copied()
        .filter(|n| n.get("hwnd").is_none())
        .map(resumen_uia)
        .collect();
    json!({ "ventanas": ventanas, "uia_sin_hwnd": sin_hwnd })
}

pub(crate) fn controles_con_uia(paths: &crate::config::Paths, args: &Value) -> Result<Value> {
    let mut r = super::ventanas::controles(paths, args)?;
    if flag(args, &["incluir_uia"]) {
        let h = hwnd_de(args, "ventana")?;
        let arbol = uia::arbol(&paths.estado, h, DEFAULT_UIA_DEPTH, DEFAULT_UIA_MAX_NODES)?;
        let mut plano = Vec::new();
        aplanar(&arbol, &mut plano);
        let uia_sin: Vec<Value> = plano
            .iter()
            .copied()
            .filter(|n| n.get("hwnd").is_none())
            .map(resumen_uia)
            .collect();
        r["uia"] = json!(uia_sin);
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

pub(crate) fn uia_invocar(args: &Value) -> Result<Value> {
    let h = hwnd_de(args, "ventana")?;
    let operacion = req_str(
        args,
        &["operacion"],
        "invocar|fijar_valor|alternar|seleccionar|expandir|contraer",
    )?;
    uia::invocar(
        h,
        args.get("nombre").and_then(Value::as_str),
        args.get("rol").and_then(Value::as_str),
        operacion,
        args.get("valor").and_then(Value::as_str),
    )
}

pub(crate) fn operacion_para(rol: Option<&str>) -> &'static str {
    match rol.unwrap_or("").to_ascii_lowercase().as_str() {
        "checkbox" => "alternar",
        "tab item" | "list item" => "seleccionar",
        "menu item" | "tree item" => "expandir",
        _ => "invocar",
    }
}

pub(crate) fn raiz_por_defecto(scoped: &[Hwnd]) -> Option<Hwnd> {
    scoped
        .iter()
        .copied()
        .find(|&h| visible(h) && class_of(h) == "THomeForm")
        .or_else(|| scoped.iter().copied().find(|&h| visible(h)))
}

pub(crate) struct Objetivo<'a> {
    pub(crate) nombre: Option<&'a str>,
    pub(crate) rol: Option<&'a str>,
    pub(crate) clase: Option<&'a str>,
    pub(crate) titulo: Option<&'a str>,
}

pub(crate) struct CascadaContext<'a> {
    pub(crate) paths: &'a crate::config::Paths,
    pub(crate) args: &'a Value,
    pub(crate) scoped: &'a [Hwnd],
    pub(crate) raiz: Option<Hwnd>,
}

pub(crate) fn cascada(ctx: &CascadaContext, obj: &Objetivo) -> Result<&'static str> {
    if let Some(h) = ctx.raiz {
        if obj.nombre.is_some() || obj.rol.is_some() {
            let op = operacion_para(obj.rol);
            if uia::invocar(h, obj.nombre, obj.rol, op, None).is_ok() {
                return Ok("uia");
            }
        }
    }
    if let Some(referencia) = obj.clase.or(obj.titulo) {
        if let Some(h) = resolve(ctx.scoped, referencia) {
            // SAFETY: `PostMessageW` encola el mensaje sin punteros; un HWND inválido solo hace fallar la llamada.
            unsafe { ffi::PostMessageW(h, BM_CLICK, 0, 0) };
            return Ok("win32");
        }
    }
    if let Some(h) = ctx.raiz {
        if escritorio_interactivo() {
            if let Ok((x, y)) = uia::centro(h, obj.nombre, obj.rol) {
                clic_fisico_en(x, y);
                return Ok("fisico");
            }
        }
    }
    if let Some(t) = ctx.args.get("teclas").and_then(Value::as_str) {
        let mut a = json!({ "teclas": t });
        if let Some(v) = ctx.args.get("ventana") {
            a["ventana"] = v.clone();
        }
        teclas(ctx.paths, &a)?;
        return Ok("teclas");
    }
    Err(Error::Gui(
        "No se pudo ejecutar el paso: objetivo no encontrado".into(),
    ))
}

pub(crate) fn paso(paths: &crate::config::Paths, args: &Value) -> Result<Value> {
    let objetivo = args
        .get("objetivo")
        .ok_or_else(|| crate::args::missing("objetivo", "nombre, rol, clase o titulo"))?;
    let campo = |k: &str| objetivo.get(k).and_then(Value::as_str);
    let obj = Objetivo {
        nombre: campo("nombre"),
        rol: campo("rol"),
        clase: campo("clase"),
        titulo: campo("titulo"),
    };
    if obj.nombre.is_none() && obj.rol.is_none() && obj.clase.is_none() && obj.titulo.is_none() {
        return Err(Error::Gui(
            "'objetivo' necesita nombre, rol, clase o titulo".into(),
        ));
    }
    let scoped = ventanas_de(args)?;
    let raiz = match args.get("ventana").and_then(Value::as_str) {
        Some(r) => {
            Some(resolve(&scoped, r).ok_or_else(|| Error::Gui(format!("Window '{r}' not found")))?)
        }
        None => raiz_por_defecto(&scoped),
    };
    let ctx = CascadaContext {
        paths,
        args,
        scoped: &scoped,
        raiz,
    };
    let via = cascada(&ctx, &obj)?;
    let verificado = match args.get("verificar") {
        Some(v) => {
            let params = EsperarParams {
                clase: v.get("clase").and_then(Value::as_str),
                titulo: v.get("titulo").and_then(Value::as_str),
                timeout_ms: STEP_VERIFY_TIMEOUT_MS,
            };
            esperar_ventana(args, &params)?;
            true
        }
        None => false,
    };
    Ok(json!({ "via": via, "objetivo": objetivo, "verificado": verificado }))
}

#[cfg(test)]
#[path = "../../../tests/unit/adapters/gui/uia_vista.rs"]
mod tests;
