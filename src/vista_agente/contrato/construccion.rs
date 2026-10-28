//! Builds the `vista` block of a finished call from its request and its answer.

use super::campos::campos_de;
use super::catalogo::{self, Entrada, Paso};
use super::pendiente::{pendiente_de, Pendiente};
use super::tipos::{ErrorVista, Estado, Origen, PasoSiguiente, Recorte, Vista};
use serde_json::{Map, Value};
use std::collections::HashSet;

/// What the block needs to know about the call that produced the answer.
pub(crate) struct Contexto<'a> {
    pub(crate) tool: &'a str,
    pub(crate) accion: &'a str,
    pub(crate) args: &'a Value,
}

/// Tools whose answers describe the workspace or the MCP, not a PVsyst figure.
const TOOLS_DEL_MCP: &[&str] = &["entorno", "workspace", "listar"];
/// Value of `origen` that marks an approximation computed by the MCP.
const ORIGEN_APROXIMACION: &str = "aproximacion_mcp";
/// Keys that carry warnings, as strings or lists of strings.
const CLAVES_AVISO: &[&str] = &[
    "avisos",
    "advertencias",
    "warnings",
    "aviso",
    "aviso_origen",
];
const MAX_CARACTERES_MOTIVO: usize = 300;
/// Warnings listed in `vista`; the answer keeps all of them in full.
const MAX_AVISOS: usize = 5;
const MAX_CARACTERES_AVISO: usize = 160;
const PEDIR_RESTO: &str = "detalle:completo";
const PEDIR_AVISOS: &str = "leer avisos completos en la propia respuesta";
const SUFIJO_TOTAL: &str = "_total";

fn origen_de(ctx: &Contexto<'_>, salida: &Value) -> Origen {
    if salida.get("origen").and_then(Value::as_str) == Some(ORIGEN_APROXIMACION) {
        return Origen::AproximacionMcp;
    }
    if TOOLS_DEL_MCP.contains(&ctx.tool) {
        return Origen::Mcp;
    }
    Origen::Pvsyst
}

fn textos_de(salida: &Value, clave: &str) -> Vec<String> {
    match salida.get(clave) {
        Some(Value::String(s)) => vec![s.clone()],
        Some(Value::Array(items)) => items
            .iter()
            .filter_map(Value::as_str)
            .map(str::to_string)
            .collect(),
        _ => Vec::new(),
    }
}

fn avisos_de(salida: &Value) -> Vec<String> {
    let mut avisos: Vec<String> = CLAVES_AVISO
        .iter()
        .flat_map(|clave| textos_de(salida, clave))
        .collect();
    let mut vistos = HashSet::new();
    avisos.retain(|a| vistos.insert(a.clone()));
    avisos
        .into_iter()
        .map(|a| a.chars().take(MAX_CARACTERES_AVISO).collect())
        .collect()
}

/// The first warnings, and the cut when there are more: the answer keeps all of them.
fn acotar_avisos(mut avisos: Vec<String>) -> (Vec<String>, Option<Recorte>) {
    let total = avisos.len();
    avisos.truncate(MAX_AVISOS);
    let recorte = (total > MAX_AVISOS).then(|| Recorte {
        clave: "vista.avisos".into(),
        mostrados: MAX_AVISOS,
        total,
        pedir_resto: PEDIR_AVISOS,
    });
    (avisos, recorte)
}

fn algun_paso_ok(salida: &Value) -> bool {
    salida
        .get("pasos")
        .and_then(Value::as_array)
        .is_some_and(|pasos| {
            pasos
                .iter()
                .any(|p| p.get("ok").and_then(Value::as_bool) == Some(true))
        })
}

/// `ok:false` or a non-empty `errores` list marks a failure inside a successful reply;
/// it is partial when a step worked or the reply never said `ok:false`.
fn estado_de(salida: &Value) -> Estado {
    let fallo = salida.get("ok").and_then(Value::as_bool) == Some(false);
    let con_errores = !textos_de(salida, "errores").is_empty();
    if !fallo && !con_errores {
        return Estado::Ok;
    }
    if algun_paso_ok(salida) || !fallo {
        return Estado::Parcial;
    }
    Estado::Error
}

fn motivo_de(salida: &Value) -> String {
    let errores = textos_de(salida, "errores");
    let motivo = if errores.is_empty() {
        "la acción informó ok:false".to_string()
    } else {
        errores.join("; ")
    };
    motivo.chars().take(MAX_CARACTERES_MOTIVO).collect()
}

