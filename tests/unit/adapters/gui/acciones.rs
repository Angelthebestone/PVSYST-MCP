use super::*;
use serde_json::json;

/// GUI-06: `texto` was read by nobody and the click went to the window itself.
#[test]
fn should_refuse_texto_and_id_that_no_click_reads() {
    for args in [
        json!({ "ventana": "TDimensForm", "texto": "Sitio y datos meteo" }),
        json!({ "ventana": "TDimensForm", "id": 23 }),
    ] {
        let error = exigir_sin_texto_ni_id(&args).expect_err("ignored argument");

        assert!(error.to_string().contains("clic_texto"), "{error}");
    }
}
