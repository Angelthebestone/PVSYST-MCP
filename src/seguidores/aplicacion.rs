//! `proyecto accion:seguidor`: converts one fixed-orientation variant to a
//! one-axis tracker by replacing its orientation and shading objects.

use crate::config::Paths;
use crate::error::{Error, Result};
use crate::format::edit::{self, EdicionObjeto};
use crate::format::span::tstart;
use crate::format::{insert, object};
use crate::projects::lifecycle::{abrir_variante, guardar_variante, respuesta_de_edicion};
use crate::projects::tipo_sistema::TipoSistema;
use crate::seguidores::dominio::{cambios_orientacion, EntradaSeguidor, ParametrosSeguidor};
use crate::workspace::leer_bytes;
use serde_json::{json, Value};

const PLANTILLA_SEGUIDOR: &str = "_DEMO_Tracking.VC0";

/// Convert the variant to `UnlimTracking` and save it with history.
pub(crate) fn convertir_a_seguidor(paths: &Paths, args: &Value) -> Result<Value> {
    let parametros = ParametrosSeguidor::nuevo(EntradaSeguidor::de_args(args)?)?;
    let variante = abrir_variante(paths, args)?;
    TipoSistema::exigir(&variante.bytes, TipoSistema::Red)?;
    exigir_una_orientacion(&variante.bytes)?;
    rechazar_escena_3d(&variante.bytes)?;
    rechazar_bifacial(&variante.bytes)?;
    let no_orient = no_orient(&variante.bytes)?;

    let plantilla = leer_plantilla()?;
    let bytes = insert::reemplazar_objeto(
        &variante.bytes,
        "pvOrient",
        &insert::extraer_objeto(&plantilla, "pvOrient")?,
    )?;
    let bytes = insert::reemplazar_objeto(
        &bytes,
        "pvShading",
        &insert::extraer_objeto(&plantilla, "pvShading")?,
    )?;
    let cambios = cambios_orientacion(&parametros, &no_orient);
    let (bytes, outcomes) = edit::editar_objetos(
        &bytes,
        &[EdicionObjeto {
            selector: "pvOrient".into(),
            cambios,
        }],
    )?;
    let historial_id = guardar_variante(&variante, &bytes, "proyecto seguidor")?;

    let mut out = respuesta_de_edicion(
        &variante,
        &parametros.como_json(),
        (&outcomes, historial_id),
    );
    out.insert("tipo".into(), json!("UnlimTracking"));
    Ok(out.into())
}

fn leer_plantilla() -> Result<Vec<u8>> {
    let ruta = crate::config::proyectos_demo_ro().join(PLANTILLA_SEGUIDOR);
    leer_bytes(&ruta).map_err(|e| {
        Error::NotFound(format!(
            "Plantilla de seguidores no disponible en {}: {e}",
            ruta.display()
        ))
    })
}

fn texto_de_objeto(bytes: &[u8], selector: &str) -> Option<String> {
    insert::extraer_objeto(bytes, selector)
        .ok()
        .map(|bloque| String::from_utf8_lossy(&bloque).into_owned())
}

fn exigir_una_orientacion(bytes: &[u8]) -> Result<()> {
    let declaradas = String::from_utf8_lossy(bytes)
        .lines()
        .find_map(|linea| {
            tstart(linea)
                .strip_prefix("Orientations, list of=")?
                .split_whitespace()
                .next()?
                .parse::<usize>()
                .ok()
        })
        .ok_or_else(|| {
            Error::Invalid("La variante no declara 'Orientations, list of=...'".into())
        })?;
    let objetos = edit::spans_of(bytes)
        .iter()
        .filter(|span| span.ty.eq_ignore_ascii_case("pvOrient"))
        .count();
    if declaradas == 1 && objetos == 1 {
        return Ok(());
    }
    Err(Error::Invalid(
        "Solo se convierten variantes con una orientación".into(),
    ))
}

fn rechazar_escena_3d(bytes: &[u8]) -> Result<()> {
    let tiene = texto_de_objeto(bytes, "pvShading").is_some_and(|t| t.contains("ListeObjets"));
    if !tiene {
        return Ok(());
    }
    Err(Error::Invalid(
        "La variante tiene escena 3D de sombras; los seguidores 3D se crean desde plantilla: \
proyecto crear plantilla:_DEMO_Tracking variante:VC2 (o _DEMO_Utility VC7-VCD)"
            .into(),
    ))
}

fn rechazar_bifacial(bytes: &[u8]) -> Result<()> {
    if edit::find_span(&edit::spans_of(bytes), "pvBiFacialSheds").is_none() {
        return Ok(());
    }
    Err(Error::Invalid(
        "Quita el modelo bifacial de mesas antes de convertir (o usa la plantilla _DEMO_Utility.VC6)"
            .into(),
    ))
}

fn no_orient(bytes: &[u8]) -> Result<String> {
    let bloque = texto_de_objeto(bytes, "pvOrient")
        .ok_or_else(|| Error::NotFound("La variante no tiene pvOrient".into()))?;
    object::parse_first(&bloque)
        .and_then(|nodo| nodo.find_deep("NoOrient").map(str::to_string))
        .ok_or_else(|| Error::NotFound("pvOrient no declara NoOrient".into()))
}
