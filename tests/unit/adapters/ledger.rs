use super::*;

const BANNER: &str =
    "PVsystCLI is running in TRIAL mode, remaining days: 58, remaining executions: 223";

/// A private workspace that is removed when the test ends.
fn ws_temporal() -> tempfile::TempDir {
    tempfile::tempdir().expect("workspace temporal")
}

#[test]
fn banner_trial_da_dias_y_ejecuciones() {
    assert_eq!(parsear_banner(BANNER), Some((Some(58), 223)));
}

#[test]
fn salida_sin_banner_trial_no_tiene_contadores() {
    assert_eq!(parsear_banner("Simulation done in 3min 2sec"), None);
}

#[test]
fn registrar_descuenta_la_ejecucion_usada() {
    let tmp = ws_temporal();
    let dir = tmp.path();
    let costo = registrar(dir, BANNER, 1);
    assert_eq!(costo["restantes"], json!(222));
    assert_eq!(estado(dir).map(|e| e.restantes), Some(Some(222)));
}

#[test]
fn sin_banner_se_registra_licencia() {
    let tmp = ws_temporal();
    let dir = tmp.path();
    registrar(dir, "Simulation done in 1min", 1);
    assert_eq!(estado(dir).map(|e| e.modo), Some("licencia".to_owned()));
}

#[test]
fn confirmacion_solo_en_trial_con_varias_ejecuciones() {
    let tmp = ws_temporal();
    let dir = tmp.path();
    let plan = vec!["barrido".to_owned()];
    assert!(requiere_confirmacion(dir, &json!({}), 3, &plan).is_none());
    registrar(dir, BANNER, 0);
    let retencion =
        requiere_confirmacion(dir, &json!({}), 3, &plan).expect("debe pedir confirmación en trial");
    assert_eq!(retencion["requiere_confirmacion"], json!(true));
    assert!(requiere_confirmacion(dir, &json!({}), 1, &plan).is_none());
    assert!(requiere_confirmacion(dir, &json!({"confirmar": true}), 3, &plan).is_none());
}

#[test]
fn should_ask_for_confirmation_when_ledger_is_corrupt() {
    let tmp = ws_temporal();
    let dir = tmp.path();
    fs::create_dir_all(carpeta_cache(dir)).expect("cache");
    fs::write(ruta(dir), "{ no es json").expect("ledger roto");
    let plan = vec!["barrido".to_owned()];

    let retencion =
        requiere_confirmacion(dir, &json!({}), 3, &plan).expect("debe pedir confirmación");

    assert_eq!(retencion["requiere_confirmacion"], json!(true));
    assert!(requiere_confirmacion(dir, &json!({"confirmar": true}), 3, &plan).is_none());
    assert_eq!(estado(dir), None);
}

#[test]
fn should_not_ask_for_confirmation_before_the_first_run() {
    let tmp = ws_temporal();

    assert!(requiere_confirmacion(tmp.path(), &json!({}), 3, &["x".to_owned()]).is_none());
}
