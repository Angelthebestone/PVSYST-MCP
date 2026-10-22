//! Array sizing by PVsyst's own "pre-sizing help" in the System window: given
//! a planned power, PVsyst chooses the modules in series, the strings and the
//! inverter count, and warns about the electrical window.

use super::rotulos::{rotulos_dimensionado, RotulosDimensionado};
use super::{contar, objeto, ProyectoEnPvsyst};
use crate::adapters::gui::conduccion::Campo;
use crate::adapters::gui::rotulos::Idioma;
use crate::adapters::gui::sesion_pvsyst::Sesion;
use crate::config::Paths;
use crate::error::{Error, Result};
use serde_json::{json, Value};

const ROTULOS: RotulosDimensionado = rotulos_dimensionado(Idioma::Es);

/// "Planned power" option and field of the pre-sizing help.
const OPCION_POTENCIA: Campo = Campo::opcion(1044, 73);
const POTENCIA_PLANEADA: Campo = Campo::edit(1080, 73);
const MODULOS_SERIE: Campo = Campo::edit(468, 486);
const CADENAS: Campo = Campo::edit(468, 514);
/// PVsyst sizes one sub-array per pre-sizing; several need one each.
const UN_SUBCONJUNTO: usize = 1;

/// Sizes the only sub-array of `proyecto`/`variante` to `kwp` with PVsyst.
pub(crate) fn dimensionar(
    paths: &Paths,
    proyecto: &str,
    variante: &str,
    kwp: f64,
) -> Result<Value> {
    let sesion = Sesion::abrir(paths)?;
    sesion.ejecutar(|s| {
        let p = ProyectoEnPvsyst::abrir(s, paths, proyecto, variante)?;
        let subconjuntos = contar(&p.leer_variante()?, "pvSubArray");
        if subconjuntos != UN_SUBCONJUNTO {
            return Err(Error::Invalid(format!(
                "La variante tiene {subconjuntos} subconjuntos: el pre-dimensionado de PVsyst trabaja uno a uno; pasa n_mod_serie y n_strings"
            )));
        }
        p.ventana.pulsar(ROTULOS.sistema)?;
        let sistema = s.esperar(ROTULOS.formulario_sistema)?;
        sistema.marcar(OPCION_POTENCIA)?;
        sistema.escribir(POTENCIA_PLANEADA, &format!("{kwp}"))?;
        let propuesta = json!({
            "n_mod_serie": sistema.leer(MODULOS_SERIE)?,
            "n_strings": sistema.leer(CADENAS)?,
        });
        sistema.pulsar(ROTULOS.ok)?;
        s.aceptar_mensajes();
        let raiz = p.guardar_variante()?;
        Ok(respuesta(&raiz, kwp, &propuesta, &s.mensajes()))
    })
}

fn respuesta(
    raiz: &crate::format::object::PvNode,
    kwp: f64,
    propuesta: &Value,
    avisos: &[String],
) -> Value {
    let sub = objeto(raiz, "pvSubArray");
    let campo = |k: &str| sub.and_then(|s| s.field(k)).map(str::to_string);
    let realizado = raiz
        .find_deep("PNomSizing")
        .and_then(|v| v.trim().parse::<f64>().ok());
    json!({
        "origen": "pvsyst",
        "metodo": "pre-dimensionado de PVsyst (ventana Sistema, potencia planeada)",
        "kwp_planeado": kwp,
        "realised_kwp": realizado,
        "propuesta_pvsyst": propuesta,
        "guardado": {
            "NModSerie": campo("NModSerie"),
            "NStringCh": campo("NStringCh"),
            "NInverter": campo("NInverter"),
            "PNomSizing": raiz.find_deep("PNomSizing"),
        },
        "avisos_pvsyst": avisos,
    })
}
