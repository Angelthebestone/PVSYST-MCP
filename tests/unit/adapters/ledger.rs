use super::*;

const BANNER: &str =
    "PVsystCLI is running in TRIAL mode, remaining days: 58, remaining executions: 223";

/// A private workspace that is removed when the test ends.
fn ws_temporal() -> tempfile::TempDir {
    tempfile::tempdir().expect("workspace temporal")
}

#[test]
fn should_discount_the_spent_executions_when_registering_a_run() {
    let tmp = ws_temporal();
    let dir = tmp.path();
    let costo = registrar(dir, BANNER, 1);
    assert_eq!(costo["restantes"], json!(222));
    assert_eq!(estado(dir).map(|e| e.restantes), Some(Some(222)));
}

/// PVsystCLI's counter was measured jumping between two states (133/134 and
/// 249/250) without a licence change, so a budget taken from it cannot be spent.
/// The ledger keeps the lowest count it has seen until the licence renews.
#[test]
fn should_keep_the_trial_counter_from_going_back_up() {
    let tmp = ws_temporal();
    let dir = tmp.path();
    registrar(dir, BANNER, 1);
    let bajo = "PVsystCLI is running in TRIAL mode, remaining days: 58, remaining executions: 134";
    assert_eq!(registrar(dir, bajo, 1)["restantes"], json!(133));
    assert_eq!(registrar(dir, BANNER, 1)["restantes"], json!(133));
    assert_eq!(estado(dir).map(|e| e.restantes), Some(Some(133)));
}

/// A renewal raises the day count, and then the new counter is the true one.
#[test]
fn should_accept_a_higher_counter_after_a_renewal() {
    let tmp = ws_temporal();
    let dir = tmp.path();
    let bajo = "PVsystCLI is running in TRIAL mode, remaining days: 30, remaining executions: 134";
    registrar(dir, bajo, 1);
    let renovado =
        "PVsystCLI is running in TRIAL mode, remaining days: 60, remaining executions: 250";
    assert_eq!(registrar(dir, renovado, 1)["restantes"], json!(249));
}

#[test]
fn should_ask_for_confirmation_when_a_trial_run_would_spend_several_executions() {
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
