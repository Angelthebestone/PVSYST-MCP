//! Runs catalogue entries against the live PVsyst: finds the host window,
//! triggers the menu item or button, and reports the windows that appear.

use super::dominio::{buscar, pedido_de_menu, Acceso, Efecto, Funcion, PedidoMenu};
use super::entradas::funciones;
use super::espera::{esperar_nuevas, Resultado, SolicitudEspera};
use crate::adapters::gui::acciones::clic_por_mensajes;
use crate::adapters::gui::conduccion::Formulario;
use crate::adapters::gui::ffi::{self, Hwnd, WM_COMMAND};
use crate::adapters::gui::menu_por_posicion::{
    contar_menus, id_en_posicion, id_en_submenu, Posicion,
};
use crate::adapters::gui::modal;
use crate::adapters::gui::resumen::resumen_de_ventana;
use crate::adapters::gui::ventanas::{
    class_of, hijo_visible_con_texto, hwnd_de, text_of, ventanas_de, visible,
};
use crate::args::req_str;
use crate::error::{Error, Result};
use crate::idioma::{Idioma, Texto};
use serde_json::{json, Value};
use std::time::{Duration, Instant};

const SONDEO: Duration = Duration::from_millis(100);
const ESPERA_POR_DEFECTO_MS: u64 = 5_000;
/// A form that has just opened keeps computing (the system form needs several
/// seconds); a click sent meanwhile is lost without an error. Measured: 3.5 s
/// to appear, a click right after it opens nothing, one 5 s later does.
const ASENTAMIENTO_DEL_ANFITRION: Duration = Duration::from_secs(5);
/// Longest wait a caller may ask for with `timeout_ms`.
const ESPERA_MAXIMA_MS: u64 = 60_000;

/// `gui_observar que:funciones`: the catalogue, or the keys under `funcion`.
pub(crate) fn listar(args: &Value) -> Value {
    let base: Vec<&Funcion> = match args.get("funcion").and_then(Value::as_str) {
        Some(clave) => buscar(funciones(), clave),
        None => funciones().collect(),
    };
    let ventana = args.get("ventana").and_then(Value::as_str);
    let lista: Vec<Value> = base
        .into_iter()
        .filter(|f| ventana.is_none_or(|v| f.ventana.eq_ignore_ascii_case(v)))
        .map(Funcion::a_json)
        .collect();
    json!({ "count": lista.len(), "funciones": lista })
}

/// The request the catalogue serves: its arguments and the client's language.
#[derive(Clone, Copy)]
struct Pedido<'a> {
    args: &'a Value,
    idioma: Idioma,
}

/// `gui_actuar accion:funcion`: opens one catalogued function, first opening
/// the window that hosts its button when that window is closed.
pub(crate) fn abrir(args: &Value) -> Result<Value> {
    // `projects::gui` and the scene steps open catalogue entries without a
    // `Paths`: the environment is the same source `Paths` reads.
    let pedido = Pedido {
        args,
        idioma: Idioma::de_entorno(),
    };
    let clave_del_catalogo = Texto {
        es: "clave del catálogo (gui_observar que:funciones)",
        en: "catalogue key (gui_observar que:funciones)",
    };
    let clave = req_str(
        args,
        &["funcion"],
        clave_del_catalogo.en_idioma(pedido.idioma),
    )?;
    abrir_funcion(pedido, funcion_unica(clave, pedido.idioma)?)
}

fn abrir_funcion(pedido: Pedido, funcion: &Funcion) -> Result<Value> {
    let anfitrion = anfitrion_de(pedido, funcion)?;
    if let Some(pagina) = funcion.pagina {
        Formulario::de(anfitrion).pagina(usize::from(pagina))?;
    }
    let antes = visibles(pedido.args)?;
    disparar(pedido, anfitrion, funcion)?;
    // Nothing is left to watch once PVsyst itself is closing.
    let visto = match funcion.efecto {
        Efecto::Cierra => Resultado::vacio(),
        _ => esperar_nuevas(
            SolicitudEspera {
                antes: &antes,
                esperada: funcion.abre,
                limite: espera_pedida(pedido.args),
            },
            || visibles(pedido.args),
        )?,
    };
    let efecto_json = funcion.efecto.nombre();
    let ausente = match funcion.efecto {
        Efecto::Cierra => None,
        _ => visto.clase_ausente(funcion.abre),
    };
    let mut respuesta = json!({
        "funcion": funcion.clave,
        "etiqueta": funcion.etiqueta,
        "efecto": efecto_json,
        "ventanas_nuevas": visto.nuevas_json(),
        "ya_abiertas": visto.ya_abiertas_json(),
        "nota": funcion.nota,
        "resumen": visto.primera_nueva().map(resumen_de_ventana),
    });
    if let (Some(clase), Some(o)) = (ausente, respuesta.as_object_mut()) {
        let no_abrio = Texto {
            es: "PVsyst no abrió la ventana esperada ({clase}) en el plazo de espera",
            en: "PVsyst did not open the expected window ({clase}) within the wait",
        };
        o.insert("completo".into(), json!(false));
        o.insert(
            "error".into(),
            json!(no_abrio.texto_con(pedido.idioma, &[("clase", clase)])),
        );
    }
    Ok(respuesta)
}

