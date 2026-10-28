//! Result trimming by detail level and next-step recommendations for callers.

use crate::vista_agente::contrato::catalogo;
use serde_json::Value;

/// Accepted detail levels; `resumen` is the default.
pub(crate) const DETALLES: &[&str] = &["resumen", "normal", "completo"];

/// Keys whose contents are never trimmed.
pub(super) const SIN_RECORTE: &[&str] = &[
    "text",
    "contenido",
    "base64",
    "stdout_tail",
    "_imagen_png_base64",
    "horas",
    // A form's fields and buttons are the answer, not a list to summarise.
    "campos",
    "botones",
    "pestanas",
    "textos",
    "sin_mapear",
    "resultados",
    "recalculados",
    // The window coverage of `gui_observar que:esquema` is a registry, not a list to summarise.
    "ventanas",
];

const LIMITE_ARRAY_NORMAL: usize = 50;
const LIMITE_ARRAY_RESUMEN: usize = 13; // 12 calendar months + full year
const LIMITE_STR_NORMAL: usize = 8000;
const LIMITE_STR_RESUMEN: usize = 1500;

/// Size limits (max array items, max string chars) for `detalle`; `None` keeps everything.
pub(crate) fn limites(args: &Value) -> Option<(usize, usize)> {
    match args
        .get("detalle")
        .and_then(Value::as_str)
        .unwrap_or("resumen")
    {
        "completo" => None,
        "normal" => Some((LIMITE_ARRAY_NORMAL, LIMITE_STR_NORMAL)),
        _ => Some((LIMITE_ARRAY_RESUMEN, LIMITE_STR_RESUMEN)),
    }
}

/// Trims long arrays (adding `<clave>_total`) and long strings in place; returns whether anything was cut.
pub(crate) fn recortar(v: &mut Value, lim: (usize, usize)) -> bool {
    let (max_arr, max_str) = lim;
    match v {
        Value::Object(mapa) => {
            let mut totales = Vec::new();
            let mut ctx = ContextoRecorte {
                lim,
                totales: &mut totales,
            };
            let mut recortado = false;
            for (k, hijo) in mapa.iter_mut() {
                recortado |= recortar_campo_objeto(k, hijo, &mut ctx);
            }
            recortado |= !totales.is_empty();
            // A `<clave>_total` the answer already carries is the domain's count: keep it.
            for (k, total) in totales {
                mapa.entry(k).or_insert(Value::from(total));
            }
            recortado
        }
        Value::Array(items) => {
            let mut recortado = items.len() > max_arr;
            items.truncate(max_arr);
            for item in items.iter_mut() {
                recortado |= recortar(item, lim);
            }
            recortado
        }
        Value::String(s) => truncar_cadena(s, max_str),
        _ => false,
    }
}

/// Suggested next step after a successful call, as a short hint for the caller.
pub(crate) fn siguiente(tool: &str, valor: &str) -> Option<&'static str> {
    catalogo::buscar(tool, valor)?.pista
}

struct ContextoRecorte<'a> {
    lim: (usize, usize),
    totales: &'a mut Vec<(String, usize)>,
}

fn recortar_campo_objeto(k: &str, hijo: &mut Value, ctx: &mut ContextoRecorte<'_>) -> bool {
    if SIN_RECORTE.contains(&k) {
        return false;
    }
    let (max_arr, max_str) = ctx.lim;
    match hijo {
        Value::Array(items) if items.len() > max_arr => {
            ctx.totales.push((format!("{k}_total"), items.len()));
            items.truncate(max_arr);
            true
        }
        Value::String(s) => truncar_cadena(s, max_str),
        _ => recortar(hijo, ctx.lim),
    }
}

fn truncar_cadena(s: &mut String, max_str: usize) -> bool {
    let excede = s.chars().count() > max_str;
    if excede {
        let corto: String = s.chars().take(max_str).collect();
        *s = format!("{corto}…");
    }
    excede
}

/// Marks a trimmed response and tells the caller how to fetch everything.
const HINT_DETALLE: &str = "detalle:completo";

/// What the response decorator needs to know about a finished call.
pub(crate) struct Respuesta<'a> {
    pub(crate) herramienta: &'a str,
    pub(crate) accion: &'a str,
    pub(crate) ms: u64,
    pub(crate) ignorados: Vec<String>,
}

/// `todos:true` on `listar objetos` keeps every object, as `detalle:completo` does.
fn pide_todos(r: &Respuesta<'_>, args: &Value) -> bool {
    r.herramienta == "listar"
        && r.accion == "objetos"
        && args.get("todos").and_then(Value::as_bool) == Some(true)
}

/// Trims the result to `detalle` and adds `elapsed_ms`, `siguiente`, `ignorados`, and, when cut, `truncado`, `hint` and `total`.
pub(crate) fn decorar(v: &mut Value, args: &Value, r: Respuesta<'_>) {
    let lim = if pide_todos(&r, args) {
        None
    } else {
        limites(args)
    };
    let total = lim.and_then(|l| total_principal(v, l));
    let truncado = lim.is_some_and(|l| recortar(v, l));
    let Some(obj) = v.as_object_mut() else {
        return;
    };
    obj.insert("elapsed_ms".to_string(), Value::from(r.ms));
    if let Some(sig) = siguiente(r.herramienta, r.accion) {
        obj.insert("siguiente".to_string(), Value::from(sig));
    }
    if !r.ignorados.is_empty() {
        obj.insert("ignorados".to_string(), Value::from(r.ignorados));
    }
    if truncado {
        obj.insert("truncado".to_string(), Value::Bool(true));
        obj.insert("hint".to_string(), Value::from(HINT_DETALLE));
    }
    if let Some(n) = total {
        obj.entry("total").or_insert(Value::from(n));
    }
}

/// Original length of the longest top-level list that the detail level trims.
fn total_principal(v: &Value, (max_arr, _): (usize, usize)) -> Option<usize> {
    v.as_object()?
        .iter()
        .filter(|(k, _)| !SIN_RECORTE.contains(&k.as_str()))
        .filter_map(|(_, hijo)| hijo.as_array().map(Vec::len))
        .filter(|n| *n > max_arr)
        .max()
}
