//! The error of `campo` when the label is not text: it must point at the keyed
//! way (`rellenar`) and list the keys the open form exposes.

use super::{error_sin_etiqueta, sugerencia_de_relleno};
use serde_json::json;

#[test]
fn should_suggest_rellenar_and_list_the_keys_when_the_label_is_not_text() {
    let vista = json!({
        "entidad": "components.paneles",
        "campos": [{"clave": "potencia_nominal_w"}, {"clave": "modelo"}]
    });

    let sugerencia = sugerencia_de_relleno(&vista);

    assert!(sugerencia.contains("accion:rellenar"), "{sugerencia}");
    assert!(sugerencia.contains("components.paneles"), "{sugerencia}");
    assert!(
        sugerencia.contains("potencia_nominal_w, modelo"),
        "{sugerencia}"
    );
}

#[test]
fn should_point_at_vista_when_no_entity_is_known() {
    let vista = json!({"entidad": null, "aviso": "no known window"});

    let sugerencia = sugerencia_de_relleno(&vista);

    assert!(sugerencia.contains("accion:rellenar"), "{sugerencia}");
    assert!(sugerencia.contains("que:vista"), "{sugerencia}");
}

#[test]
fn should_keep_the_original_reason_in_the_error() {
    let vista = json!({"entidad": "components.paneles", "campos": [{"clave": "modelo"}]});

    let error = error_sin_etiqueta("No field is called 'x'. Labels seen: ", &vista).to_string();

    assert!(error.contains("No field is called 'x'"), "{error}");
    assert!(error.contains("accion:rellenar"), "{error}");
}
