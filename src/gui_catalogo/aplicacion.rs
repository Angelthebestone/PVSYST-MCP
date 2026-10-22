//! Runs catalogue entries against the live PVsyst: finds the host window,
//! triggers the menu item or button, and reports the windows that appear.

use super::dominio::{buscar, pedido_de_menu, Acceso, Efecto, Funcion, PedidoMenu};
use super::entradas::funciones;
use crate::adapters::gui::acciones::clic_por_mensajes;
use crate::adapters::gui::conduccion::Formulario;
use crate::adapters::gui::ffi::{self, Hwnd, WM_COMMAND};
use crate::adapters::gui::menu_por_posicion::{contar_menus, id_en_posicion, Posicion};
use crate::adapters::gui::ventanas::{
    class_of, hijo_visible_con_texto, hwnd_de, text_of, ventanas_de, visible, window_json,
};
use crate::args::req_str;
use crate::error::{Error, Result};
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
    let lista: Vec<Value> = match args.get("funcion").and_then(Value::as_str) {
        Some(clave) => buscar(funciones(), clave)
            .iter()
            .map(|f| f.a_json())
            .collect(),
        None => funciones().map(Funcion::a_json).collect(),
    };
    json!({ "count": lista.len(), "funciones": lista })
}

/// `gui_actuar accion:funcion`: opens one catalogued function, first opening
/// the window that hosts its button when that window is closed.
pub(crate) fn abrir(args: &Value) -> Result<Value> {
    let clave = req_str(
        args,
        &["funcion"],
        "clave del catálogo (gui_observar que:funciones)",
    )?;
    abrir_funcion(args, funcion_unica(clave)?)
}

fn abrir_funcion(args: &Value, funcion: &Funcion) -> Result<Value> {
    let anfitrion = anfitrion_de(args, funcion)?;
    if let Some(pagina) = funcion.pagina {
        Formulario::de(anfitrion).pagina(usize::from(pagina))?;
    }
    let antes = visibles(args)?;
    disparar(args, anfitrion, funcion)?;
    // Nothing is left to watch once PVsyst itself is closing.
    let visto = match funcion.efecto {
        Efecto::Cierra => Resultado::vacio(),
        _ => esperar_nuevas(args, &antes, funcion.abre)?,
    };
    Ok(json!({
        "funcion": funcion.clave,
        "etiqueta": funcion.etiqueta,
        "efecto": funcion.a_json()["efecto"],
        "ventanas_nuevas": visto.nuevas_json(),
        "ya_abiertas": visto.ya_abiertas_json(),
        "nota": funcion.nota,
    }))
}

fn anfitrion_de(args: &Value, funcion: &Funcion) -> Result<Hwnd> {
    let ausente = match ventana_de_clase(args, funcion) {
        Ok(h) => return Ok(h),
        Err(e) => e,
    };
    let Some(padre) = funcion.requiere else {
        return Err(ausente);
    };
    abrir_funcion(args, funcion_unica(padre)?)?;
    std::thread::sleep(ASENTAMIENTO_DEL_ANFITRION);
    ventana_de_clase(args, funcion)
}

/// `gui_actuar accion:menu`: posts a menu command to `ventana`, named by its
/// command `id` or by its place in the menu bar (`menu` and `item`).
pub(crate) fn menu(args: &Value) -> Result<Value> {
    let pedido = pedido_de_menu(args)?;
    let anfitrion = hwnd_de(args, "ventana")?;
    exigir_menu_en_la_barra(anfitrion, pedido)?;
    let id = id_de_pedido(anfitrion, pedido)?;
    let antes = visibles(args)?;
    enviar_comando(anfitrion, id)?;
    let visto = esperar_nuevas(args, &antes, None)?;
    Ok(json!({
        "id": id,
        "ventanas_nuevas": visto.nuevas_json(),
    }))
}

/// A menu number past the end of the bar wraps around to the first menus and
/// would run another command: a request typed by hand is checked against the
/// real bar. Catalogue entries are fixed data, verified live, and skip this.
fn exigir_menu_en_la_barra(anfitrion: Hwnd, pedido: PedidoMenu) -> Result<()> {
    let PedidoMenu::Posicion { menu, .. } = pedido else {
        return Ok(());
    };
    exigir_anfitrion_habilitado(anfitrion)?;
    let menus = contar_menus(anfitrion)?;
    if menu < menus {
        return Ok(());
    }
    Err(Error::Invalid(format!(
        "La barra de {} tiene {menus} menús (0..{}): 'menu' {menu} daría la vuelta",
        class_of(anfitrion),
        menus.saturating_sub(1)
    )))
}

fn id_de_pedido(anfitrion: Hwnd, pedido: PedidoMenu) -> Result<u16> {
    match pedido {
        PedidoMenu::Id(id) => Ok(id),
        PedidoMenu::Posicion { menu, item } => {
            exigir_anfitrion_habilitado(anfitrion)?;
            id_en_posicion(anfitrion, Posicion { menu, item })
        }
    }
}

fn funcion_unica(clave: &str) -> Result<&'static Funcion> {
    match buscar(funciones(), clave).as_slice() {
        [unica] => Ok(*unica),
        [] => Err(Error::NotFound(format!(
            "Función '{clave}' fuera del catálogo (gui_observar que:funciones)"
        ))),
        varias => Err(Error::Invalid(format!(
            "'{clave}' es ambigua: {}",
            varias
                .iter()
                .map(|f| f.clave)
                .collect::<Vec<_>>()
                .join(", ")
        ))),
    }
}

