//! Behavioral tests for admin tools: path resolution, archive validation and boundaries.

use super::archive::validar_listado_zip;
use super::proyecto::exportar_proyecto;
use super::workspace::exportar_workspace;
use crate::config::Paths;
use serde_json::json;
use std::fs;

fn crear_workspace_temporal() -> (tempfile::TempDir, Paths) {
    let tmp = tempfile::tempdir().expect("create temp dir");
    let ws = tmp.path().join("PVsyst8.1_Data");
    fs::create_dir_all(ws.join("Projects")).expect("create Projects");
    fs::create_dir_all(ws.join("ComposPV")).expect("create ComposPV");
    let mut paths = Paths::resolve();
    paths.workspace = ws;
    (tmp, paths)
}

#[test]
fn should_reject_export_outside_workspace_by_default() {
    let (tmp, paths) = crear_workspace_temporal();
    let prj = paths.workspace.join("Projects").join("Demo.PRJ");
    fs::write(&prj, "PVObject_=pvProject\n").expect("write prj");

    let dest_fuera = tmp.path().join("outside.zip");
    let args = json!({
        "proyecto": "Demo",
        "destino": dest_fuera.to_string_lossy()
    });

    let res = exportar_proyecto(&paths, &args);
    assert!(
        res.is_err(),
        "Must reject export outside workspace by default"
    );
    let err_str = res.unwrap_err().to_string();
    assert!(
        err_str.contains("Ruta fuera del workspace"),
        "Expected boundary rejection error, got: {err_str}"
    );
}

#[test]
fn should_allow_export_outside_workspace_with_explicit_flag() {
    let (tmp, paths) = crear_workspace_temporal();
    let prj = paths.workspace.join("Projects").join("Demo.PRJ");
    fs::write(&prj, "PVObject_=pvProject\n").expect("write prj");

    let dest_fuera = tmp.path().join("outside.zip");
    let args = json!({
        "proyecto": "Demo",
        "destino": dest_fuera.to_string_lossy(),
        "permitir_fuera_del_workspace": true
    });

    let res = exportar_proyecto(&paths, &args);
    if let Err(err) = res {
        let s = err.to_string();
        assert!(
            !s.contains("Ruta fuera del workspace"),
            "Boundary check must pass when flag is given; failed with: {s}"
        );
    }
}

#[test]
fn should_reject_workspace_export_outside_workspace_by_default() {
    let (tmp, paths) = crear_workspace_temporal();
    let dest_fuera = tmp.path().join("outside_ws.zip");
    let args = json!({
        "destino": dest_fuera.to_string_lossy()
    });

    let res = exportar_workspace(&paths, &args);
    assert!(
        res.is_err(),
        "Must reject workspace export outside workspace by default"
    );
    let err_str = res.unwrap_err().to_string();
    assert!(
        err_str.contains("Ruta fuera del workspace"),
        "Expected boundary error, got: {err_str}"
    );
}

/// `tar -tvf`-style line for `nombre` with entry type `tipo` and `tamano` bytes.
fn linea_tv(tipo: char, tamano: u64, nombre: &str) -> String {
    format!("{tipo}rw-rw-rw-  0 0      0 {tamano:>11} oct. 14 17:20 {nombre}")
}

fn listado(entradas: &[(char, u64, &str)]) -> (String, String) {
    let nombres = entradas.iter().map(|e| e.2).collect::<Vec<_>>().join("\n");
    let detalle = entradas
        .iter()
        .map(|e| linea_tv(e.0, e.1, e.2))
        .collect::<Vec<_>>()
        .join("\n");
    (nombres, detalle)
}

#[test]
fn should_reject_zip_entries_escaping_the_destination() {
    for malo in [
        "Projects/../../Windows/System32/calc.exe",
        "/etc/shadow",
        r"C:\x.PRJ",
    ] {
        let (n, d) = listado(&[('-', 1, "Projects/Normal.PRJ"), ('-', 1, malo)]);
        let err = validar_listado_zip(&n, &d).expect_err(malo).to_string();
        assert!(err.contains("Entrada insegura"), "{malo}: {err}");
    }
}

