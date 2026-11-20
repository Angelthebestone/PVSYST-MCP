use super::campos_vivos::{localizar, CampoVivo};
use super::dominio::{Ancla, TOLERANCIA_ANCLA_PX};
use super::esquemas::entidad;
use super::plan::planificar;
use serde_json::{json, Map, Value};

mod coincidencia;
mod guardados;

fn valores(v: &Value) -> Map<String, Value> {
    v.as_object().cloned().unwrap_or_default()
}

fn panel() -> &'static super::dominio::Entidad {
    entidad("components.panels").expect("panel schema")
}

fn vivo(clase: &str, x: i32, y: i32) -> CampoVivo {
    CampoVivo {
        hwnd: 1,
        clase: clase.into(),
        texto: String::new(),
        habilitado: true,
        posicion: Ancla { x, y },
    }
}

#[test]
fn should_report_every_problem_when_several_values_are_invalid() {
    let e = planificar(
        panel(),
        &valores(&json!({"potencia_nominal_w": 99999, "no_existe": 1, "t_oper_c": 20})),
    )
    .expect_err("invalid")
    .to_string();
    assert!(e.contains("fuera de rango"), "{e}");
    assert!(e.contains("'no_existe' no existe"), "{e}");
    assert!(e.contains("'t_oper_c' es de solo lectura"), "{e}");
    assert!(
        e.contains("potencia_nominal_w"),
        "valid keys are listed: {e}"
    );
}

#[test]
fn should_accept_numbers_given_as_text_with_a_comma() {
    let cambios = planificar(panel(), &valores(&json!({"isc_a": "10,5"}))).expect("plan");
    assert_eq!(cambios[0].texto, "10.5");
}

#[test]
fn should_not_locate_a_control_beyond_the_tolerance() {
    let campo = panel().campo("isc_a").expect("field");
    let lejos = [vivo(
        "TEdit",
        campo.ancla.x + TOLERANCIA_ANCLA_PX + 1,
        campo.ancla.y,
    )];
    assert!(localizar(campo, &lejos).is_none());
}

#[test]
fn should_plan_a_fill_without_a_window_when_simulating() {
    let paths = crate::config::Paths::para_test(std::env::temp_dir());
    // No process and no window: a rehearsal must answer with PVsyst closed.
    let pedido = json!({
        "entidad": "variante.orientation",
        "valores": { "inclinacion_plano_deg": 30 },
        "simular": true,
        "pids": [],
    });
    let salida = super::rellenar(&paths, &pedido).expect("el ensayo no necesita ventana");
    let paso = &salida["resultados"][0];
    assert_eq!(salida["simulado"], json!(true), "{salida}");
    assert_eq!(paso["clave"], json!("inclinacion_plano_deg"), "{salida}");
    assert_eq!(paso["pedido"], json!("30"), "{salida}");
    assert_eq!(paso["estado"], json!("simulado"), "{salida}");
    assert!(salida.get("ventana").is_none(), "{salida}");
    assert_eq!(paso["antes"], json!(""), "no se leyó nada: {salida}");
    let aviso = salida["aviso"].as_str().unwrap_or_default();
    assert!(!aviso.is_empty(), "dice que no se leyó: {salida}");
}
