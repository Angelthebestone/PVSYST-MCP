//! `entorno` answers: the preferences block must carry the keyed values the
//! `vista` contract promises, not an empty list.

use super::cliente_mcp::{tool_text, Client};
use serde_json::json;
use std::fs;
use std::path::Path;

/// A `PVsyst.ini` under a private `LOCALAPPDATA`, as PVsyst writes it (UTF-16 LE).
fn local_app_data_con_ini() -> tempfile::TempDir {
    let local = tempfile::tempdir().expect("LOCALAPPDATA temporal");
    let admin = local.path().join("PVsyst").join("8.1").join("Admin");
    fs::create_dir_all(&admin).expect("carpeta Admin");
    // More sections than `vista` reads as values: the reported symptom is an empty list.
    let texto = "[Preferences]\r\nIdioma=Español\r\n[General]\r\nPuerto=80\r\n[Web]\r\nPuerto=8080\r\n\
                 [A]\r\nK=1\r\n[B]\r\nK=1\r\n[C]\r\nK=1\r\n[D]\r\nK=1\r\n[E]\r\nK=1\r\n\
                 [F]\r\nK=1\r\n[G]\r\nK=1\r\n[H]\r\nK=1\r\n[I]\r\nK=1\r\n";
    let mut bytes = vec![0xFF, 0xFE];
    for unidad in texto.encode_utf16() {
        bytes.extend_from_slice(&unidad.to_le_bytes());
    }
    fs::write(admin.join("PVsyst.ini"), bytes).expect("PVsyst.ini");
    local
}

fn servidor(local: &Path) -> Client {
    let ws = local.join("workspace");
    fs::create_dir_all(&ws).expect("workspace");
    let mut c = Client::start_env(
        Some(&ws.to_string_lossy()),
        &[("LOCALAPPDATA", &local.to_string_lossy())],
    );
    c.initialize();
    c
}

#[test]
fn should_report_the_section_count_as_a_vista_field_when_reading_preferences() {
    let local = local_app_data_con_ini();
    let mut c = servidor(local.path());

    let data = tool_text(&c.call(20, "entorno", json!({ "accion": "preferencias" })));

    let campos = data["vista"]["campos"]
        .as_array()
        .unwrap_or_else(|| panic!("sin vista.campos: {data}"));
    assert!(
        campos
            .iter()
            .any(|c| c["clave"] == json!("n_secciones") && c["valor"] == json!(12)),
        "vista.campos nombra el número de secciones: {campos:?}"
    );
}
