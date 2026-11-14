//! Project steps that only PVsyst's own windows perform: array pre-sizing,
//! horizon import with its shading factors, and the P50-P90 evaluation set-up.
//! The MCP opens the project in PVsyst, drives the window, and lets PVsyst save
//! the variant; the result is read back from the file PVsyst wrote.

pub(crate) mod dimensionado;
pub(crate) mod horizonte;
pub(crate) mod lanzar;
pub(crate) mod p50;
mod relleno;
pub(crate) mod rotulos;

use crate::adapters::gui::conduccion::Formulario;
use crate::adapters::gui::rotulos::dialogos::{GUARDAR, OK};
use crate::adapters::gui::rotulos::pvsyst::{
    INICIO_AISLADO, INICIO_BOMBEO, INICIO_RED, INICIO_RED_DC,
};
use crate::adapters::gui::rotulos::Rotulo;
use crate::adapters::gui::sesion_pvsyst::Sesion;
use crate::adapters::gui::sondeo::sondear_hasta;
use crate::config::Paths;
use crate::error::{Error, Result};
use rotulos::{FORMULARIO_CARGAR, FORMULARIO_GUARDAR, FORMULARIO_PROYECTO};
use serde_json::json;

use crate::format::object::{self, PvNode};
use crate::projects::lifecycle::variantes::{es_variante_valida, ruta_variante};
use crate::projects::tipo_sistema::TipoSistema;
use std::fs;
use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};

/// The project window's commands, as the GUI catalogue names them (menu entries by position).
const CARGAR_PROYECTO: &str = "ventana_proyecto.proyecto.cargar";
const GUARDAR_VARIANTE: &str = "ventana_proyecto.variante.guardar";
/// The project list shows the file typed in the search box as its first row.
const PRIMERA_FILA: u64 = 0;
const ESPERA_TITULO: Duration = Duration::from_secs(10);
const ESPERA_GUARDADO: Duration = Duration::from_secs(20);
const SONDEO_GUARDADO: Duration = Duration::from_millis(300);

/// A project open in PVsyst on one variant.
pub(crate) struct ProyectoEnPvsyst<'s> {
    pub(crate) sesion: &'s Sesion,
    pub(crate) ventana: Formulario,
    pub(crate) archivo_variante: PathBuf,
}

/// A session for a step that loads a project and saves its variant. Both go through the menu
/// bar, which PVsyst opens only for the keyboard, and the isolated desktop ignores keys: the
/// step is refused before PVsyst is touched instead of failing with a variant half edited.
pub(super) fn abrir_sesion(paths: &Paths) -> Result<Sesion> {
    if crate::adapters::gui::escritorio::es_aislado() {
        return Err(Error::Gui(
            "Cargar el proyecto y guardar la variante usan los menús de PVsyst, que se abren con el teclado: el escritorio aislado (PVSYST_MCP_ESCRITORIO=aislado) lo ignora. Quita esa variable para este paso".into(),
        ));
    }
    Sesion::abrir(paths)
}

fn boton_de_inicio(tipo: TipoSistema) -> Rotulo<'static> {
    match tipo {
        TipoSistema::Red => INICIO_RED,
        TipoSistema::Aislado => INICIO_AISLADO,
        TipoSistema::Bombeo => INICIO_BOMBEO,
        TipoSistema::RedDc => INICIO_RED_DC,
    }
}

fn esperar_titulo(f: &Formulario, prj: &str) -> Result<()> {
    let fin = Instant::now() + ESPERA_TITULO;
    while !f.titulo().ends_with(prj) {
        if Instant::now() >= fin {
            return Err(Error::Gui(format!(
                "PVsyst abrió '{}' en lugar de {prj}",
                f.titulo()
            )));
        }
        std::thread::sleep(Duration::from_millis(250));
    }
    Ok(())
}

/// Runs a catalogued command of the project window (its menu entry, found by position).
fn ejecutar(proyecto: &Formulario, funcion: &str) -> Result<()> {
    crate::adapters::gui::ffi::traer_al_frente(proyecto.h);
    crate::adapters::gui::catalogo::abrir(&json!({ "funcion": funcion })).map(|_| ())
}

fn cargar(sesion: &Sesion, proyecto: &Formulario, prj: &str) -> Result<()> {
    ejecutar(proyecto, CARGAR_PROYECTO)?;
    let lista = sesion.esperar(FORMULARIO_CARGAR)?;
    lista.escribir_unico("TEdit", prj)?;
    let arbol = lista.unico("TVirtualStringTree")?;
    crate::adapters::gui::fila::fila(
        &json!({ "ventana": format!("0x{arbol:X}"), "indice": PRIMERA_FILA }),
        crate::idioma::Idioma::de_entorno(),
    )?;
    lista.pulsar(OK)?;
    sesion.aceptar_mensajes();
    esperar_titulo(proyecto, prj)
}

