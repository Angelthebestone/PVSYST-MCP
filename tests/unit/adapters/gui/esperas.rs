use super::*;
use serde_json::json;

#[test]
fn should_honour_esperar_segundos_over_timeout_ms() {
    let args = json!({"esperar_segundos": 2, "timeout_ms": 500});
    assert_eq!(duracion_ms(&args), 2_000);
}

#[test]
fn should_not_wait_again_when_the_window_appeared_on_time() {
    let hallazgo = clasificar_espera(
        Some(7),
        || panic!("no re-read after a hit"),
        || panic!("no retry after a hit"),
    );

    assert_eq!(hallazgo, Hallazgo::ATiempo(7));
}

#[test]
fn should_report_late_when_the_recheck_finds_the_window() {
    let hallazgo = clasificar_espera(None, || Some(7), || panic!("the recheck already found it"));

    assert_eq!(hallazgo, Hallazgo::Tarde(7));
}

#[test]
fn should_report_late_when_the_window_appears_during_the_retry() {
    let hallazgo = clasificar_espera(None, || None, || Some(7));

    assert_eq!(hallazgo, Hallazgo::Tarde(7));
}

#[test]
fn should_report_nothing_when_the_window_never_appears() {
    assert_eq!(
        clasificar_espera::<u32>(None, || None, || None),
        Hallazgo::Nada
    );
}

#[test]
fn should_mark_the_answer_when_the_window_arrived_late() {
    let mut window = json!({"hwnd": "0x10", "class": "TProjectForm"});

    marcar_espera(
        &mut window,
        &Espera {
            tarde: true,
            timeout_ms: 12_000,
            etiqueta: "TProjectForm",
        },
    );

    assert_eq!(window["tarde"], true);
    assert_eq!(window["esperado_ms"], 12_000);
    assert!(
        window["aviso"]
            .as_str()
            .unwrap_or_default()
            .contains("12000 ms"),
        "{window}"
    );
}

#[test]
fn should_not_warn_when_the_window_arrived_on_time() {
    let mut window = json!({"hwnd": "0x10"});

    marcar_espera(
        &mut window,
        &Espera {
            tarde: false,
            timeout_ms: 12_000,
            etiqueta: "TProjectForm",
        },
    );

    assert_eq!(window["tarde"], false);
    assert!(window.get("aviso").is_none());
}
