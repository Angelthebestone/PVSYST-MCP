//! Hybrid GUI view fusing Win32 window handles with the UI Automation tree.

use super::acciones::escritorio_interactivo;
use super::esperas::{esperar_ventana, EsperarParams};
use super::ffi::{self, enum_children, Hwnd, BM_CLICK};
use super::teclado::teclas;
use super::ventanas::{class_of, hwnd_de, resolve, ventanas_de, visible, window_brief};
use crate::adapters::uia;
use crate::args::{flag, req_str};
use crate::error::{Error, Result};
use serde_json::{json, Value};
/// Lets PVsyst finish the tab change (or its veto) before the strip is read back.
const PAUSA_PESTANA: std::time::Duration = std::time::Duration::from_millis(300);

const DEFAULT_UIA_DEPTH: u32 = 4;
const DEFAULT_UIA_MAX_NODES: u32 = 150;
const STEP_VERIFY_TIMEOUT_MS: u64 = 5000;

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
    let mut r = super::ventanas::controles(paths, args)?;
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

pub(crate) fn uia_invocar(args: &Value) -> Result<Value> {
    let h = hwnd_de(args, "ventana")?;
    let operacion = req_str(
        args,
        &["operacion"],
        "invocar|fijar_valor|alternar|seleccionar|expandir|contraer",
    )?;
    let nombre = args.get("nombre").and_then(Value::as_str);
    let resultado = uia::invocar(
        h,
        nombre,
        args.get("rol").and_then(Value::as_str),
        operacion,
        args.get("valor").and_then(Value::as_str),
    )?;
    if let (Some(n), "seleccionar", Some("tab item")) =
        (nombre, operacion, args.get("rol").and_then(Value::as_str))
    {
        exigir_pestana_cambiada(h, n)?;
    }
    Ok(resultado)
}

/// PVsyst may veto a tab change while the current page has invalid fields, and
/// UIA still reports the tab as selected: compare the strip's real selection.
fn exigir_pestana_cambiada(h: Hwnd, nombre: &str) -> Result<()> {
    let Some((indice, franja)) = uia::posicion_pestana(h, nombre)? else {
        return Ok(());
    };
    std::thread::sleep(PAUSA_PESTANA);
    match ffi::pagina_seleccionada(franja) {
        Some(actual) if actual != indice => Err(Error::Gui(format!(
            "PVsyst no cambió a la pestaña {indice}: sigue en la {actual}. El formulario vetó el cambio (validación de los campos de la pestaña actual); el motivo está en gui_observar que:vista ocr:true (textos)"
        ))),
        _ => Ok(()),
    }
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

fn intentar_uia(raiz: Hwnd, obj: &Objetivo) -> bool {
    if obj.nombre.is_none() && obj.rol.is_none() {
        return false;
    }
    let op = operacion_para(obj.rol);
    uia::invocar(raiz, obj.nombre, obj.rol, op, None).is_ok()
}

fn intentar_fisico(raiz: Hwnd, obj: &Objetivo) -> bool {
    if !escritorio_interactivo() {
        return false;
    }
    let Ok((x, y)) = uia::centro(raiz, obj.nombre, obj.rol) else {
        return false;
    };
    ffi::clic_fisico_en(x, y);
    true
}

pub(crate) fn cascada(ctx: &CascadaContext, obj: &Objetivo) -> Result<&'static str> {
    if let Some(h) = ctx.raiz {
        if intentar_uia(h, obj) {
            return Ok("uia");
        }
    }
    if let Some(referencia) = obj.clase.or(obj.titulo) {
        if let Some(h) = resolve(ctx.scoped, referencia) {
            ffi::post_message(h, ffi::Mensaje::simple(BM_CLICK));
            return Ok("win32");
        }
    }
    if let Some(h) = ctx.raiz {
        if intentar_fisico(h, obj) {
            return Ok("fisico");
        }
    }
    if let Some(t) = ctx.args.get("teclas").and_then(Value::as_str) {
        let mut a = serde_json::Map::new();
        a.insert("teclas".into(), json!(t));
        if let Some(v) = ctx.args.get("ventana") {
            a.insert("ventana".into(), v.clone());
        }
        teclas(ctx.paths, &Value::Object(a))?;
        return Ok("teclas");
    }
    Err(Error::Gui(
        "No se pudo ejecutar el paso: objetivo no encontrado".into(),
    ))
}

pub(crate) fn paso(paths: &crate::config::Paths, args: &Value) -> Result<Value> {
    if args.get("operacion").is_some() {
        return Err(Error::Invalid(
            "'operacion' no aplica a 'paso': la cascada elige la acción; use uia_invocar operacion:… o gui_actuar accion:campo/seleccionar".into(),
        ));
    }
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
