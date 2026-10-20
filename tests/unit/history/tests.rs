//! Behavioral and concurrency tests for the history module.

use super::escritura_atomica::limpiar_temporales_abandonados;
use super::snapshot::{civil, sorted_dirs};
use super::*;
use serde_json::json;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{Duration, SystemTime};

static TEST_COUNTER: AtomicU64 = AtomicU64::new(0);

pub(super) fn crear_workspace_temporal(name: &str) -> PathBuf {
    let n = TEST_COUNTER.fetch_add(1, Ordering::Relaxed);
    let d = std::env::temp_dir().join(format!("mcp_hist_{name}_{}_{n}", std::process::id()));
    let _ = fs::remove_dir_all(&d);
    fs::create_dir_all(&d).expect("tmp dir");
    d
}

pub(super) fn snap_dir(ws: &Path, id: &str) -> PathBuf {
    let d = ws.join("UserData").join("mcp_history").join(id);
    fs::create_dir_all(&d).expect("create snap dir");
    d
}

pub(super) fn escribir_manifiesto(dir: &Path, id: &str, archivos: &serde_json::Value) {
    let m = json!({ "id": id, "fecha": "2026-10-14T10:00:00.000Z", "motivo": "test", "archivos": archivos });
    fs::write(dir.join("manifest.json"), m.to_string()).expect("write manifest");
}

/// Restore confined to the workspace, as every tool call does by default.
pub(super) fn restaurar(ws: &Path, id: &str) -> crate::error::Result<serde_json::Value> {
    super::restaurar(ws, id, crate::config::Alcance::Workspace)
}

#[test]
fn should_calculate_civil_date_matching_known_instants() {
    assert_eq!(civil(0), (1970, 1, 1, 0, 0, 0));
    assert_eq!(civil(951_868_800), (2000, 3, 1, 0, 0, 0));
    assert_eq!(civil(1_709_251_199), (2024, 2, 29, 23, 59, 59));
}

#[test]
fn should_generate_unique_snapshot_ids_within_same_millisecond() {
    let ws = crear_workspace_temporal("ids");
    let f = ws.join("a.txt");
    fs::write(&f, "x").expect("write");
    let a = snapshot(&ws, std::slice::from_ref(&f), "a").expect("snap a");
    let b = snapshot(&ws, std::slice::from_ref(&f), "b").expect("snap b");
    assert_ne!(a, b);
    let _ = fs::remove_dir_all(&ws);
}

#[test]
fn should_restore_files_and_allow_undo_via_previous_snapshot() {
    let ws = crear_workspace_temporal("restore_undo");
    let kept = ws.join("kept.txt");
    let created = ws.join("created.txt");
    fs::write(&kept, "antes").expect("write");
    let id = snapshot(&ws, &[kept.clone(), created.clone()], "test").expect("snap");
    fs::write(&kept, "despues").expect("write");
    fs::write(&created, "nuevo").expect("write");

    let res = restaurar(&ws, &id).expect("restore");
    assert_eq!(fs::read_to_string(&kept).expect("read"), "antes");
    assert!(!created.exists());

    let previo = res["historial_id_previo"].as_str().expect("previo");
    restaurar(&ws, previo).expect("undo restore");
    assert_eq!(fs::read_to_string(&kept).expect("read"), "despues");
    assert_eq!(fs::read_to_string(&created).expect("read"), "nuevo");
    let _ = fs::remove_dir_all(&ws);
}

#[test]
fn should_replace_existing_file_atomically() {
    let ws = crear_workspace_temporal("atomic");
    let f = ws.join("v.txt");
    fs::write(&f, "old").expect("write");
    escribir_atomico(&f, b"new").expect("atomic");
    assert_eq!(fs::read(&f).expect("read"), b"new");
    assert!(!ws.join("v.txt.mcp_tmp").exists());
    let _ = fs::remove_dir_all(&ws);
}

