//! UIA tree building, traversal and caching.

use super::{cadena, error, rol_de, Sesion};
use crate::error::Result;
use crate::state::Estado;
use serde_json::{json, Value};
use std::ffi::c_void;
use std::time::{Duration, Instant};
use windows::core::Interface;
use windows::Win32::Foundation::HWND;
use windows::Win32::UI::Accessibility::{
    IUIAutomationElement, IUIAutomationTreeWalker, UIA_ExpandCollapsePatternId,
    UIA_InvokePatternId, UIA_SelectionItemPatternId, UIA_TogglePatternId, UIA_ValuePatternId,
    UIA_PATTERN_ID,
};

/// Lifetime of a cached tree snapshot.
const CACHE_TTL: Duration = Duration::from_secs(2);

/// The tree walkers signal "no more children/siblings" with a null interface
/// pointer, which windows-rs surfaces as `Error::empty()` (code S_OK). Any
/// negative HRESULT is a real COM failure.
fn es_fin_de_lista(e: &windows::core::Error) -> bool {
    e.code().is_ok()
}

/// Direct children of `el` in control view, with the cache request applied.
/// The error is `Some` when COM failed mid-enumeration: the returned list is
/// then a prefix and callers must treat the snapshot as incomplete.
pub(crate) fn hijos(
    sesion: &Sesion,
    walker: &IUIAutomationTreeWalker,
    el: &IUIAutomationElement,
) -> (Vec<IUIAutomationElement>, Option<windows::core::Error>) {
    let mut out = Vec::new();
    // SAFETY: Llamada COM sobre una interfaz viva en el hilo con COM inicializado.
    let mut c = match unsafe { walker.GetFirstChildElementBuildCache(el, &sesion.req) } {
        Ok(c) => c,
        Err(e) if es_fin_de_lista(&e) => return (out, None),
        Err(e) => return (out, Some(e)),
    };
    loop {
        out.push(c.clone());
        // SAFETY: Llamada COM sobre una interfaz viva en el hilo con COM inicializado.
        match unsafe { walker.GetNextSiblingElementBuildCache(&c, &sesion.req) } {
            Ok(n) => c = n,
            Err(e) if es_fin_de_lista(&e) => break,
            Err(e) => return (out, Some(e)),
        }
    }
    (out, None)
}

/// Traversal state bundling session, walker, remaining node budget and the
/// facts that make a snapshot incomplete.
pub(crate) struct RecorridoContext<'a> {
    pub(crate) sesion: &'a Sesion,
    pub(crate) walker: &'a IUIAutomationTreeWalker,
    pub(crate) budget: &'a mut u32,
    /// The snapshot is incomplete: the node budget ran out or COM failed while
    /// enumerating children.
    pub(crate) truncado: bool,
    /// Nodes whose properties could not all be read.
    pub(crate) nodos_con_error: u32,
}

struct PropiedadesNodo {
    nombre: Value,
    rol: Value,
    automation_id: Value,
    clase: Value,
    rect: Value,
    hwnd: isize,
    errores: Vec<String>,
}

fn leer_str_prop<F>(f: F, campo: &str, errores: &mut Vec<String>) -> Value
where
    F: FnOnce() -> windows::core::Result<windows::core::BSTR>,
{
    match cadena(f()) {
        Ok(s) => Value::String(s),
        Err(e) => {
            errores.push(format!("{campo}: {e}"));
            Value::Null
        }
    }
}

fn leer_propiedades(el: &IUIAutomationElement) -> PropiedadesNodo {
    let mut errores = Vec::new();

    // SAFETY: Llamada COM sobre interfaz viva con COM inicializado.
    let nombre = leer_str_prop(|| unsafe { el.CachedName() }, "nombre", &mut errores);

    // SAFETY: Llamada COM sobre interfaz viva con COM inicializado.
    let rol = match unsafe { el.CachedControlType() } {
        Ok(t) => Value::String(rol_de(t.0).to_string()),
        Err(e) => {
            errores.push(format!("rol: {e}"));
            Value::Null
        }
    };

    // SAFETY: Llamada COM sobre interfaz viva con COM inicializado.
    let automation_id = leer_str_prop(
        || unsafe { el.CachedAutomationId() },
        "automation_id",
        &mut errores,
    );

    // SAFETY: Llamada COM sobre interfaz viva con COM inicializado.
    let clase = leer_str_prop(|| unsafe { el.CachedClassName() }, "clase", &mut errores);

    // SAFETY: Llamada COM sobre interfaz viva con COM inicializado.
    let rect = match unsafe { el.CachedBoundingRectangle() } {
        Ok(r) => json!([r.left, r.top, r.right, r.bottom]),
        Err(e) => {
            errores.push(format!("rect: {e}"));
            Value::Null
        }
    };

    // SAFETY: Llamada COM sobre interfaz viva con COM inicializado.
    let hwnd = match unsafe { el.CachedNativeWindowHandle() } {
        Ok(h) => h.0 as isize,
        Err(e) => {
            errores.push(format!("hwnd: {e}"));
            0
        }
    };

    PropiedadesNodo {
        nombre,
        rol,
        automation_id,
        clase,
        rect,
        hwnd,
        errores,
    }
}