fn error_de(estado: Estado, salida: &Value) -> Option<ErrorVista> {
    (estado != Estado::Ok).then(|| ErrorVista {
        codigo: "fallo_parcial".into(),
        motivo: motivo_de(salida),
        causa_probable: "Un paso del flujo falló; los pasos anteriores sí se aplicaron.",
        que_hacer: "Corrige la causa indicada en motivo y avisos y repite la acción.",
        claves_validas: Vec::new(),
    })
}

fn resumen_de(entrada: Option<&Entrada>, estado: Estado, salida: &Value) -> String {
    let hecho = entrada.map_or("Acción completada.", |e| e.resumen);
    match estado {
        Estado::Ok => hecho.to_string(),
        _ => format!("No se completó del todo: {}.", motivo_de(salida)),
    }
}

/// Lists the answer cut by the detail level, from the `<clave>_total` the trimmer adds.
fn recortes_de(salida: &Value) -> Vec<Recorte> {
    let Some(mapa) = salida.as_object() else {
        return Vec::new();
    };
    mapa.iter()
        .filter_map(|(k, total)| Some((k.strip_suffix(SUFIJO_TOTAL)?, total.as_u64()?)))
        .filter_map(|(base, total)| {
            let mostrados = mapa.get(base)?.as_array()?.len();
            Some(Recorte {
                clave: base.to_string(),
                mostrados,
                total: usize::try_from(total).ok()?,
                pedir_resto: PEDIR_RESTO,
            })
        })
        .collect()
}

fn texto_de<'a>(fuente: &'a Value, claves: &[&str]) -> Option<&'a str> {
    claves
        .iter()
        .find_map(|k| fuente.get(k).and_then(Value::as_str))
}

/// Arguments of `paso`, or `None` when a value it must take is in neither the request nor the answer.
fn argumentos_de(paso: &Paso, ctx: &Contexto<'_>, salida: &Value) -> Option<Value> {
    let mut argumentos: Map<String, Value> = paso
        .fijos
        .iter()
        .map(|(k, v)| ((*k).to_string(), Value::from(*v)))
        .collect();
    for (destino, candidatos) in paso.tomar {
        let texto = texto_de(ctx.args, candidatos).or_else(|| texto_de(salida, candidatos))?;
        argumentos.insert((*destino).to_string(), Value::from(texto));
    }
    if let Some(workspace) = texto_de(ctx.args, &["workspace"]) {
        argumentos.insert("workspace".into(), Value::from(workspace));
    }
    Some(Value::Object(argumentos))
}

fn paso_siguiente(paso: &Paso, ctx: &Contexto<'_>, salida: &Value) -> Option<PasoSiguiente> {
    Some(PasoSiguiente {
        tool: paso.tool.to_string(),
        argumentos: argumentos_de(paso, ctx, salida)?,
        motivo: paso.motivo,
    })
}

/// Next calls of the entry; the same call completed when the answer is pending;
/// the diagnostic call when the action did not finish.
pub(crate) fn siguiente_de(
    ctx: &Contexto<'_>,
    estado: Estado,
    salida: &Value,
) -> Vec<PasoSiguiente> {
    if let Some(pendiente) = pendiente_de(ctx, salida) {
        return vec![pendiente.siguiente(ctx)];
    }
    let pasos: &[Paso] = match (estado, catalogo::buscar(ctx.tool, ctx.accion)) {
        (Estado::Ok, Some(entrada)) => entrada.siguiente,
        _ => std::slice::from_ref(&catalogo::DIAGNOSTICO),
    };
    pasos
        .iter()
        .filter_map(|p| paso_siguiente(p, ctx, salida))
        .collect()
}

/// The `vista` block for an answer that arrived.
pub(crate) fn construir(ctx: &Contexto<'_>, salida: &Value) -> Vista {
    let pendiente = pendiente_de(ctx, salida);
    let estado = pendiente.map_or_else(|| estado_de(salida), Pendiente::estado);
    let (campos, recorte_campos) = campos_de(salida);
    let mut recortado = recortes_de(salida);
    recortado.extend(recorte_campos);
    let (avisos, recorte_avisos) = acotar_avisos(avisos_de(salida));
    recortado.extend(recorte_avisos);
    let hecho = catalogo::buscar(ctx.tool, ctx.accion);
    Vista {
        estado,
        resumen: match pendiente {
            Some(p) => p.resumen(hecho.map_or("la acción pedida.", |e| e.resumen), salida),
            None => resumen_de(hecho, estado, salida),
        },
        origen: origen_de(ctx, salida),
        campos,
        avisos,
        siguiente: siguiente_de(ctx, estado, salida),
        recortado,
        error: match pendiente {
            Some(p) => p.error(salida),
            None => error_de(estado, salida),
        },
    }
}
