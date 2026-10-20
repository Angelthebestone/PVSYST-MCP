//! HTTP GET through the Windows-bundled `curl.exe` (no HTTP client dependency).

use crate::error::{Error, Result};
use std::path::PathBuf;
use std::process::Command;

/// Path to the system `curl.exe`, falling back to the bare name on PATH.
fn curl_exe() -> PathBuf {
    std::env::var_os("SystemRoot")
        .map(|root| PathBuf::from(root).join("System32").join("curl.exe"))
        .filter(|p| p.exists())
        .unwrap_or_else(|| PathBuf::from("curl.exe"))
}

/// Fetch `url` and return the response body. Fails when curl cannot start,
/// exits non-zero, or returns an empty body.
pub(crate) fn http_get(url: &str) -> Result<String> {
    tracing::info!(url, "HTTP GET");
    let out = Command::new(curl_exe())
        .args(["-sS", "-L", "--max-time", "30", url])
        .output()
        .map_err(|e| Error::Network(format!("No se pudo ejecutar curl.exe: {e}")))?;
    if !out.status.success() {
        return Err(Error::Network(format!(
            "HTTP GET {url} falló: {}",
            String::from_utf8_lossy(&out.stderr).trim()
        )));
    }
    let body = String::from_utf8_lossy(&out.stdout).into_owned();
    if body.trim().is_empty() {
        return Err(Error::Network(format!(
            "HTTP GET {url} devolvió una respuesta vacía"
        )));
    }
    Ok(body)
}