#[test]
fn should_snapshot_only_when_file_exists_in_reemplazar() {
    let ws = crear_workspace_temporal("reemplazar");
    let f = ws.join("p.txt");
    assert_eq!(reemplazar(&ws, &f, b"uno", "t").expect("new"), None);
    let id = reemplazar(&ws, &f, b"dos", "t").expect("replace");
    assert_eq!(fs::read(&f).expect("read"), b"dos");
    restaurar(&ws, &id.expect("snapshot id")).expect("restore");
    assert_eq!(fs::read(&f).expect("read"), b"uno");
    let _ = fs::remove_dir_all(&ws);
}

#[test]
fn should_reject_path_like_snapshot_ids() {
    let ws = crear_workspace_temporal("badid");
    assert!(restaurar(&ws, "../x").is_err());
    assert!(restaurar(&ws, "a/b").is_err());
    assert!(restaurar(&ws, "a\\b").is_err());
    assert!(restaurar(&ws, "").is_err());
    let _ = fs::remove_dir_all(&ws);
}

#[test]
fn should_reject_restore_when_original_is_outside_workspace_and_leave_files_untouched() {
    let ws = crear_workspace_temporal("bad_orig");
    let outside_dir = crear_workspace_temporal("outside");
    let outside_file = outside_dir.join("secret.txt");
    fs::write(&outside_file, "intacto").expect("write outside");
    let inside_file = ws.join("local.txt");
    fs::write(&inside_file, "contenido_inicial").expect("write inside");

    let snap_dir = snap_dir(&ws, "snap_outside");
    fs::write(snap_dir.join("000.bin"), "nuevo_contenido").expect("write copia");

    escribir_manifiesto(
        &snap_dir,
        "snap_outside",
        &json!([{ "original": outside_file.to_string_lossy(), "copia": "000.bin", "existia": true }]),
    );

    let resultado = restaurar(&ws, "snap_outside");
    assert!(
        resultado.is_err(),
        "Debe rechazar original fuera del workspace"
    );
    assert_eq!(fs::read_to_string(&outside_file).expect("read"), "intacto");
    assert_eq!(
        fs::read_to_string(&inside_file).expect("read"),
        "contenido_inicial"
    );

    let dirs = sorted_dirs(&ws.join("UserData").join("mcp_history"));
    assert_eq!(
        dirs.len(),
        1,
        "No debe crearse snapshot previo si el manifiesto falla"
    );
    let _ = fs::remove_dir_all(&ws);
    let _ = fs::remove_dir_all(&outside_dir);
}

#[test]
fn should_reject_restore_when_copia_contains_parent_dir() {
    let ws = crear_workspace_temporal("bad_copia_dotdot");
    let inside_file = ws.join("file.txt");
    fs::write(&inside_file, "v1").expect("write inside");

    let snap_dir = snap_dir(&ws, "snap_copia");

    escribir_manifiesto(
        &snap_dir,
        "snap_copia",
        &json!([{ "original": inside_file.to_string_lossy(), "copia": "../secret.txt", "existia": true }]),
    );
    let resultado = restaurar(&ws, "snap_copia");
    assert!(resultado.is_err(), "Debe rechazar copia con '..'");
    assert_eq!(fs::read_to_string(&inside_file).expect("read"), "v1");
    let _ = fs::remove_dir_all(&ws);
}

#[test]
fn should_reject_restore_when_copia_is_absolute() {
    let ws = crear_workspace_temporal("bad_copia_abs");
    let inside_file = ws.join("file.txt");
    fs::write(&inside_file, "v1").expect("write inside");

    let snap_dir = snap_dir(&ws, "snap_copia_abs");

    let abs = if cfg!(windows) {
        "C:\\temp\\000.bin"
    } else {
        "/tmp/000.bin"
    };
    escribir_manifiesto(
        &snap_dir,
        "snap_copia_abs",
        &json!([{ "original": inside_file.to_string_lossy(), "copia": abs, "existia": true }]),
    );
    let resultado = restaurar(&ws, "snap_copia_abs");
    assert!(resultado.is_err(), "Debe rechazar copia absoluta");
    assert_eq!(fs::read_to_string(&inside_file).expect("read"), "v1");
    let _ = fs::remove_dir_all(&ws);
}