fn ventana_de_clase(args: &Value, funcion: &Funcion) -> Result<Hwnd> {
    let clase = funcion.ventana;
    ventanas_de(args)?
        .into_iter()
        .find(|h| {
            visible(*h)
                && class_of(*h) == clase
                && funcion.titulo_anfitrion.is_none_or(|t| text_of(*h) == t)
        })
        .ok_or_else(|| Error::NotFound(format!("Ventana {clase} no está abierta en PVsyst")))
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
    if ffi::estado_ventana(anfitrion).habilitada {
        return Ok(());
    }
    Err(Error::Gui(format!(
        "{} está deshabilitada: hay un diálogo modal abierto sobre ella. Ciérralo (gui_actuar accion:cerrar) antes de lanzar comandos de menú",
        class_of(anfitrion)
    )))
}

fn enviar_comando(anfitrion: Hwnd, id: u16) -> Result<()> {
    exigir_anfitrion_habilitado(anfitrion)?;
    if ffi::post_message(
        anfitrion,
        ffi::Mensaje::nuevo(WM_COMMAND, usize::from(id), 0),
    ) {
        return Ok(());
    }
    Err(Error::Gui(format!(
        "PostMessage falló para la ventana 0x{anfitrion:X}"
    )))
}

fn disparar(args: &Value, anfitrion: Hwnd, funcion: &Funcion) -> Result<()> {
    match funcion.acceso {
        Acceso::Menu { id } => enviar_comando(anfitrion, id),
        Acceso::MenuPosicion { menu, item } => {
            let pedido = PedidoMenu::Posicion {
                menu: usize::from(menu),
                item: usize::from(item),
            };
            enviar_comando(anfitrion, id_de_pedido(anfitrion, pedido)?)
        }
        Acceso::Boton { titulo } => {
            exigir_anfitrion_habilitado(anfitrion)?;
            clic_por_mensajes(esperar_boton(anfitrion, titulo, espera_pedida(args))?)
        }
    }
}

/// The enabled button `titulo` of `anfitrion`, waiting for it: a form that has
/// just opened builds its controls for a moment, and a click sent before they
/// are enabled is lost without an error.
fn esperar_boton(anfitrion: Hwnd, titulo: &str, limite: Duration) -> Result<Hwnd> {
    let fin = Instant::now() + limite;
    loop {
        let boton = hijo_visible_con_texto(anfitrion, titulo);
        if let Some(b) = boton.filter(|b| ffi::estado_ventana(*b).habilitada) {
            return Ok(b);
        }
        if Instant::now() >= fin {
            return Err(Error::NotFound(match boton {
                Some(_) => format!("Botón '{titulo}' deshabilitado"),
                None => format!("Botón '{titulo}' no visible"),
            }));
        }
        std::thread::sleep(SONDEO);
    }
}

/// The wait the caller asked for, capped.
fn espera_pedida(args: &Value) -> Duration {
    let ms = args
        .get("timeout_ms")
        .and_then(Value::as_u64)
        .unwrap_or(ESPERA_POR_DEFECTO_MS);
    Duration::from_millis(ms.min(ESPERA_MAXIMA_MS))
}

/// Windows that appeared since `antes`.
fn ventanas_nuevas(ahora: &[Hwnd], antes: &[Hwnd]) -> Vec<Hwnd> {
    ahora
        .iter()
        .copied()
        .filter(|h| !antes.contains(h))
        .collect()
}

/// Windows of class `clase` among `ventanas`.
fn de_clase(ventanas: &[Hwnd], clase: &str) -> Vec<Hwnd> {
    ventanas
        .iter()
        .copied()
        .filter(|h| class_of(*h) == clase)
        .collect()
}

/// What an invocation showed.
struct Resultado {
    nuevas: Vec<Hwnd>,
    /// Windows of the expected class that were already open: the command may
    /// have only raised them.
    ya_abiertas: Vec<Hwnd>,
}

impl Resultado {
    fn vacio() -> Self {
        Self {
            nuevas: Vec::new(),
            ya_abiertas: Vec::new(),
        }
    }

    fn nuevas_json(&self) -> Vec<Value> {
        self.nuevas.iter().map(|h| window_json(*h)).collect()
    }

    fn ya_abiertas_json(&self) -> Vec<Value> {
        self.ya_abiertas.iter().map(|h| window_json(*h)).collect()
    }
}

/// Waits for a window that was not there before (of class `esperada` when one
/// is expected). Returns an empty list on timeout: some items open nothing. A
/// PVsyst that went away meanwhile (the command closed it) ends the wait.
fn esperar_nuevas(args: &Value, antes: &[Hwnd], esperada: Option<&str>) -> Result<Resultado> {
    let fin = Instant::now() + espera_pedida(args);
    loop {
        let Ok(ahora) = visibles(args) else {
            return Ok(Resultado::vacio());
        };
        let nuevas = ventanas_nuevas(&ahora, antes);
        let llegada = match esperada {
            Some(clase) => !de_clase(&nuevas, clase).is_empty(),
            None => !nuevas.is_empty(),
        };
        if llegada || Instant::now() >= fin {
            let ya_abiertas = esperada.map_or_else(Vec::new, |c| {
                de_clase(antes, c)
                    .into_iter()
                    .filter(|h| ahora.contains(h))
                    .collect()
            });
            return Ok(Resultado {
                nuevas,
                ya_abiertas,
            });
        }
        std::thread::sleep(SONDEO);
    }
}
