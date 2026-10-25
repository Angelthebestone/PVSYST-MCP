//! Result trimming by detail level and next-step recommendations for callers.

use serde_json::Value;

/// Accepted detail levels; `resumen` is the default.
pub(crate) const DETALLES: &[&str] = &["resumen", "normal", "completo"];

/// Keys whose contents are never trimmed.
const SIN_RECORTE: &[&str] = &[
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

/// Trims long arrays (adding `<clave>_total`) and long strings in place.
pub(crate) fn recortar(v: &mut Value, lim: (usize, usize)) {
    let (max_arr, max_str) = lim;
    match v {
        Value::Object(mapa) => {
            let mut totales = Vec::new();
            let mut ctx = ContextoRecorte {
                lim,
                totales: &mut totales,
            };
            for (k, hijo) in mapa.iter_mut() {
                recortar_campo_objeto(k, hijo, &mut ctx);
            }
            for (k, total) in totales {
                mapa.insert(k, Value::from(total));
            }
        }
        Value::Array(items) => {
            if items.len() > max_arr {
                items.truncate(max_arr);
            }
            for item in items.iter_mut() {
                recortar(item, lim);
            }
        }
        Value::String(s) => truncar_cadena(s, max_str),
        _ => {}
    }
}

/// Suggested next step after a successful call, as a short hint for the caller.
pub(crate) fn siguiente(tool: &str, valor: &str) -> Option<&'static str> {
    Some(match (tool, valor) {
        ("entorno", "diagnostico") => "disenar_sistema {latitud,longitud,kwp}",
        ("entorno", "preferencias") => "entorno preferencias {seccion:'Preferences'}",
        ("sitio", "crear") => "proyecto crear {plantilla,nombre_nuevo}",
        ("proyecto", "crear") => "proyecto dimensionar {proyecto,kwp}",
        ("proyecto", "dimensionar") => "proyecto doctor {proyecto}",
        ("proyecto", "doctor") => "simular {proyecto}",
        ("simular", "ejecutar") | ("disenar_sistema", _) => "resultados que:perdidas",
        ("resultados", "kpis") => "resultados que:comparar",
        ("editar", "campos") => "proyecto doctor",
        ("componente", "crear") | ("componente", "crear_desde_ficha") => {
            "proyecto crear {modulo|inversor}"
        }
        _ => return None,
    })
}

struct ContextoRecorte<'a> {
    lim: (usize, usize),
    totales: &'a mut Vec<(String, usize)>,
}

fn recortar_campo_objeto(k: &str, hijo: &mut Value, ctx: &mut ContextoRecorte<'_>) {
    if SIN_RECORTE.contains(&k) {
        return;
    }
    let (max_arr, max_str) = ctx.lim;
    match hijo {
        Value::Array(items) if items.len() > max_arr => {
            ctx.totales.push((format!("{k}_total"), items.len()));
            items.truncate(max_arr);
        }
        Value::String(s) => truncar_cadena(s, max_str),
        _ => recortar(hijo, ctx.lim),
    }
}

fn truncar_cadena(s: &mut String, max_str: usize) {
    if s.chars().count() > max_str {
        let corto: String = s.chars().take(max_str).collect();
        *s = format!("{corto}…");
    }
}
