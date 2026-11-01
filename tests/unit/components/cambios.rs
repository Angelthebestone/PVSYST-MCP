use super::*;
use crate::config::Paths;
use serde_json::json;

const OND: &[u8] = b"PVObject_=pvInverter\r\n  PNomConv=5000\r\nEnd of PVObject pvInverter\r\n";

#[test]
fn should_apply_changes_given_as_campos_to_a_text_component() {
    let mut bytes = OND.to_vec();

    let aplicados = aplicar_cambios(&mut bytes, &json!({ "campos": { "PNomConv": "6000" } }))
        .expect("aplicados");

    assert_eq!(aplicados.len(), 1);
    assert!(String::from_utf8_lossy(&bytes).contains("PNomConv=6000"));
}

#[test]
fn should_refuse_text_component_without_a_readable_object() {
    let mut bytes = b"PVObject_=pvInverter\r\n  PNomConv=5000\r\n".to_vec();

    let res = aplicar_cambios(&mut bytes, &json!({ "cambios": { "PNomConv": "6000" } }));

    assert!(res.is_err_and(|e| matches!(e, Error::Format(_) | Error::Invalid(_))));
}

#[test]
fn should_not_rewrite_component_when_edit_has_no_changes() {
    let ws = tempfile::tempdir().expect("ws");
    let dir = ws.path().join("ComposPV").join("Inverters");
    std::fs::create_dir_all(&dir).expect("dir");
    let ond = dir.join("Mio.OND");
    std::fs::write(&ond, OND).expect("ond");
    let paths = Paths::para_test(ws.path());

    for args in [
        json!({ "componente": "Mio.OND", "tipo": "OND" }),
        json!({ "componente": "Mio.OND", "tipo": "OND", "cambios": {} }),
    ] {
        let res = crate::components::editar_componente(&paths, &args);
        assert!(
            matches!(res, Err(Error::Invalid(m)) if m.contains("cambios")),
            "{args}"
        );
    }
    assert_eq!(std::fs::read(&ond).expect("ond"), OND);
    assert!(!ws.path().join("UserData").join("mcp_history").exists());
}

const OND_TRUNCADO: &[u8] =
    b"PVObject_=pvInverter\r\n  PVObject_=pvCommercial\r\n    Model=X\r\nEnd of PVObject pvInverter\r\n";

#[test]
fn should_refuse_to_read_or_edit_a_truncated_component() {
    let ws = tempfile::tempdir().expect("ws");
    let dir = ws.path().join("ComposPV").join("Inverters");
    std::fs::create_dir_all(&dir).expect("dir");
    std::fs::write(dir.join("Roto.OND"), OND_TRUNCADO).expect("ond");
    let paths = Paths::para_test(ws.path());
    let leer = json!({ "componente": "Roto.OND", "tipo": "OND" });
    let editar = json!({ "componente": "Roto.OND", "tipo": "OND", "cambios": { "Model": "Y" } });

    assert!(matches!(
        crate::components::leer_componente(&paths, &leer),
        Err(Error::Format(_))
    ));
    assert!(matches!(
        crate::components::editar_componente(&paths, &editar),
        Err(Error::Format(_))
    ));
    assert_eq!(
        std::fs::read(dir.join("Roto.OND")).expect("ond"),
        OND_TRUNCADO
    );
}
