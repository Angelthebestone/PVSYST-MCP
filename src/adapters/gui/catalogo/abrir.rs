//! `gui_actuar accion:funcion`: opens one catalogued function, first opening
//! the window that hosts its button when that window is closed.

use super::aplicacion::{
    enviar_comando, espera_pedida, exigir_anfitrion_habilitado, id_de_pedido, visibles,
};
use super::dominio::{buscar, Acceso, Efecto, Funcion, PedidoMenu};
use super::entradas::funciones;
use super::espera::{esperar_nuevas, Resultado, SolicitudEspera};
use crate::adapters::gui::acciones::clic_por_mensajes;
use crate::adapters::gui::conduccion::Formulario;
use crate::adapters::gui::ffi::{self, Hwnd};
use crate::adapters::gui::resumen::resumen_de_ventana;
use crate::adapters::gui::rotulos::Rotulo;
use crate::adapters::gui::ventanas::{
    class_of, hijo_visible_con_texto, text_of, ventanas_de, visible,
};
use crate::args::req_str;
use crate::error::{Error, Result};
use crate::idioma::{Idioma, Texto};
use serde_json::{json, Value};
use std::time::{Duration, Instant};

const SONDEO: Duration = Duration::from_millis(100);
/// A form that has just opened keeps computing (the system form needs several
/// seconds); a click sent meanwhile is lost without an error. Measured: 3.5 s
/// to appear, a click right after it opens nothing, one 5 s later does.
const ASENTAMIENTO_DEL_ANFITRION: Duration = Duration::from_secs(5);

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
    let de_la_clase: Vec<Hwnd> = ventanas_de(pedido.args)?
        .into_iter()
        .filter(|h| visible(*h) && class_of(*h) == clase)
        .collect();
    let esperado = funcion.titulo_anfitrion;
    de_la_clase
        .iter()
        .copied()
        .find(|h| esperado.is_none_or(|t| text_of(*h) == t))
        .ok_or_else(|| sin_ventana(pedido.idioma, (clase, esperado), &de_la_clase))
}

/// Why the host window was not found. A window of the class with another title is
/// told apart: the title is a caption PVsyst writes in its own language.
pub(super) fn sin_ventana(
    idioma: Idioma,
    (clase, esperado): (&str, Option<&str>),
    abiertas: &[Hwnd],
) -> Error {
    let cerrada = Texto {
        es: "La ventana {clase} no está abierta en PVsyst",
        en: "Window {clase} is not open in PVsyst",
    };
    let otro_titulo = Texto {
        es: "Hay una {clase} abierta con el título '{titulo}', pero el catálogo espera '{esperado}': el título depende del idioma de PVsyst (gui_observar que:idioma). Nómbrala con ventana:<hwnd> (gui_observar que:ventanas)",
        en: "A {clase} is open with the title '{titulo}', but the catalogue expects '{esperado}': the title depends on PVsyst's language (gui_observar que:idioma). Name it with ventana:<hwnd> (gui_observar que:ventanas)",
    };
    let titulos: Vec<String> = abiertas.iter().map(|h| text_of(*h)).collect();
    let motivo = match (esperado, titulos.is_empty()) {
        (Some(esperado), false) => otro_titulo.texto_con(
            idioma,
            &[
                ("clase", clase),
                ("titulo", &titulos.join("', '")),
                ("esperado", esperado),
            ],
        ),
        _ => cerrada.texto_con(idioma, &[("clase", clase)]),
    };
    Error::NotFound(motivo)
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
fn esperar_boton(pedido: Pedido, anfitrion: Hwnd, titulo: Rotulo<'static>) -> Result<Hwnd> {
    let fin = Instant::now() + espera_pedida(pedido.args);
    loop {
        let boton = hijo_visible_con_texto(anfitrion, titulo);
        if let Some(b) = boton.filter(|b| ffi::estado_ventana(*b).habilitada) {
            return Ok(b);
        }
        if Instant::now() >= fin {
            return Err(boton_ausente(&titulo, boton.is_some(), pedido.idioma));
        }
        std::thread::sleep(SONDEO);
    }
}

/// Why the button never became clickable.
fn boton_ausente(titulo: &Rotulo<'_>, existe: bool, idioma: Idioma) -> Error {
    let deshabilitado = Texto {
        es: "Botón '{titulo}' deshabilitado",
        en: "Button '{titulo}' is disabled",
    };
    let invisible = Texto {
        es: "Botón '{titulo}' no visible: se busca por ese rótulo. Si PVsyst usa otro texto (gui_observar que:idioma dice en qué idioma está), localiza el botón con gui_observar que:inventario y púlsalo con gui_actuar click por hwnd",
        en: "Button '{titulo}' is not visible: it is looked for by that caption. If PVsyst uses another text (gui_observar que:idioma says which language it is in), find the button with gui_observar que:inventario and press it with gui_actuar click by hwnd",
    };
    let motivo = if existe { deshabilitado } else { invisible };
    Error::NotFound(motivo.texto_con(idioma, &[("titulo", &titulo.to_string())]))
}