#[test]
fn should_reject_restore_when_manifest_fields_are_missing_or_invalid() {
    let ws = crear_workspace_temporal("missing_fields");
    let snap_dir = snap_dir(&ws, "snap_missing");

    escribir_manifiesto(
        &snap_dir,
        "snap_missing",
        &json!([{"original": "local.txt"}]),
    );

    assert!(restaurar(&ws, "snap_missing").is_err());
    let _ = fs::remove_dir_all(&ws);
}

#[test]
fn should_serialize_concurrent_atomic_writes_with_no_leftover_temps() {
    let ws = crear_workspace_temporal("concurrent_atomic");
    let destino = ws.join("concurrente.txt");
    fs::write(&destino, "inicial").expect("write initial");

    const NUM_HILOS: usize = 16;
    let mut handles = Vec::with_capacity(NUM_HILOS);

    for i in 0..NUM_HILOS {
        let dest = destino.clone();
        handles.push(std::thread::spawn(move || {
            let payload = format!("payload_del_hilo_{i:02}_{}", "X".repeat(1024));
            escribir_atomico(&dest, payload.as_bytes())
                .unwrap_or_else(|e| panic!("Error en hilo {i}: {e}"));
        }));
    }

    for h in handles {
        h.join().expect("hilo terminó con pánico");
    }

    let final_bytes = fs::read(&destino).expect("read destino final");
    let final_text = String::from_utf8(final_bytes).expect("utf8");
    assert!(
        final_text.starts_with("payload_del_hilo_"),
        "El archivo debe contener el payload completo de uno de los hilos"
    );
    assert_eq!(
        final_text.len(),
        "payload_del_hilo_00_".len() + 1024,
        "La longitud debe coincidir con un payload exacto"
    );

    let mut tmp_files = Vec::new();
    for entry in fs::read_dir(&ws).expect("read ws").flatten() {
        let name = entry.file_name().to_string_lossy().to_string();
        if name.contains(".mcp_tmp") {
            tmp_files.push(name);
        }
    }
    assert!(
        tmp_files.is_empty(),
        "No deben quedar temporales huérfanos: {tmp_files:?}"
    );

    let _ = fs::remove_dir_all(&ws);
}

#[test]
fn should_surface_corrupt_manifests_in_listar() {
    let ws = crear_workspace_temporal("listar_corruptos");
    let snap_good = snap_dir(&ws, "20261014_100000_000_0001");
    let snap_bad = snap_dir(&ws, "20261014_100000_000_0002");

    escribir_manifiesto(&snap_good, "20261014_100000_000_0001", &json!([]));
    fs::write(snap_bad.join("manifest.json"), "{ corrupt json").expect("write bad");

    let resultado = listar(&ws, 10);
    assert_eq!(resultado["total"].as_u64(), Some(2));
    let snapshots = resultado["snapshots"].as_array().expect("snapshots array");
    assert_eq!(snapshots.len(), 1);
    assert_eq!(snapshots[0]["id"], "20261014_100000_000_0001");

    let corruptos = resultado["corruptos"].as_array().expect("corruptos array");
    assert_eq!(corruptos.len(), 1);
    assert_eq!(corruptos[0]["dir"], "20261014_100000_000_0002");

    let _ = fs::remove_dir_all(&ws);
}

#[test]
fn should_clean_abandoned_temporaries_older_than_threshold() {
    let ws = crear_workspace_temporal("clean_tmp");
    let old_tmp = ws.join("salida.txt.mcp_tmp_9999_1_1");
    fs::write(&old_tmp, "abandonado").expect("write old tmp");

    limpiar_temporales_abandonados(&ws, SystemTime::now(), Duration::from_secs(0));

    assert!(
        !old_tmp.exists(),
        "El archivo temporal antiguo debió ser eliminado"
    );
    let _ = fs::remove_dir_all(&ws);
}