fn anfitrion_de(pedido: Pedido, funcion: &Funcion) -> Result<Hwnd> {
    let ausente = match ventana_de_clase(pedido, funcion) {
        Ok(h) => return Ok(h),
        Err(e) => e,
    };
    let Some(padre) = funcion.requiere else {
        return Err(ausente);
    };
    abrir_funcion(pedido, funcion_unica(padre, pedido.idioma)?)?;
    std::thread::sleep(ASENTAMIENTO_DEL_ANFITRION);
    ventana_de_clase(pedido, funcion)
}

/// `gui_actuar accion:menu`: posts a menu command to `ventana`, named by its
/// command `id` or by its place in the menu bar (`menu` and `item`).
pub(crate) fn menu(args: &Value) -> Result<Value> {
    // Same as `abrir`: the scene importer calls it without a `Paths`.
    let pedido = Pedido {
        args,
        idioma: Idioma::de_entorno(),
    };
    let pedido_menu = pedido_de_menu(args, pedido.idioma)?;
    let anfitrion = hwnd_de(args, "ventana")?;
    exigir_menu_en_la_barra(anfitrion, pedido_menu, pedido.idioma)?;
    let id = id_de_pedido(anfitrion, pedido_menu)?;
    let antes = visibles(args)?;
    enviar_comando(anfitrion, id, pedido.idioma)?;
    let visto = esperar_nuevas(
        SolicitudEspera {
            antes: &antes,
            esperada: None,
            limite: espera_pedida(args),
        },
        || visibles(args),
    )?;
    Ok(json!({
        "id": id,
        "ventanas_nuevas": visto.nuevas_json(),
    }))
}

/// A menu number past the end of the bar wraps around to the first menus and
/// would run another command: a request typed by hand is checked against the
/// real bar. Catalogue entries are fixed data, verified live, and skip this.
fn exigir_menu_en_la_barra(anfitrion: Hwnd, pedido: PedidoMenu, idioma: Idioma) -> Result<()> {
    let (PedidoMenu::Posicion { menu, .. } | PedidoMenu::Submenu { menu, .. }) = pedido else {
        return Ok(());
    };
    exigir_anfitrion_habilitado(anfitrion)?;
    let menus = contar_menus(anfitrion)?;
    if menu < menus {
        return Ok(());
    }
    let fuera_de_rango = Texto {
        es: "La barra de {clase} tiene {menus} menús (0..{ultimo}): 'menu' {menu} daría la vuelta",
        en: "The {clase} bar has {menus} menus (0..{ultimo}): 'menu' {menu} would wrap around",
    };
    Err(Error::Invalid(fuera_de_rango.texto_con(
        idioma,
        &[
            ("clase", &class_of(anfitrion)),
            ("menus", &menus.to_string()),
            ("ultimo", &menus.saturating_sub(1).to_string()),
            ("menu", &menu.to_string()),
        ],
    )))
}

fn id_de_pedido(anfitrion: Hwnd, pedido: PedidoMenu) -> Result<u16> {
    match pedido {
        PedidoMenu::Id(id) => Ok(id),
        PedidoMenu::Posicion { menu, item } => {
            exigir_anfitrion_habilitado(anfitrion)?;
            id_en_posicion(anfitrion, Posicion { menu, item })
        }
        PedidoMenu::Submenu { menu, item, sub } => {
            exigir_anfitrion_habilitado(anfitrion)?;
            id_en_submenu(anfitrion, Posicion { menu, item }, sub)
        }
    }
}

fn funcion_unica(clave: &str, idioma: Idioma) -> Result<&'static Funcion> {
    match buscar(funciones(), clave).as_slice() {
        [unica] => Ok(*unica),
        [] => {
            let fuera = Texto {
                es: "Función '{clave}' fuera del catálogo (gui_observar que:funciones)",
                en: "Function '{clave}' is outside the catalogue (gui_observar que:funciones)",
            };
            Err(Error::NotFound(
                fuera.texto_con(idioma, &[("clave", clave)]),
            ))
        }
        varias => {
            let ambigua = Texto {
                es: "'{clave}' es ambigua: {claves}",
                en: "'{clave}' is ambiguous: {claves}",
            };
            Err(Error::Invalid(
                ambigua.texto_con(
                    idioma,
                    &[
                        ("clave", clave),
                        (
                            "claves",
                            &varias
                                .iter()
                                .map(|f| f.clave)
                                .collect::<Vec<_>>()
                                .join(", "),
                        ),
                    ],
                ),
            ))
        }
    }
}

