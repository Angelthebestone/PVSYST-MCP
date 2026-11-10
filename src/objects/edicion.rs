//! Editing PVObject field definitions with snapshot-backed atomic writes.

use crate::args::{missing, req_str, workspace_of};
use crate::config::Paths;
use crate::error::{Error, Result};
use crate::format::accion::AccionCambio;
use crate::format::edit;
use crate::format::object;
use crate::history;
use crate::idioma::{Idioma, Texto};
use crate::objects::resolucion::resolve_file;
use serde_json::{json, Value};
use std::fs;
use std::path::Path;

const RELEER: Texto = Texto {
    es: "No se pudo releer '{selector}' para verificar la escritura",
    en: "Could not re-read '{selector}' to verify the write",
};
const NO_CONTIENE: Texto = Texto {
    es: "{archivo} no contiene lo escrito: restaura con historial",
    en: "{archivo} does not contain what was written: restore it from the history",
};
const ESCRITURA_NO_VERIFICADA: Texto = Texto {
    es: "Escritura no verificada en el campo '{clave}'",
    en: "Write not verified in field '{clave}'",
};
const CAMPOS_INEXISTENTES: Texto = Texto {
    es: "Campos que el objeto '{selector}' no tiene: {nuevos}. editar campos cambia valores existentes; pasa nuevo:true para añadir un campo nuevo, o corrige la clave",
    en: "Fields the object '{selector}' does not have: {nuevos}. editar campos changes existing values; pass nuevo:true to add a new field, or fix the key",
};

/// Re-parse only the edited object's lines and read back each changed key.
/// A readback that differs from the requested value, or an object that cannot be
/// found or parsed again, refuses the edit: nothing unverified is reported as done.
fn verificar_span(
    bytes: &[u8],
    selector: &str,
    changes: &[(String, Option<String>)],
) -> Result<serde_json::Map<String, Value>> {
    let no_verificable =
        || Error::Format(RELEER.texto_con(Idioma::de_entorno(), &[("selector", selector)]));
    let spans = edit::spans_of(bytes);
    let span = edit::find_span(&spans, selector)
        .and_then(|i| spans.get(i))
        .ok_or_else(no_verificable)?;
    let text = String::from_utf8_lossy(bytes);
    let lines: Vec<&str> = text.lines().collect();
    let slice = lines
        .get(span.open..=span.close.min(lines.len().saturating_sub(1)))
        .ok_or_else(no_verificable)?
        .join("\n");
    let root = object::parse_first(&slice).ok_or_else(no_verificable)?;
    let mut verified = serde_json::Map::new();
    for (k, nuevo) in changes {
        let leido = root.find_deep(k);
        comprobar_lectura(k, nuevo.as_deref(), leido)?;
        verified.insert(k.clone(), json!(leido));
    }
    Ok(verified)
}

/// The file as it is on disk after the write, which must be what was sent.
fn releer_escrito(path: &Path, enviado: &[u8]) -> Result<Vec<u8>> {
    let leido = crate::workspace::leer_bytes(path)?;
    if leido != enviado {
        return Err(Error::Format(NO_CONTIENE.texto_con(
            Idioma::de_entorno(),
            &[("archivo", &path.to_string_lossy())],
        )));
    }
    Ok(leido)
}

/// Relative tolerance of a Real48 round trip (39-bit mantissa, about 1.8e-12).
const TOLERANCIA_REAL48: f64 = 1e-9;

/// Legacy PAN values as they are on disk: the slots are decoded again from the bytes
/// read back and each must match the value asked for.
fn verificar_legacy(
    leido: &[u8],
    changes: &[(String, Option<String>)],
    outcomes: &[edit::ChangeOutcome],
) -> Result<serde_json::Map<String, Value>> {
    let (_, en_disco) = crate::components::pan::parchear_campos_legacy(leido, changes)?;
    let numero = |v: &Option<String>| v.as_deref().and_then(|t| t.parse::<f64>().ok());
    let mut verified = serde_json::Map::new();
    for (pedido, disco) in outcomes.iter().zip(&en_disco) {
        let (Some(p), Some(d)) = (numero(&pedido.new_value), numero(&disco.old_value)) else {
            return Err(Error::Format(
                ESCRITURA_NO_VERIFICADA.texto_con(Idioma::de_entorno(), &[("clave", &pedido.key)]),
            ));
        };
        if (p - d).abs() > TOLERANCIA_REAL48 * p.abs().max(1.0) {
            return Err(Error::Format(
                ESCRITURA_NO_VERIFICADA.texto_con(Idioma::de_entorno(), &[("clave", &pedido.key)]),
            ));
        }
        verified.insert(pedido.key.clone(), json!(disco.old_value));
    }
    Ok(verified)
}

