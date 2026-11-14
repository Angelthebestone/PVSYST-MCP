//! UI Automation actions on a form: invoke a control by name and role, and the `paso`
//! cascade (UIA, Win32 message, physical click, keys) with a check afterwards.

use super::acciones::escritorio_interactivo;
use super::esperas::{esperar_ventana, EsperarParams};
use super::ffi::{self, Hwnd, BM_CLICK};
use super::teclado::teclas;
use super::ventanas::{class_of, hwnd_de, resolve, sin_ventana, ventanas_de, visible};
use crate::adapters::uia;
use crate::args::req_str;
use crate::error::{Error, Result};
use crate::idioma::{Idioma, Texto};
use serde_json::{json, Value};

/// Lets PVsyst finish the tab change (or its veto) before the strip is read back.
const PAUSA_PESTANA: std::time::Duration = std::time::Duration::from_millis(300);
const STEP_VERIFY_TIMEOUT_MS: u64 = 5000;

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
    ffi::exigir_cambio_de_pestana(indice, ffi::pagina_seleccionada(franja))
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

pub(crate) fn cascada(
    ctx: &CascadaContext,
    obj: &Objetivo,
    idioma: Idioma,
) -> Result<&'static str> {
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
        teclas(&Value::Object(a), idioma)?;
        return Ok("teclas");
    }
    let sin_objetivo = Texto {
        es: "No se pudo ejecutar el paso: objetivo no encontrado",
        en: "Could not run the step: target not found",
    };
    Err(Error::Gui(sin_objetivo.en_idioma(idioma).into()))
}

pub(crate) fn paso(args: &Value, idioma: Idioma) -> Result<Value> {
    if args.get("operacion").is_some() {
        let no_aplica = Texto {
            es: "'operacion' no aplica a 'paso': la cascada elige la acción; usa uia_invocar operacion:… o gui_actuar accion:campo/seleccionar",
            en: "'operacion' does not apply to 'paso': the cascade picks the action; use uia_invocar operacion:… or gui_actuar accion:campo/seleccionar",
        };
        return Err(Error::Invalid(no_aplica.en_idioma(idioma).into()));
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
        let sin_criterio = Texto {
            es: "'objetivo' necesita nombre, rol, clase o titulo",
            en: "'objetivo' needs nombre, rol, clase or titulo",
        };
        return Err(Error::Gui(sin_criterio.en_idioma(idioma).into()));
    }
    let scoped = ventanas_de(args)?;
    let raiz = match args.get("ventana").and_then(Value::as_str) {
        Some(r) => Some(resolve(&scoped, r).ok_or_else(|| Error::Gui(sin_ventana(r)))?),
        None => raiz_por_defecto(&scoped),
    };
    let ctx = CascadaContext {
        args,
        scoped: &scoped,
        raiz,
    };
    let via = cascada(&ctx, &obj, idioma)?;
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
