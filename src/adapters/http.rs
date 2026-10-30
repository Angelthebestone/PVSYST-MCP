//! HTTP GET through the Windows-bundled `curl.exe` (no HTTP client dependency).

use crate::error::{Error, Result};
use std::io::Read;
use std::path::PathBuf;
use std::process::{Command, Stdio};

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
    origen
        .take(max + 1)
        .read_to_end(&mut cuerpo)
        .map_err(|e| Error::Network(format!("No se pudo leer la respuesta: {e}")))?;
    if cuerpo.len() as u64 > max {
        return Err(Error::Network(format!(
            "La respuesta supera el máximo de {max} bytes"
        )));
    }
    Ok(cuerpo)
}

/// Fetch `url` and return the response body. Fails when curl cannot start,
/// exits non-zero, returns an empty body or one over [`MAX_CUERPO_BYTES`].
pub(crate) fn http_get(url: &str) -> Result<String> {
    tracing::info!(url, "HTTP GET");
    let mut curl = Command::new(curl_exe())
        .args(["-sS", "-L", "--max-time", "30", url])
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|e| Error::Network(format!("No se pudo ejecutar curl.exe: {e}")))?;
    let Some(salida) = curl.stdout.take() else {
        return Err(Error::Network("curl.exe no abrió su salida".into()));
    };
    let cuerpo = leer_acotado(salida, MAX_CUERPO_BYTES);
    if cuerpo.is_err() {
        // `kill` fails only when curl has already ended, which is what is wanted.
        let _ = curl.kill();
    }
    let estado = curl
        .wait_with_output()
        .map_err(|e| Error::Network(format!("curl.exe no terminó: {e}")))?;
    let cuerpo = cuerpo?;
    if !estado.status.success() {
        return Err(Error::Network(format!(
            "HTTP GET {url} falló: {}",
            String::from_utf8_lossy(&estado.stderr).trim()
        )));
    }
    let body = String::from_utf8_lossy(&cuerpo).into_owned();
    if body.trim().is_empty() {
        return Err(Error::Network(format!(
            "HTTP GET {url} devolvió una respuesta vacía"
        )));
    }
    Ok(body)
}

#[cfg(test)]
#[path = "../../tests/unit/adapters/http.rs"]
mod tests;
