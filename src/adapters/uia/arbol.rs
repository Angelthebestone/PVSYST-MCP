//! UIA tree building, traversal and caching.

use super::{com, rol_de, Sesion};
use crate::error::Result;
use crate::state::Estado;
use serde_json::{json, Value};
use std::time::{Duration, Instant};
use windows::Win32::UI::Accessibility::{IUIAutomationElement, IUIAutomationTreeWalker};

/// Lifetime of a cached tree snapshot.
const CACHE_TTL: Duration = Duration::from_secs(2);

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

pub(crate) fn leer_str_prop(
    valor: windows::core::Result<String>,
    campo: &str,
    errores: &mut Vec<String>,
) -> Value {
    match valor {
        Ok(s) => Value::String(s),
        Err(e) => {
            errores.push(format!("{campo}: {e}"));
            Value::Null
        }
    }
}

fn leer_propiedades(el: &IUIAutomationElement) -> PropiedadesNodo {
    let mut errores = Vec::new();
    let p = com::propiedades(el);

    let nombre = leer_str_prop(p.nombre, "nombre", &mut errores);
    let rol = match p.tipo {
        Ok(t) => Value::String(rol_de(t).to_string()),
        Err(e) => {
            errores.push(format!("rol: {e}"));
            Value::Null
        }
    };
    let automation_id = leer_str_prop(p.automation_id, "automation_id", &mut errores);
    let clase = leer_str_prop(p.clase, "clase", &mut errores);
    let rect = match p.rect {
        Ok(r) => json!([r.left, r.top, r.right, r.bottom]),
        Err(e) => {
            errores.push(format!("rect: {e}"));
            Value::Null
        }
    };
    let hwnd = match p.hwnd {
        Ok(h) => h,
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

/// Serialises `el` and up to `prof` levels of children, decrementing `budget`
/// per node visited. A property read or child enumeration failure is recorded
/// in an `error` field (and an enumeration failure also marks the snapshot as
/// `truncado`) so that an incomplete tree is clearly identifiable.
pub(crate) fn nodo(ctx: &mut RecorridoContext, el: &IUIAutomationElement, prof: u32) -> Value {
    *ctx.budget = ctx.budget.saturating_sub(1);
    let mut hijos_vec = Vec::new();
    let mut error_enum: Option<String> = None;
    if prof > 0 {
        let (children, error) = com::hijos(ctx.sesion, ctx.walker, el);
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
    let patrones = com::patrones_soportados(el);

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
    let (raiz, walker) = com::raiz_y_walker(&sesion, hwnd)?;
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
