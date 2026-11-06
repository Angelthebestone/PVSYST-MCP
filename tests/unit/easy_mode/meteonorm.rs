use super::*;

/// A fake PVsyst installation folder, with or without Meteonorm data.
fn instalacion(dir: &Path, con_meteonorm: bool) -> Vec<PathBuf> {
    if con_meteonorm {
        let datos = dir.join(CARPETA_DATOS);
        std::fs::create_dir_all(&datos).expect("DataMN9");
        std::fs::write(datos.join(BASE_ESTACIONES), b"stub").expect("station db");
    }
    vec![dir.to_path_buf()]
}

#[test]
fn should_reject_synthetic_weather_when_meteonorm_is_missing() {
    // Given: a PVsyst installation with no Meteonorm data
    let dir = tempfile::tempdir().expect("dir");
    let bases = instalacion(dir.path(), false);

    // When
    let err = exigir_si_sintetica(Some(&bases), true).expect_err("sin Meteonorm");

    // Then: the cause and the way out, before any trial execution is spent
    let msg = err.to_string();
    assert!(msg.contains("Meteonorm"), "msg={msg}");
    assert!(msg.contains("station.db.enc"), "msg={msg}");
    assert!(msg.contains("synthetic_weather:false"), "msg={msg}");
}

#[test]
fn should_accept_synthetic_weather_when_meteonorm_is_installed() {
    let dir = tempfile::tempdir().expect("dir");
    let bases = instalacion(dir.path(), true);
    assert!(exigir_si_sintetica(Some(&bases), true).is_ok());
}

#[test]
fn should_not_judge_meteonorm_when_no_pvsyst_installation_is_visible() {
    // Given: no installation to derive the Meteonorm folder from
    let dir = tempfile::tempdir().expect("dir");
    let paths = Paths {
        gui_exe: dir.path().join("missing").join("PVsyst.exe"),
        ..Paths::resolve()
    };

    // Then: the run itself reports what it finds
    assert!(bases_de_instalacion(&paths).is_none());
    assert!(exigir_si_sintetica(None, true).is_ok());
    assert!(exigir_si_sintetica(None, false).is_ok());
}

#[test]
fn should_ignore_meteonorm_when_the_run_does_not_ask_for_synthetic_weather() {
    let dir = tempfile::tempdir().expect("dir");
    let bases = instalacion(dir.path(), false);
    assert!(exigir_si_sintetica(Some(&bases), false).is_ok());
}
