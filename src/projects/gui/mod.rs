//! Project steps that only PVsyst's own windows perform: array pre-sizing,
//! horizon import with its shading factors, and the P50-P90 evaluation set-up.
//! The MCP opens the project in PVsyst, drives the window, and lets PVsyst save
//! the variant; the result is read back from the file PVsyst wrote.

pub(crate) mod dimensionado;
pub(crate) mod horizonte;
pub(crate) mod p50;
pub(crate) mod rotulos;

use crate::adapters::gui::conduccion::{Campo, Formulario};
use crate::adapters::gui::rotulos::Idioma;
use crate::adapters::gui::sesion_pvsyst::Sesion;
use crate::config::Paths;
use crate::error::{Error, Result};
use rotulos::{rotulos_proyecto_gui, RotulosProyectoGui};

const ROTULOS: RotulosProyectoGui = rotulos_proyecto_gui(Idioma::Es);
use crate::format::object::{self, PvNode};
use crate::projects::lifecycle::variantes::{es_variante_valida, ruta_variante};
use crate::projects::tipo_sistema::TipoSistema;
use std::fs;
use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};

/// Offsets (px at 96 DPI) of "Carga" in the project bar and "Guardar" in the variant bar.
const X_CARGAR: i32 = 100;
const X_GUARDAR_VARIANTE: i32 = 113;
/// First row of the project list, relative to the list's top-left corner.
const FILA_1: (i32, i32) = (80, 28);
/// Variant selector of the project window.
const SELECTOR_VARIANTE: Campo = Campo::combo(448, 339);
const ESPERA_TITULO: Duration = Duration::from_secs(10);

/// A project open in PVsyst on one variant.
pub(crate) struct ProyectoEnPvsyst<'s> {
    pub(crate) sesion: &'s Sesion,
    pub(crate) ventana: Formulario,
    pub(crate) archivo_variante: PathBuf,
}

fn boton_de_inicio(tipo: TipoSistema) -> &'static str {
    match tipo {
        TipoSistema::Red => ROTULOS.inicio_red,
        TipoSistema::Aislado => ROTULOS.inicio_aislado,
        TipoSistema::Bombeo => ROTULOS.inicio_bombeo,
        TipoSistema::RedDc => ROTULOS.inicio_red_dc,
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

fn cargar(sesion: &Sesion, proyecto: &Formulario, prj: &str) -> Result<()> {
    proyecto.clic_barra(0, X_CARGAR)?;
    let lista = sesion.esperar(ROTULOS.formulario_cargar)?;
    lista.escribir_unico("TEdit", prj)?;
    lista.clic_en_unico("TVirtualStringTree", FILA_1)?;
    lista.pulsar(ROTULOS.ok)?;
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
        "Variante",
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
        let ventana = sesion.esperar(ROTULOS.formulario_proyecto)?;
        let prj = format!("{proyecto}.PRJ");
        if !ventana.titulo().ends_with(&prj) {
            cargar(sesion, &ventana, &prj)?;
        }
        ventana.seleccionar(SELECTOR_VARIANTE, &variante.to_ascii_uppercase())?;
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
        self.ventana.clic_barra(1, X_GUARDAR_VARIANTE)?;
        let dialogo = self.sesion.esperar(ROTULOS.formulario_guardar)?;
        dialogo.pulsar(ROTULOS.guardar)?;
        self.sesion.aceptar_mensajes();
        let despues = crate::workspace::leer_bytes(&self.archivo_variante)?;
        if despues == antes {
            return Err(Error::Gui(format!(
                "PVsyst no guardó {}: ¿usa PVsyst otro espacio de trabajo?",
                self.archivo_variante.display()
            )));
        }
        object::parse_first(&String::from_utf8_lossy(&despues))
            .ok_or_else(|| Error::Format("La variante guardada no es un PVObject".into()))
    }

    pub(crate) fn leer_variante(&self) -> Result<PvNode> {
        let bytes = crate::workspace::leer_bytes(&self.archivo_variante)?;
        object::parse_first(&String::from_utf8_lossy(&bytes))
            .ok_or_else(|| Error::Format("La variante guardada no es un PVObject".into()))
    }
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
