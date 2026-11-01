use super::key::{absorber, argumentos_clave, referencias, OFFSET_FNV};
use super::store::buscar_con_clave;
use super::*;
use serde_json::json;
use std::fs;
use std::path::PathBuf;

fn ws_temporal(nombre: &str) -> PathBuf {
    let dir = std::env::temp_dir().join("pvsyst_cache_test").join(nombre);
    let _ = fs::remove_dir_all(&dir);
    dir
}

fn cli(partes: &[&str]) -> Vec<String> {
    partes.iter().map(|s| (*s).to_owned()).collect()
}

#[test]
fn fnv1a_coincide_con_los_vectores_de_prueba() {
    assert_eq!(absorber(OFFSET_FNV, b""), 0xcbf2_9ce4_8422_2325);
    assert_eq!(absorber(OFFSET_FNV, b"a"), 0xaf63_dc4c_8601_ec8c);
}

#[test]
fn las_salidas_no_alteran_los_argumentos_de_la_clave() {
    let con_ocf = cli(&[
        "run-simulation",
        "-ocf:X.csv",
        "-rpf:X.pdf",
        "-p:a",
        "-v:VC0",
    ]);
    let con_otra_salida = cli(&["run-simulation", "-p:a", "-ocf", "otro.csv", "-v:VC0"]);
    assert_eq!(
        argumentos_clave(&con_ocf),
        argumentos_clave(&con_otra_salida)
    );
    assert_eq!(
        argumentos_clave(&con_ocf),
        cli(&["-p:a", "-v:VC0", "run-simulation"])
    );
}

#[test]
fn un_cambio_de_variante_cambia_los_argumentos_de_la_clave() {
    let vc0 = argumentos_clave(&cli(&["run-simulation", "-p:a", "-v:VC0"]));
    let vc1 = argumentos_clave(&cli(&["run-simulation", "-p:a", "-v:VC1"]));
    assert_ne!(vc0, vc1);
}

#[test]
fn referencias_solo_toman_sitios_meteo_y_componentes() {
    let texto = "MeteoFileName=Barrancabermeja.MET\r\nDesignTempMin=-10\r\nPanel=Foo.PAN\r\nComment=ver informe.pdf";
    let hallados = referencias(texto);
    assert_eq!(
        hallados,
        vec![
            ("MET".to_owned(), "Barrancabermeja.MET".to_owned()),
            ("PAN".to_owned(), "Foo.PAN".to_owned()),
        ]
    );
}

#[test]
fn guardar_y_buscar_copian_el_csv_y_marcan_cache() {
    let dir = ws_temporal("ida_y_vuelta");
    fs::create_dir_all(&dir).expect("directorio temporal");
    let origen = dir.join("salida.csv");
    fs::write(&origen, "a;b\n1;2\n").expect("csv de origen");
    guardar(
        &dir,
        "abc123",
        Some(&origen),
        None,
        &json!({"success": true}),
    )
    .expect("guardar");

    let destino = dir.join("copia.csv");
    let hallado = buscar(&dir, "abc123", Some(&destino), None)
        .expect("buscar")
        .expect("acierto");
    assert_eq!(hallado["cache"], json!(true));
    assert_eq!(hallado["success"], json!(true));
    assert_eq!(fs::read_to_string(&destino).expect("copia"), "a;b\n1;2\n");
    assert!(buscar(&dir, "abc123", None, Some(&dir.join("x.pdf")))
        .expect("buscar")
        .is_none());
    assert!(buscar(&dir, "otra", None, None).expect("buscar").is_none());
    let _ = fs::remove_dir_all(&dir);
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
                razon.contains("Referencia no encontrada"),
                "Reason should indicate reference not found: {razon}"
            );
            assert!(
                razon.contains("NoExiste.MET"),
                "Reason should mention file: {razon}"
            );
            // Look up in cache with unusable key should yield None (not a hit)
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

#[test]
fn should_reject_cache_key_when_variant_is_not_an_identifier() {
    let ws = tempfile::tempdir().expect("ws");
    let dir = ws.path().join("Projects");
    fs::create_dir_all(&dir).expect("Projects");
    fs::write(dir.join("Planta.PRJ"), "x").expect("prj");

    let res = clave(
        ws.path(),
        &cli(&["run-simulation", "-p:Planta.PRJ", "-v:../../x"]),
    );

    assert!(matches!(res, Err(crate::error::Error::Invalid(_))));
}

#[test]
fn should_treat_corrupt_cache_entry_as_a_miss() {
    let ws = tempfile::tempdir().expect("ws");
    let dir = crate::adapters::ledger::carpeta_cache(ws.path()).join("abc");
    fs::create_dir_all(&dir).expect("cache");
    fs::write(dir.join("meta.json"), "{ cortado").expect("meta");

    let res = buscar(ws.path(), "abc", None, None).expect("fallo de caché, no error");

    assert!(res.is_none());
}

#[test]
fn should_find_entry_written_by_guardar() {
    let ws = tempfile::tempdir().expect("ws");

    guardar(ws.path(), "k1", None, None, &json!({ "success": true })).expect("guardado");
    let hit = buscar(ws.path(), "k1", None, None).expect("buscar");

    assert!(hit.is_some());
}

/// Workspace with `Projects/Planta.PRJ`, its `VC0` and `Sites/S.SIT` holding `sitio`.
fn workspace_con_sitio(sitio: &str) -> tempfile::TempDir {
    let ws = tempfile::tempdir().expect("ws");
    let proyectos = ws.path().join("Projects");
    let sitios = ws.path().join("Sites");
    fs::create_dir_all(&proyectos).expect("Projects");
    fs::create_dir_all(&sitios).expect("Sites");
    fs::write(
        proyectos.join("Planta.PRJ"),
        "PVObject_=pvProject\nEnd of PVObject pvProject\n",
    )
    .expect("prj");
    fs::write(
        proyectos.join("Planta.VC0"),
        "PVObject_=pvVCalcul\nEnd of PVObject pvVCalcul\n",
    )
    .expect("vc0");
    fs::write(sitios.join("S.SIT"), sitio).expect("sit");
    ws
}

#[test]
fn should_change_cache_key_when_site_content_changes() {
    let args = cli(&["run-simulation", "-p:Planta.PRJ", "-v:VC0", "-s:S.SIT"]);
    let a = workspace_con_sitio("Latitude=4.6");
    let b = workspace_con_sitio("Latitude=10.4");

    let clave_a = clave(a.path(), &args).expect("clave a");
    let clave_b = clave(b.path(), &args).expect("clave b");

    assert!(matches!(clave_a, ClaveCache::Valida(_)), "{clave_a:?}");
    assert_ne!(clave_a, clave_b);
    let igual = workspace_con_sitio("Latitude=4.6");
    assert_eq!(clave(igual.path(), &args).expect("misma clave"), clave_a);
}
