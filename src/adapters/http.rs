//! HTTP GET through the Windows-bundled `curl.exe` (no HTTP client dependency).

use crate::error::{Error, Result};
use crate::idioma::{Idioma, Texto};
use std::io::Read;
use std::path::PathBuf;
use std::process::{Command, Stdio};

const LEER_RESPUESTA: Texto = Texto {
    es: "No se pudo leer la respuesta: {e}",
    en: "Could not read the response: {e}",
};
const RESPUESTA_GRANDE: Texto = Texto {
    es: "La respuesta supera el máximo de {max} bytes",
    en: "The response exceeds the maximum of {max} bytes",
};
const CURL_NO_ARRANCA: Texto = Texto {
    es: "No se pudo ejecutar curl.exe: {e}",
    en: "Could not run curl.exe: {e}",
};
const CURL_SIN_SALIDA: Texto = Texto {
    es: "curl.exe no abrió su salida",
    en: "curl.exe did not open its output",
};
const CURL_NO_TERMINA: Texto = Texto {
    es: "curl.exe no terminó: {e}",
    en: "curl.exe did not finish: {e}",
};
const GET_FALLO: Texto = Texto {
    es: "HTTP GET {url} falló: {detalle}",
    en: "HTTP GET {url} failed: {detalle}",
};
const GET_VACIO: Texto = Texto {
    es: "HTTP GET {url} devolvió una respuesta vacía",
    en: "HTTP GET {url} returned an empty response",
};

/// Path to the system `curl.exe`, falling back to the bare name on PATH.
fn curl_exe() -> PathBuf {
    std::env::var_os("SystemRoot")
        .map(|root| PathBuf::from(root).join("System32").join("curl.exe"))
        .filter(|p| p.exists())
        .unwrap_or_else(|| PathBuf::from("curl.exe"))
}

/// Largest body accepted. PVGIS hourly series run to a few MB; a server that keeps sending
/// must not fill the memory in the 30 s the request is allowed.
const MAX_CUERPO_BYTES: u64 = 64 * 1024 * 1024;

/// Reads `origen` to its end, failing as soon as it holds more than `max` bytes.
fn leer_acotado(origen: impl Read, max: u64) -> Result<Vec<u8>> {
    let mut cuerpo = Vec::new();
    origen.take(max + 1).read_to_end(&mut cuerpo).map_err(|e| {
        Error::Network(LEER_RESPUESTA.texto_con(Idioma::de_entorno(), &[("e", &e.to_string())]))
    })?;
    if cuerpo.len() as u64 > max {
        return Err(Error::Network(
            RESPUESTA_GRANDE.texto_con(Idioma::de_entorno(), &[("max", &max.to_string())]),
        ));
    }
    Ok(cuerpo)
}

/// Fetch `url` and return the response body. Fails when curl cannot start,
/// exits non-zero, returns an empty body or one over [`MAX_CUERPO_BYTES`].
pub(crate) fn http_get(url: &str) -> Result<String> {
    // The query carries the site coordinates: only the endpoint is logged.
    let endpoint = url.split_once('?').map_or(url, |(base, _)| base);
    tracing::info!(endpoint, "HTTP GET");
    let mut curl = Command::new(curl_exe())
        .args(["-sS", "-L", "--max-time", "30", url])
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|e| {
            Error::Network(
                CURL_NO_ARRANCA.texto_con(Idioma::de_entorno(), &[("e", &e.to_string())]),
            )
        })?;
    let Some(salida) = curl.stdout.take() else {
        return Err(Error::Network(
            CURL_SIN_SALIDA.en_idioma(Idioma::de_entorno()).to_string(),
        ));
    };
    let cuerpo = leer_acotado(salida, MAX_CUERPO_BYTES);
    if cuerpo.is_err() {
        // `kill` fails only when curl has already ended, which is what is wanted.
        let _ = curl.kill();
    }
    let estado = curl.wait_with_output().map_err(|e| {
        Error::Network(CURL_NO_TERMINA.texto_con(Idioma::de_entorno(), &[("e", &e.to_string())]))
    })?;
    let cuerpo = cuerpo?;
    if !estado.status.success() {
        return Err(Error::Network(GET_FALLO.texto_con(
            Idioma::de_entorno(),
            &[
                ("url", url),
                ("detalle", String::from_utf8_lossy(&estado.stderr).trim()),
            ],
        )));
    }
    let body = String::from_utf8_lossy(&cuerpo).into_owned();
    if body.trim().is_empty() {
        return Err(Error::Network(
            GET_VACIO.texto_con(Idioma::de_entorno(), &[("url", url)]),
        ));
    }
    Ok(body)
}

#[cfg(test)]
#[path = "../../tests/unit/adapters/http.rs"]
mod tests;