fn leer_patrones(el: &IUIAutomationElement) -> Vec<&'static str> {
    let mut patrones = Vec::new();
    let nombres_patron: [(UIA_PATTERN_ID, &str); 5] = [
        (UIA_InvokePatternId, "invoke"),
        (UIA_ValuePatternId, "valor"),
        (UIA_TogglePatternId, "alternar"),
        (UIA_SelectionItemPatternId, "seleccion"),
        (UIA_ExpandCollapsePatternId, "expandir"),
    ];
    for (id, etiqueta) in nombres_patron {
        // SAFETY: Llamada COM sobre interfaz viva; verifica puntero no nulo.
        if unsafe { el.GetCachedPattern(id) }.is_ok_and(|p| !p.as_raw().is_null()) {
            patrones.push(etiqueta);
        }
    }
    patrones
}

/// Serialises `el` and up to `prof` levels of children, decrementing `budget`
/// per node visited. A property read or child enumeration failure is recorded
/// in an `error` field (and an enumeration failure also marks the snapshot as
/// `truncado`) so that an incomplete tree is clearly identifiable.
pub(crate) fn nodo(ctx: &mut RecorridoContext, el: &IUIAutomationElement, prof: u32) -> Value {
    *ctx.budget = ctx.budget.saturating_sub(1);
    let mut hijos_vec = Vec::new();
    let mut error_enum: Option<String> = None;
    if prof > 0 {
        let (children, error) = hijos(ctx.sesion, ctx.walker, el);
        if let Some(e) = error {
            // COM falló a mitad de la enumeración: la lista es un prefijo.
            ctx.truncado = true;
            error_enum = Some(format!("hijos: {e}"));
        }
        for c in children {
            if *ctx.budget == 0 {
                ctx.truncado = true;
                break;
            }
            hijos_vec.push(nodo(ctx, &c, prof - 1));
        }
    }

    let props = leer_propiedades(el);
    let patrones = leer_patrones(el);

    let mut n = json!({
        "nombre": props.nombre,
        "rol": props.rol,
        "automation_id": props.automation_id,
        "clase": props.clase,
        "rect": props.rect,
        "patrones": patrones,
        "hijos": hijos_vec,
    });
    if props.hwnd != 0 {
        n["hwnd"] = json!(format!("0x{:X}", props.hwnd));
    }
    let mut errores = props.errores;
    if let Some(msg) = error_enum {
        errores.push(msg);
    }
    if !errores.is_empty() {
        ctx.nodos_con_error += 1;
        n["error"] = json!(errores);
    }
    n
}

/// Snapshot of the element tree under `hwnd`, `profundidad` levels deep and at
/// most `max_nodos` nodes. Results are cached for two seconds per argument set.
pub(crate) fn arbol(
    estado: &Estado,
    hwnd: isize,
    profundidad: u32,
    max_nodos: u32,
) -> Result<Value> {
    let clave = (hwnd, profundidad, max_nodos);
    let cache = &estado.arboles;
    if let Ok(mapa) = cache.lock() {
        if let Some((_, valor)) = mapa
            .get(&clave)
            .filter(|(cuando, _)| cuando.elapsed() < CACHE_TTL)
        {
            return Ok(valor.clone());
        }
    }
    let sesion = Sesion::nueva()?;
    // SAFETY: Llamada COM sobre una interfaz viva con COM inicializado.
    let raiz = unsafe {
        sesion
            .auto
            .ElementFromHandleBuildCache(HWND(hwnd as *mut c_void), &sesion.req)
    }
    .map_err(error)?;
    // SAFETY: Llamada COM sobre una interfaz viva con COM inicializado.
    let walker = unsafe { sesion.auto.ControlViewWalker() }.map_err(error)?;
    let mut presupuesto = max_nodos;
    let mut ctx = RecorridoContext {
        sesion: &sesion,
        walker: &walker,
        budget: &mut presupuesto,
        truncado: false,
        nodos_con_error: 0,
    };
    let mut valor = nodo(&mut ctx, &raiz, profundidad);
    // A partial snapshot must say so at the root, not only in some leaves.
    valor["truncado"] = json!(ctx.truncado);
    valor["nodos_con_error"] = json!(ctx.nodos_con_error);
    if let Ok(mut mapa) = cache.lock() {
        mapa.insert(clave, (Instant::now(), valor.clone()));
    }
    Ok(valor)
}