/// Bytes of the variant file; a missing one lists the project folder's files instead of `os error 2`.
fn leer_variante(archivo: &Path, dir: &Path) -> Result<Vec<u8>> {
    if archivo.exists() {
        return crate::workspace::leer_bytes(archivo);
    }
    let pedido = archivo
        .file_name()
        .map(|n| n.to_string_lossy().into_owned())
        .unwrap_or_default();
    let opciones: Vec<String> = fs::read_dir(dir)
        .into_iter()
        .flatten()
        .flatten()
        .filter_map(|e| e.file_name().into_string().ok())
        .collect();
    Err(crate::args::no_encontrado(
        crate::args::VARIANTE,
        &pedido,
        &opciones,
        &dir.to_string_lossy(),
    ))
}

impl<'s> ProyectoEnPvsyst<'s> {
    /// Opens `proyecto` (stem) on `variante` in PVsyst.
    pub(crate) fn abrir(
        sesion: &'s Sesion,
        paths: &Paths,
        proyecto: &str,
        variante: &str,
    ) -> Result<Self> {
        if !es_variante_valida(variante) {
            return Err(Error::Invalid(format!("Variante inválida '{variante}'")));
        }
        let dir = paths.projects_dir();
        let archivo_variante = ruta_variante(&dir, proyecto, variante)?;
        let bytes = leer_variante(&archivo_variante, &dir)?;
        let tipo = TipoSistema::de_variante(&bytes)?;
        sesion.inicio.pulsar(boton_de_inicio(tipo))?;
        let ventana = sesion.esperar(FORMULARIO_PROYECTO)?;
        let prj = format!("{proyecto}.PRJ");
        if !ventana.titulo().ends_with(&prj) {
            cargar(sesion, &ventana, &prj)?;
        }
        let eleccion = json!({ "variante_activa": variante.to_ascii_uppercase() });
        relleno::rellenar(paths, &ventana, ("proyecto.ventana", eleccion))?;
        sesion.aceptar_mensajes();
        Ok(Self {
            sesion,
            ventana,
            archivo_variante,
        })
    }

    /// Saves the variant through PVsyst and checks that PVsyst wrote the file.
    pub(crate) fn guardar_variante(&self) -> Result<PvNode> {
        let antes = crate::workspace::leer_bytes(&self.archivo_variante)?;
        ejecutar(&self.ventana, GUARDAR_VARIANTE)?;
        let dialogo = self.sesion.esperar(FORMULARIO_GUARDAR)?;
        dialogo.pulsar(GUARDAR)?;
        self.sesion.aceptar_mensajes();
        let Some(despues) = esperar_guardado(&self.archivo_variante, &antes) else {
            return Err(Error::Gui(format!(
                "PVsyst no guardó {}: ¿usa PVsyst otro espacio de trabajo?",
                self.archivo_variante.display()
            )));
        };
        raiz_completa(&despues, &self.archivo_variante)
    }

    pub(crate) fn leer_variante(&self) -> Result<PvNode> {
        let bytes = crate::workspace::leer_bytes(&self.archivo_variante)?;
        raiz_completa(&bytes, &self.archivo_variante)
    }
}

/// PVsyst closes its save dialog before it has finished writing the variant: it
/// counts as saved once it differs from `antes` and two reads in a row agree.
/// Reading at once found it half written, or unchanged (BUG-05).
fn esperar_guardado(archivo: &Path, antes: &[u8]) -> Option<Vec<u8>> {
    let mut anterior: Option<Vec<u8>> = None;
    sondear_hasta(ESPERA_GUARDADO, SONDEO_GUARDADO, || {
        let actual = crate::workspace::leer_bytes(archivo).ok()?;
        let listo = es_guardado_estable(antes, anterior.as_deref(), &actual);
        anterior = Some(actual.clone());
        listo.then_some(actual)
    })
}

fn es_guardado_estable(antes: &[u8], anterior: Option<&[u8]>, actual: &[u8]) -> bool {
    actual != antes && anterior == Some(actual)
}

/// Root object of a saved variant; a truncated file is an error, not a wrongly nested tree.
fn raiz_completa(bytes: &[u8], archivo: &Path) -> Result<PvNode> {
    object::parse_completo(&String::from_utf8_lossy(bytes), archivo)?
        .into_iter()
        .next()
        .ok_or_else(|| Error::Format("La variante guardada no es un PVObject".into()))
}

/// Object `tipo` anywhere under `raiz`. O(n) in the tree.
pub(crate) fn objeto<'a>(raiz: &'a PvNode, tipo: &str) -> Option<&'a PvNode> {
    if raiz.obj_type.eq_ignore_ascii_case(tipo) {
        return Some(raiz);
    }
    raiz.children.iter().find_map(|c| objeto(c, tipo))
}

/// Number of `tipo` objects under `raiz`. O(n) in the tree.
pub(crate) fn contar(raiz: &PvNode, tipo: &str) -> usize {
    usize::from(raiz.obj_type.eq_ignore_ascii_case(tipo))
        + raiz.children.iter().map(|c| contar(c, tipo)).sum::<usize>()
}

#[cfg(test)]
#[path = "../../../tests/unit/projects/gui/variante.rs"]
mod tests;
