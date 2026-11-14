use super::key::ClaveCache;
use super::*;
use std::fs;
use std::path::{Path, PathBuf};

fn ws_temporal(nombre: &str) -> PathBuf {
    let dir = std::env::temp_dir().join("pvsyst_cache_test").join(nombre);
    let _ = fs::remove_dir_all(&dir);
    dir
}

fn cli(partes: &[&str]) -> Vec<String> {
    partes.iter().map(|s| (*s).to_owned()).collect()
}

#[test]
fn should_mark_key_unusable_when_referenced_file_is_missing_or_unreadable() {
    let dir = ws_temporal("ref_invalida");
    let prj_dir = dir.join("Projects");
    fs::create_dir_all(&prj_dir).expect("crear Projects");
    fs::write(
        prj_dir.join("TestPrj.PRJ"),
        "PVObject_=pvProject\nMeteoFileName=NoExiste.MET\n",
    )
    .expect("escribir PRJ");

    let args = cli(&["run-simulation", "-p:TestPrj"]);
    let resultado_clave = clave(&dir, &args).expect("calculo de clave");

    match resultado_clave {
        ClaveCache::Inutilizable { razon } => {
            assert!(
                razon.contains("NoExiste.MET"),
                "Reason should mention file: {razon}"
            );
            // A cache lookup with an unusable key must never report a hit.
            let hit = buscar_con_clave(&dir, &ClaveCache::Inutilizable { razon }, None, None)
                .expect("buscar con clave");
            assert!(hit.is_none(), "Unusable key must never hit cache");
        }
        ClaveCache::Valida(k) => {
            panic!("Key should not be valid when referenced file is missing: {k}");
        }
    }
    let _ = fs::remove_dir_all(&dir);
}

/// A lookup that treats an unusable key as a miss, as `preparar` does.
fn buscar_con_clave(
    workspace: &Path,
    clave: &ClaveCache,
    ocf: Option<&Path>,
    rpf: Option<&Path>,
) -> crate::error::Result<Option<serde_json::Value>> {
    match clave {
        ClaveCache::Valida(k) => super::buscar(workspace, k, ocf, rpf),
        ClaveCache::Inutilizable { .. } => Ok(None),
    }
}
