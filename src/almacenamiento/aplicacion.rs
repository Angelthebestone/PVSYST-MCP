//! `proyecto accion:almacenamiento`: configures battery storage on a grid
//! variant, inserting `pvGridStorage` from the catalogue when it is missing.

use crate::almacenamiento::dominio::{EntradaAlmacenamiento, ParametrosAlmacenamiento};
use crate::components::{resolve_component, ConsultaComponente};
use crate::config::Paths;
use crate::error::{Error, Result};
use crate::format::{edit, object};
use crate::idioma::{Idioma, Texto};
use crate::projects::anadir::catalogo::{aplicar_elemento, AlcanceElemento};
use crate::projects::lifecycle::variantes::VarianteAbierta;
use crate::projects::lifecycle::{abrir_variante, guardar_variante, respuesta_de_edicion};
use crate::projects::tipo_sistema::TipoSistema;
use serde_json::{json, Value};

/// PVsyst refuses the "Peak shaving" strategy when the variant defines a
/// self-consumption (a `pvLoad`), and only says so at simulation time.
const RECORTE_CON_CARGA: Texto = Texto {
    es: "PVsyst rechaza 'recorte_picos' cuando la variante define un autoconsumo (objeto pvLoad): elimínalo o usa estrategia 'autoconsumo'",
    en: "PVsyst refuses 'recorte_picos' when the variant defines a self-consumption (a pvLoad object): remove it or use estrategia 'autoconsumo'",
};

/// Storage inserted without the load the catalogue used to add silently.
const SIN_CARGA: Texto = Texto {
    es: "La variante no tiene objeto pvLoad: PVsyst necesita una carga para simular el autoconsumo. Añádela con proyecto accion:anadir elemento:carga",
    en: "The variant has no pvLoad object: PVsyst needs a load to simulate self-consumption. Add one with proyecto accion:anadir elemento:carga",
};

pub(crate) fn configurar_almacenamiento(paths: &Paths, args: &Value) -> Result<Value> {
    let parametros = ParametrosAlmacenamiento::nuevo(EntradaAlmacenamiento::de_args(args)?)?;
    let mut variante = abrir_variante(paths, args)?;
    TipoSistema::exigir(&variante.bytes, TipoSistema::Red)?;
    exigir_bateria(&variante, parametros.bateria())?;

    let bytes = std::mem::take(&mut variante.bytes);
    if parametros.es_recorte_picos() {
        exigir_recorte_sin_carga(&bytes)?;
    }
    if parametros.es_recorte_picos() && !tiene_plim_grid(&bytes) {
        return Err(Error::Invalid(
            "El recorte de picos usa PLimGrid como umbral: define antes proyecto accion:limite_inyeccion"
                .into(),
        ));
    }
    let (bytes, insertados) = if edit::find_span(&edit::spans_of(&bytes), "pvGridStorage").is_none()
    {
        let aplicado = aplicar_elemento(bytes, "baterias", AlcanceElemento::SoloElElemento)?;
        (aplicado.bytes, aplicado.insertados)
    } else {
        (bytes, Vec::new())
    };
    let (bytes, outcomes) = edit::editar_objetos(&bytes, &parametros.ediciones())?;
    let historial_id = guardar_variante(&variante, &bytes, "proyecto almacenamiento")?;

    let mut out = respuesta_de_edicion(
        &variante,
        &parametros.como_json(),
        (&outcomes, historial_id),
    );
    out.insert("insertados".into(), json!(insertados));
    out.insert("dimensionado".into(), parametros.criterio_dimensionado());
    if let Some(texto) = aviso_carga(&bytes, &parametros) {
        out.insert("aviso".into(), json!(texto));
    }
    Ok(out.into())
}

/// A battery PVsyst cannot find leaves a variant that fails at simulation time.
fn exigir_bateria(variante: &VarianteAbierta, bateria: &str) -> Result<()> {
    let consulta = ConsultaComponente::referencia("BTR", bateria);
    resolve_component(&variante.workspace, &consulta).map(|_| ())
}

fn exigir_recorte_sin_carga(bytes: &[u8]) -> Result<()> {
    if edit::find_span(&edit::spans_of(bytes), "pvLoad").is_none() {
        return Ok(());
    }
    Err(Error::Invalid(
        RECORTE_CON_CARGA
            .en_idioma(Idioma::de_entorno())
            .to_string(),
    ))
}

fn aviso_carga(bytes: &[u8], parametros: &ParametrosAlmacenamiento) -> Option<String> {
    if parametros.es_recorte_picos() || edit::find_span(&edit::spans_of(bytes), "pvLoad").is_some()
    {
        return None;
    }
    Some(SIN_CARGA.en_idioma(Idioma::de_entorno()).to_string())
}

fn tiene_plim_grid(bytes: &[u8]) -> bool {
    object::parse_first(&String::from_utf8_lossy(bytes))
        .and_then(|nodo| nodo.find_deep("PLimGrid").map(|v| !v.trim().is_empty()))
        .unwrap_or(false)
}