fn ventana_de_clase(pedido: Pedido, funcion: &Funcion) -> Result<Hwnd> {
    let clase = funcion.ventana;
    ventanas_de(pedido.args)?
        .into_iter()
        .find(|h| {
            visible(*h)
                && class_of(*h) == clase
                && funcion.titulo_anfitrion.is_none_or(|t| text_of(*h) == t)
        })
        .ok_or_else(|| {
            let cerrada = Texto {
                es: "La ventana {clase} no está abierta en PVsyst",
                en: "Window {clase} is not open in PVsyst",
            };
            Error::NotFound(cerrada.texto_con(pedido.idioma, &[("clase", clase)]))
        })
}

fn visibles(args: &Value) -> Result<Vec<Hwnd>> {
    Ok(ventanas_de(args)?
        .into_iter()
        .filter(|h| visible(*h))
        .collect())
}

/// A VCL modal dialog disables the form it was opened from. A menu command
/// posted to that form still runs, and PVsyst raises an internal exception
/// that forces a restart, so commands are refused while the host is disabled.
fn exigir_anfitrion_habilitado(anfitrion: Hwnd) -> Result<()> {
    modal::exigir_habilitada(anfitrion, "sending menu commands")
}

fn enviar_comando(anfitrion: Hwnd, id: u16, idioma: Idioma) -> Result<()> {
    exigir_anfitrion_habilitado(anfitrion)?;
    if ffi::post_message(
        anfitrion,
        ffi::Mensaje::nuevo(WM_COMMAND, usize::from(id), 0),
    ) {
        return Ok(());
    }
    let sin_mensaje = Texto {
        es: "PostMessage falló para la ventana 0x{hwnd}",
        en: "PostMessage failed for window 0x{hwnd}",
    };
    Err(Error::Gui(
        sin_mensaje.texto_con(idioma, &[("hwnd", &format!("{anfitrion:X}"))]),
    ))
}

fn disparar(pedido: Pedido, anfitrion: Hwnd, funcion: &Funcion) -> Result<()> {
    match funcion.acceso {
        Acceso::Menu { id } => enviar_comando(anfitrion, id, pedido.idioma),
        Acceso::MenuPosicion { menu, item } => {
            let posicion = PedidoMenu::Posicion {
                menu: usize::from(menu),
                item: usize::from(item),
            };
            enviar_comando(anfitrion, id_de_pedido(anfitrion, posicion)?, pedido.idioma)
        }
        Acceso::Boton { titulo } => {
            exigir_anfitrion_habilitado(anfitrion)?;
            clic_por_mensajes(esperar_boton(pedido, anfitrion, titulo)?)
        }
    }
}

/// The enabled button `titulo` of `anfitrion`, waiting for it: a form that has
/// just opened builds its controls for a moment, and a click sent before they
/// are enabled is lost without an error.
fn esperar_boton(pedido: Pedido, anfitrion: Hwnd, titulo: &str) -> Result<Hwnd> {
    let fin = Instant::now() + espera_pedida(pedido.args);
    loop {
        let boton = hijo_visible_con_texto(anfitrion, titulo);
        if let Some(b) = boton.filter(|b| ffi::estado_ventana(*b).habilitada) {
            return Ok(b);
        }
        if Instant::now() >= fin {
            return Err(boton_ausente(titulo, boton.is_some(), pedido.idioma));
        }
        std::thread::sleep(SONDEO);
    }
}

/// Why the button never became clickable.
fn boton_ausente(titulo: &str, existe: bool, idioma: Idioma) -> Error {
    let deshabilitado = Texto {
        es: "Botón '{titulo}' deshabilitado",
        en: "Button '{titulo}' is disabled",
    };
    let invisible = Texto {
        es: "Botón '{titulo}' no visible",
        en: "Button '{titulo}' is not visible",
    };
    let motivo = if existe { deshabilitado } else { invisible };
    Error::NotFound(motivo.texto_con(idioma, &[("titulo", titulo)]))
}

/// The wait the caller asked for, capped.
fn espera_pedida(args: &Value) -> Duration {
    let ms = args
        .get("timeout_ms")
        .and_then(Value::as_u64)
        .unwrap_or(ESPERA_POR_DEFECTO_MS);
    Duration::from_millis(ms.min(ESPERA_MAXIMA_MS))
}