/// A key the object does not have would be inserted as a new line, and the usual
/// cause is a typo (`FieldTitl` for `FieldTilt`): refuse it unless the caller asks
/// for a new field with `nuevo: true`.
pub(crate) fn exigir_campos_existentes(
    selector: &str,
    outcomes: &[edit::ChangeOutcome],
    args: &Value,
) -> Result<()> {
    if crate::args::flag(args, &["nuevo"]) {
        return Ok(());
    }
    let nuevos: Vec<&str> = outcomes
        .iter()
        .filter(|o| o.action == AccionCambio::Inserted)
        .map(|o| o.key.as_str())
        .collect();
    if nuevos.is_empty() {
        return Ok(());
    }
    Err(Error::Invalid(CAMPOS_INEXISTENTES.texto_con(
        Idioma::de_entorno(),
        &[("selector", selector), ("nuevos", &format!("{nuevos:?}"))],
    )))
}

fn comprobar_lectura(clave: &str, esperado: Option<&str>, leido: Option<&str>) -> Result<()> {
    let coincide = match esperado {
        None => leido.is_none(),
        Some(valor) => leido.map(str::trim) == Some(valor.trim()),
    };
    if coincide {
        Ok(())
    } else {
        Err(Error::Format(
            ESCRITURA_NO_VERIFICADA.texto_con(Idioma::de_entorno(), &[("clave", clave)]),
        ))
    }
}

pub(crate) fn editar_objeto(paths: &Paths, args: &Value) -> Result<Value> {
    let workspace = workspace_of(paths, args);
    let path = resolve_file(&workspace, args)?;
    let cambios = args
        .get("campos")
        .or_else(|| args.get("fields"))
        .and_then(|v| v.as_object())
        .ok_or_else(|| missing("campos", "object of key: value"))?;

    let changes = edit::changes_from_json(cambios)?;
    edit::reject_invalid_keys(&changes)?;

    let bytes =
        fs::read(&path).map_err(|e| Error::io(format!("Cannot read {}", path.display()), e))?;

    if crate::components::pan::es_pan_legacy(&bytes) {
        let selector = crate::args::opt_str(args, &["objeto"]).unwrap_or("pvModule");
        let (new_bytes, outcomes) =
            crate::components::pan::parchear_campos_legacy(&bytes, &changes)?;
        exigir_campos_existentes(selector, &outcomes, args)?;
        let historial_id =
            history::snapshot(&workspace, std::slice::from_ref(&path), "editar_objeto")?;
        history::escribir_atomico(&path, &new_bytes)?;
        let verified = verificar_legacy(&releer_escrito(&path, &new_bytes)?, &changes, &outcomes)?;
        return Ok(json!({
            "file": path.to_string_lossy(),
            "object": selector,
            "historial_id": historial_id,
            "changes": outcomes.iter().map(edit::ChangeOutcome::to_json).collect::<Vec<_>>(),
            "verified_after_write": verified,
        }));
    }

    let selector = req_str(args, &["objeto"], "e.g. \"pvSubArray\", \"role:SitePrj\"")?;

    let (new_bytes, outcomes) = edit::edit_fields(&bytes, selector, &changes)?;
    exigir_campos_existentes(selector, &outcomes, args)?;
    let verified = verificar_span(&new_bytes, selector, &changes)?;
    let historial_id = history::snapshot(&workspace, std::slice::from_ref(&path), "editar_objeto")?;
    history::escribir_atomico(&path, &new_bytes)?;
    // `verified` was read from these bytes before writing; the disk must now hold them.
    releer_escrito(&path, &new_bytes)?;
    Ok(json!({
        "file": path.to_string_lossy(),
        "object": selector,
        "historial_id": historial_id,
        "changes": outcomes.iter().map(edit::ChangeOutcome::to_json).collect::<Vec<_>>(),
        "verified_after_write": verified,
    }))
}

#[cfg(test)]
#[path = "../../tests/unit/objects/edicion.rs"]
mod tests;