#[test]
fn should_reject_links_and_oversized_archives() {
    let (n, d) = listado(&[('l', 0, "Projects/enlace.PRJ")]);
    assert!(validar_listado_zip(&n, &d).is_err(), "symlink");
    let (n, d) = listado(&[('h', 0, "Projects/duro.PRJ")]);
    assert!(validar_listado_zip(&n, &d).is_err(), "hardlink");
    let (n, d) = listado(&[('-', u64::MAX / 2, "a.bin"), ('-', u64::MAX / 2, "b.bin")]);
    assert!(validar_listado_zip(&n, &d).is_err(), "bomba");
}

#[test]
fn should_accept_safe_zip_entries_with_spaces() {
    let (n, d) = listado(&[
        ('d', 0, "Projects/"),
        ('-', 12, "Projects/Mi Demo.PRJ"),
        ('-', 5, "ComposPV/PVModules/Panel.PAN"),
    ]);
    assert!(validar_listado_zip(&n, &d).is_ok());
}

#[test]
fn should_keep_backup_and_name_it_when_restore_after_failed_import_fails() {
    let tmp = tempfile::tempdir().expect("create temp dir");
    let respaldo = tmp.path().join("respaldo.PRJ");
    fs::write(&respaldo, "original").expect("write backup");
    let objetivo = tmp.path().join("Demo.PRJ");
    fs::create_dir_all(objetivo.join("ocupado")).expect("non-empty target dir");

    let fallo = std::io::Error::other("rename failed");
    let err = super::importacion::error_de_publicacion(fallo, &objetivo, Some(&respaldo));

    assert!(err.to_string().contains(&respaldo.display().to_string()));
    assert_eq!(
        fs::read_to_string(&respaldo).expect("backup kept"),
        "original"
    );
}

#[test]
fn should_restore_backup_when_import_fails_and_target_is_free() {
    let tmp = tempfile::tempdir().expect("create temp dir");
    let respaldo = tmp.path().join("respaldo.PRJ");
    fs::write(&respaldo, "original").expect("write backup");
    let objetivo = tmp.path().join("Demo.PRJ");

    let fallo = std::io::Error::other("rename failed");
    let err = super::importacion::error_de_publicacion(fallo, &objetivo, Some(&respaldo));

    assert!(!err.to_string().contains("respaldo.PRJ"));
    assert_eq!(fs::read_to_string(&objetivo).expect("restored"), "original");
}

fn lineas_sinteticas(n: usize) -> impl Iterator<Item = std::io::Result<String>> {
    (0..n).map(|i| Ok(format!("Projects/P{i}.PRJ")))
}

#[test]
fn should_join_listing_when_inside_limits() {
    let limites = super::archive::LimitesListado::nuevo(1024, 3);
    let texto = super::archive::listado_dentro_de_limites(lineas_sinteticas(3), limites)
        .expect("inside limits");
    assert_eq!(texto.lines().count(), 3);
}

#[test]
fn should_reject_listing_when_it_has_more_lines_than_allowed() {
    let limites = super::archive::LimitesListado::nuevo(1024 * 1024, 3);
    let err = super::archive::listado_dentro_de_limites(lineas_sinteticas(4), limites)
        .expect_err("too many lines");
    assert!(err.to_string().contains("excede los límites"));
}

#[test]
fn should_reject_listing_when_it_exceeds_the_byte_budget() {
    let limites = super::archive::LimitesListado::nuevo(20, 100);
    let err = super::archive::listado_dentro_de_limites(lineas_sinteticas(2), limites)
        .expect_err("too many bytes");
    assert!(err.to_string().contains("excede los límites"));
}

#[test]
fn should_validate_a_real_small_zip() {
    let tmp = tempfile::tempdir().expect("create temp dir");
    let origen = tmp.path().join("origen");
    fs::create_dir_all(origen.join("Projects")).expect("create Projects");
    fs::write(origen.join("Projects").join("Demo.PRJ"), "x").expect("write prj");
    let zip = tmp.path().join("ws.zip");
    super::archive::zip_into(&zip, &origen, &["Projects".into()]).expect("zip");

    super::archive::validar_entradas_zip(&zip).expect("small zip is valid");
}
