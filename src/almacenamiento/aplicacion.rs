//! `proyecto accion:almacenamiento`: configures battery storage on a grid
//! variant, inserting `pvGridStorage` from the catalogue when it is missing.

use crate::almacenamiento::dominio::{EntradaAlmacenamiento, ParametrosAlmacenamiento};
use crate::components::{resolve_component, ConsultaComponente};
use crate::config::Paths;
use crate::error::{Error, Result};
use crate::format::{edit, object};
use crate::projects::lifecycle::{abrir_variante, guardar_variante};
use crate::projects::tipo_sistema::TipoSistema;
use serde_json::{json, Value};

pub(crate) fn configurar_almacenamiento(paths: &Paths, args: &Value) -> Result<Value> {
    let parametros = ParametrosAlmacenamiento::nuevo(EntradaAlmacenamiento::de_args(args)?)?;
    let mut variante = abrir_variante(paths, args)?;
    TipoSistema::exigir(&variante.bytes, TipoSistema::Red)?;

    let bytes = std::mem::take(&mut variante.bytes);
    let bytes = if edit::find_span(&edit::spans_of(&bytes), "pvGridStorage").is_none() {
        crate::projects::anadir::catalogo::aplicar_elemento(bytes, "baterias")?.bytes
    } else {
        bytes
    };
    let (bytes, outcomes) = edit::editar_objetos(&bytes, &parametros.ediciones())?;
    if parametros.es_recorte_picos() && !tiene_plim_grid(&bytes) {
        return Err(Error::Invalid(
            "El recorte de picos usa PLimGrid como umbral: define antes proyecto accion:limite_inyeccion"
                .into(),
        ));
    }
    let historial_id = guardar_variante(&variante, &bytes, "proyecto almacenamiento")?;

    let aviso = aviso_bateria(&variante, parametros.bateria());
    let mut out = json!({
        "proyecto": variante.stem,
        "variante": variante.variante,
        "archivo": variante.vpath.display().to_string(),
        "parametros": parametros.como_json(),
        "cambios": outcomes.iter().map(edit::ChangeOutcome::to_json).collect::<Vec<_>>(),
        "historial_id": historial_id,
    });
    if let Some(texto) = aviso {
        out["aviso"] = json!(texto);
    }
    Ok(out)
}

fn tiene_plim_grid(bytes: &[u8]) -> bool {
    object::parse_first(&String::from_utf8_lossy(bytes))
        .and_then(|nodo| nodo.find_deep("PLimGrid").map(|v| !v.trim().is_empty()))
        .unwrap_or(false)
}

fn aviso_bateria(
    variante: &crate::projects::lifecycle::variantes::VarianteAbierta,
    bateria: &str,
) -> Option<String> {
    let consulta = ConsultaComponente::referencia("BTR", bateria);
    resolve_component(&variante.workspace, &consulta)
        .is_err()
        .then(|| format!("PVsyst debe conocer la batería {bateria}"))
}
