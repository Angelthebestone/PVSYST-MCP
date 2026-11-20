//! The destination folder the import publishes into, refused while it holds
//! files and the caller did not ask to overwrite them.

use crate::args::flag;
use crate::config::Paths;
use crate::error::{Error, Result};
use crate::idioma::Texto;
use serde_json::Value;
use std::path::Path;

const DESTINO_NO_VACIO: Texto = Texto {
    es: "La carpeta destino no está vacía: {destino} (pasa sobrescribir:true)",
    en: "The destination folder is not empty: {destino} (pass sobrescribir:true)",
};
const NO_LEER_DESTINO: Texto = Texto {
    es: "No se pudo leer el destino {destino}",
    en: "Could not read the destination {destino}",
};

/// Refuses a destination that holds files unless the caller asked to overwrite it.
pub(super) fn exigir_destino_libre(paths: &Paths, destino: &Path, args: &Value) -> Result<()> {
    let no_vacia = match destino.read_dir() {
        Ok(mut d) => d.next().is_some(),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => false,
        Err(e) => {
            return Err(Error::io(
                NO_LEER_DESTINO.texto_con(paths.idioma, &[("destino", &destino.to_string_lossy())]),
                e,
            ))
        }
    };
    if no_vacia && !flag(args, &["sobrescribir"]) {
        return Err(Error::Invalid(DESTINO_NO_VACIO.texto_con(
            paths.idioma,
            &[("destino", &destino.to_string_lossy())],
        )));
    }
    Ok(())
}
