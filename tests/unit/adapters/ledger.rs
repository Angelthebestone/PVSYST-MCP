use super::*;
use crate::adapters::ledger_reserva::{
    bajo_reserva, requiere_confirmacion_con, reserva_de, Gasto, RESERVA_POR_DEFECTO,
};

const BANNER: &str =
    "PVsystCLI is running in TRIAL mode, remaining days: 58, remaining executions: 223";

/// A private workspace that is removed when the test ends.
fn ws_temporal() -> tempfile::TempDir {
    tempfile::tempdir().expect("workspace temporal")
}

/// A ledger that states the licence and what it has left, without a CLI run.
fn escribir_ledger(dir: &Path, modo: &str, restantes: Option<u64>) {
    let estado = Estado {
        modo: modo.to_owned(),
        dias: None,
        restantes,
    };
    guardar(dir, &estado).expect("ledger escrito");
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

/// `lic-info` prints the same counters under another shape (`Status: TRIAL`,
/// `Remaining executions: N`); reading them is what lets a cold workspace answer
/// the budget before its first run.
#[test]
fn should_read_the_counters_of_lic_info() {
    let lic_info = concat!(
        "The license file \"C:\\ProgramData\\PVsystCLI\\PVsystCLI.lic\" is missing.\r\n",
        "You can run \"lic-activate\" command to activate your license.\r\n",
        "Status: TRIAL\r\n",
        "Remaining days: 58\r\n",
        "Remaining executions: 223\r\n",
        "Host ID: HARDDISK=0000_0000_ACE4_2E00_3B04_AF6C\r\n",
    );
    let estado = parsear_lic_info(lic_info).expect("lic-info legible");
    assert_eq!(estado.modo, "trial");
    assert_eq!(estado.dias, Some(58));
    assert_eq!(estado.restantes, Some(223));
}

/// A licensed install prints `Status:` without the trial counters.
#[test]
fn should_read_a_licensed_lic_info_without_counters() {
    let estado = parsear_lic_info("Status: Licensed\r\n").expect("lic-info legible");
    assert_eq!(estado.modo, "licencia");
    assert_eq!(estado.restantes, None);
}

/// A run without the trial banner is not a licence change: it must not wipe the
/// counters a previous `lic-info` stored.
#[test]
fn should_not_wipe_the_ledger_when_a_run_prints_no_trial_banner() {
    let tmp = ws_temporal();
    let dir = tmp.path();
    registrar(dir, BANNER, 0);
    registrar(dir, "Simulation done in 0min 3sec\r\n", 1);
    assert_eq!(estado(dir).map(|e| e.restantes), Some(Some(223)));
}

/// The reserve comes from `PVSYST_RESERVA_TRIAL`; every call reads it through this
/// parser, so an unusable value must fall back to the default.
#[test]
fn should_read_the_trial_reserve_from_its_value() {
    assert_eq!(reserva_de(None), RESERVA_POR_DEFECTO);
    assert_eq!(reserva_de(Some("  ")), RESERVA_POR_DEFECTO);
    assert_eq!(reserva_de(Some("abc")), RESERVA_POR_DEFECTO);
    assert_eq!(reserva_de(Some("-3")), RESERVA_POR_DEFECTO);
    assert_eq!(reserva_de(Some("0")), 0);
    assert_eq!(reserva_de(Some("25")), 25);
}

/// A spend lands on the reserve before it goes under it.
#[test]
fn should_keep_the_reserve_only_when_the_spend_goes_below_it() {
    // 12 left, 2 spent, 10 in reserve: exactly on the reserve.
    assert!(!bajo_reserva(Some(12), 2, 10));
    // 11 left, 2 spent: 9 would remain, one under.
    assert!(bajo_reserva(Some(11), 2, 10));
    // A charge bigger than the balance saturates instead of wrapping.
    assert!(bajo_reserva(Some(1), 5, 10));
    // No counter, nothing to compare: only the threshold rule applies.
    assert!(!bajo_reserva(None, 1, 10));
    // 0 turns the reserve off.
    assert!(!bajo_reserva(Some(1), 5, 0));
}

/// The reserve of the rule is given to this call, so the test never depends on how
/// the machine running it is configured.
#[test]
fn should_ask_for_confirmation_when_a_trial_run_would_spend_several_executions() {
    let tmp = ws_temporal();
    let dir = tmp.path();
    let plan = vec!["barrido".to_owned()];
    let tres = Gasto::nuevo(3, &plan, RESERVA_POR_DEFECTO);
    assert!(requiere_confirmacion_con(dir, &json!({}), &tres).is_none());
    escribir_ledger(dir, "trial", Some(223));
    let retencion = requiere_confirmacion_con(dir, &json!({}), &tres)
        .expect("debe pedir confirmación en trial");
    assert_eq!(retencion["requiere_confirmacion"], json!(true));
    assert_eq!(retencion["ejecuciones_estimadas"], json!(3));
    assert_eq!(retencion["restantes"], json!(223));
    assert_eq!(retencion["reserva"], json!(RESERVA_POR_DEFECTO));
    assert_eq!(retencion["plan"], json!(["barrido"]));
    let una = Gasto::nuevo(1, &plan, RESERVA_POR_DEFECTO);
    assert!(requiere_confirmacion_con(dir, &json!({}), &una).is_none());
    assert!(requiere_confirmacion_con(dir, &json!({"confirmar": true}), &tres).is_none());
}

/// A loop of one-run calls drains a trial balance without a single loud call: the
/// reserve is what makes the last ones ask, and the answer says why.
#[test]
fn should_ask_for_confirmation_when_a_single_run_eats_into_the_reserve() {
    let tmp = ws_temporal();
    let dir = tmp.path();
    escribir_ledger(dir, "trial", Some(10));
    let plan = vec!["simular".to_owned()];
    let una = Gasto::nuevo(1, &plan, RESERVA_POR_DEFECTO);

    let retencion = requiere_confirmacion_con(dir, &json!({}), &una)
        .expect("1 de 10 restantes deja 9: bajo la reserva de 10");

    assert_eq!(retencion["requiere_confirmacion"], json!(true));
    assert_eq!(retencion["ejecuciones_estimadas"], json!(1));
    assert_eq!(retencion["restantes"], json!(10));
    assert_eq!(retencion["reserva"], json!(RESERVA_POR_DEFECTO));
    let mensaje = retencion["mensaje"].as_str().expect("mensaje");
    assert!(mensaje.contains("PVSYST_RESERVA_TRIAL"), "{mensaje}");
    assert!(mensaje.contains("confirmar:true"), "{mensaje}");
    assert!(mensaje.contains("10"), "{mensaje}");
    assert!(requiere_confirmacion_con(dir, &json!({"confirmar": true}), &una).is_none());

    // 11 left and 10 in reserve: the run lands exactly on the reserve, not under it.
    escribir_ledger(dir, "trial", Some(11));
    assert!(requiere_confirmacion_con(dir, &json!({}), &una).is_none());
}

#[test]
fn should_not_ask_for_confirmation_when_the_trial_balance_has_room() {
    let tmp = ws_temporal();
    let dir = tmp.path();
    escribir_ledger(dir, "trial", Some(113));
    let plan = vec!["simular".to_owned()];
    let una = Gasto::nuevo(1, &plan, RESERVA_POR_DEFECTO);
    assert!(requiere_confirmacion_con(dir, &json!({}), &una).is_none());
}

/// A licence has no budget to protect, so the reserve never applies: a stale
/// counter next to it must not hold a run back.
#[test]
fn should_not_keep_a_reserve_with_a_licence() {
    let tmp = ws_temporal();
    let dir = tmp.path();
    escribir_ledger(dir, "licencia", Some(3));
    let plan = vec!["simular".to_owned()];
    let una = Gasto::nuevo(1, &plan, RESERVA_POR_DEFECTO);
    assert!(requiere_confirmacion_con(dir, &json!({}), &una).is_none());
}

/// A call that estimates no run spends none: not even a balance already under the
/// reserve turns a cache hit into a confirmation.
#[test]
fn should_not_ask_for_confirmation_when_the_call_spends_nothing() {
    let tmp = ws_temporal();
    let dir = tmp.path();
    escribir_ledger(dir, "trial", Some(3));
    let plan = vec!["simular".to_owned()];
    let cero = Gasto::nuevo(0, &plan, RESERVA_POR_DEFECTO);
    assert!(requiere_confirmacion_con(dir, &json!({}), &cero).is_none());
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
