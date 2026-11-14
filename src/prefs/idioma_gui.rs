//! `gui_observar que:idioma`: the language PVsyst's own windows are written in.
//!
//! It is read from the captions of the open windows (what the user sees, and
//! the truth while PVsyst runs) and from the `Dialog Language` of `PVsyst.ini`
//! (PVsyst writes it on exit). It is not `PVSYST_MCP_IDIOMA`, which only picks
//! the language of the MCP's messages.

use super::ini::{decodificar, ini_path, leer, parsear};
use crate::adapters::gui::idioma_gui;
use crate::adapters::gui::rotulos::IdiomaGui;
use crate::config::Paths;
use crate::error::Result;
use crate::idioma::{Idioma, Texto};
use serde_json::{json, Value};

const SECCION: &str = "Preferences";
const CLAVE: &str = "Dialog Language";

/// The `Dialog Language` of `PVsyst.ini` text (`Spanish`, `English`...), as written.
pub(crate) fn nombre_en_ini(texto: &str) -> Option<String> {
    let secciones = parsear(texto);
    let (_, seccion) = secciones
        .iter()
        .find(|(nombre, _)| nombre.eq_ignore_ascii_case(SECCION))?;
    let (_, valor) = seccion
        .as_object()?
        .iter()
        .find(|(clave, _)| clave.eq_ignore_ascii_case(CLAVE))?;
    valor
        .as_str()
        .map(str::trim)
        .filter(|v| !v.is_empty())
        .map(str::to_string)
}

/// What each source says about the language of PVsyst's windows.
pub(crate) struct Lectura {
    /// `Dialog Language` of `PVsyst.ini`, as written.
    pub(crate) ini: Option<String>,
    /// What the captions of the open windows say.
    pub(crate) ventanas: Option<IdiomaGui>,
}

impl Lectura {
    /// Reads both sources now: the ini of the user and the windows of the running PVsyst.
    pub(crate) fn actual() -> Self {
        let ini = ini_path()
            .and_then(|ruta| leer(&ruta))
            .ok()
            .and_then(|bytes| nombre_en_ini(&decodificar(&bytes).0));
        Self {
            ini,
            ventanas: idioma_gui::en_ejecucion(),
        }
    }

    fn idioma_del_ini(&self) -> Option<IdiomaGui> {
        self.ini.as_deref().and_then(IdiomaGui::de_nombre)
    }

    /// The language to trust: the open windows over the file, which PVsyst
    /// rewrites only when it exits.
    pub(crate) fn idioma(&self) -> Option<IdiomaGui> {
        self.ventanas.or_else(|| self.idioma_del_ini())
    }

    /// The language as a code (`es`) when the captions cover it, else the name
    /// PVsyst gave it (`Italian`).
    pub(crate) fn codigo_o_nombre(&self) -> Option<String> {
        match self.idioma() {
            Some(idioma) => Some(idioma.codigo().to_string()),
            None => self.ini.clone(),
        }
    }

    fn fuente(&self) -> Option<&'static str> {
        match (self.ventanas, &self.ini) {
            (Some(_), _) => Some("ventanas"),
            (None, Some(_)) => Some("PVsyst.ini"),
            (None, None) => None,
        }
    }

    /// What the caller should know before relying on captions, or `None`.
    fn aviso(&self, idioma_cliente: Idioma) -> Option<String> {
        let texto = |t: Texto, vars: &[(&str, &str)]| t.texto_con(idioma_cliente, vars);
        let nombre = self.ini.as_deref().unwrap_or_default();
        let Some(idioma) = self.idioma() else {
            return Some(match self.ini {
                Some(_) => texto(IDIOMA_SIN_ROTULOS, &[("nombre", nombre)]),
                None => texto(IDIOMA_DESCONOCIDO, &[]),
            });
        };
        if self.ventanas.is_some() && self.idioma_del_ini().is_some_and(|i| i != idioma) {
            return Some(texto(
                INI_DISTINTO,
                &[("ini", nombre), ("ventanas", idioma.codigo())],
            ));
        }
        (!IdiomaGui::MEDIDOS.contains(&idioma))
            .then(|| texto(SIN_MEDIR, &[("idioma", idioma.codigo())]))
    }

    pub(crate) fn como_json(&self, idioma_cliente: Idioma) -> Value {
        let codigos =
            |idiomas: &[IdiomaGui]| idiomas.iter().map(|i| i.codigo()).collect::<Vec<_>>();
        json!({
            "idioma_gui": self.idioma().map(IdiomaGui::codigo),
            "fuente": self.fuente(),
            "nombre_en_ini": self.ini,
            "idioma_en_ventanas": self.ventanas.map(IdiomaGui::codigo),
            "idiomas_con_rotulos": codigos(&IdiomaGui::TODOS),
            "idiomas_medidos": codigos(&IdiomaGui::MEDIDOS),
            "idioma_mensajes_mcp": match idioma_cliente {
                Idioma::Es => "es",
                Idioma::En => "en",
            },
            "aviso": self.aviso(idioma_cliente),
            "nota": NOTA.en_idioma(idioma_cliente),
        })
    }
}

const NOTA: Texto = Texto {
    es: "idioma_gui es el idioma de las ventanas de PVsyst, el que se cambia en PVsyst. PVSYST_MCP_IDIOMA solo cambia el de los mensajes del MCP.",
    en: "idioma_gui is the language of PVsyst's windows, the one set in PVsyst. PVSYST_MCP_IDIOMA only changes the language of the MCP's messages.",
};
const IDIOMA_DESCONOCIDO: Texto = Texto {
    es: "No se pudo saber el idioma de PVsyst: PVsyst.ini no lo dice y no hay ventanas abiertas con rótulos conocidos. Lo que se encuentra por estructura (clase, índice, posición) no depende de él.",
    en: "PVsyst's language could not be told: PVsyst.ini does not say and no open window shows known captions. What is found by structure (class, index, position) does not depend on it.",
};
const IDIOMA_SIN_ROTULOS: Texto = Texto {
    es: "PVsyst está en '{nombre}', sin rótulos en el servidor: los botones que se pulsan por rótulo no se encontrarán. Usa gui_observar que:inventario y gui_actuar click con el hwnd.",
    en: "PVsyst is in '{nombre}', which the server has no captions for: buttons pressed by caption will not be found. Use gui_observar que:inventario and gui_actuar click with the hwnd.",
};
const INI_DISTINTO: Texto = Texto {
    es: "PVsyst.ini dice '{ini}' pero las ventanas abiertas están en '{ventanas}': vale el de las ventanas (PVsyst reescribe el ini al salir).",
    en: "PVsyst.ini says '{ini}' but the open windows are in '{ventanas}': the windows win (PVsyst rewrites the ini on exit).",
};
const SIN_MEDIR: Texto = Texto {
    es: "Los rótulos en '{idioma}' están inferidos, no medidos en un PVsyst real: si un botón no se encuentra, usa gui_observar que:inventario y gui_actuar click con el hwnd.",
    en: "The '{idioma}' captions are inferred, not measured on a real PVsyst: if a button is not found, use gui_observar que:inventario and gui_actuar click with the hwnd.",
};

/// `gui_observar que:idioma`.
pub(crate) fn observar(paths: &Paths) -> Result<Value> {
    Ok(Lectura::actual().como_json(paths.idioma))
}

#[cfg(test)]
#[path = "../../tests/unit/prefs/idioma_gui.rs"]
mod tests;
