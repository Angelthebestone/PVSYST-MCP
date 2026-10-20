use super::*;

const BANNER: &str =
    "PVsystCLI is running in TRIAL mode, remaining days: 58, remaining executions: 223";

fn ws_temporal(nombre: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("pvsyst_ledger_{}_{nombre}", std::process::id()));
    let _ = fs::remove_dir_all(&dir);
    dir
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
    let dir = ws_temporal("descuento");
    let costo = registrar(&dir, BANNER, 1);
    assert_eq!(costo["restantes"], json!(222));
    assert_eq!(estado(&dir).map(|e| e.restantes), Some(Some(222)));
}

#[test]
fn sin_banner_se_registra_licencia() {
    let dir = ws_temporal("licencia");
    registrar(&dir, "Simulation done in 1min", 1);
    assert_eq!(estado(&dir).map(|e| e.modo), Some("licencia".to_owned()));
}

#[test]
fn confirmacion_solo_en_trial_con_varias_ejecuciones() {
    let dir = ws_temporal("confirmacion");
    let plan = vec!["barrido".to_owned()];
    assert!(requiere_confirmacion(&dir, &json!({}), 3, &plan).is_none());
    registrar(&dir, BANNER, 0);
    let retencion = requiere_confirmacion(&dir, &json!({}), 3, &plan)
        .expect("debe pedir confirmación en trial");
    assert_eq!(retencion["requiere_confirmacion"], json!(true));
    assert!(requiere_confirmacion(&dir, &json!({}), 1, &plan).is_none());
    assert!(requiere_confirmacion(&dir, &json!({"confirmar": true}), 3, &plan).is_none());
}
