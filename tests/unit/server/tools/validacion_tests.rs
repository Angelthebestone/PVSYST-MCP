use super::*;

#[test]
fn should_reject_missing_required_arg() {
    let paths = Paths::resolve();
    let err = call(&paths, "leer", &json!({})).expect_err("should reject missing required arg");
    let msg = err.to_string();
    assert!(
        msg.contains("Missing required argument 'archivo'"),
        "error message must name the missing field: {msg}"
    );
}

#[test]
fn should_reject_wrong_type_arg_mentioning_tipo() {
    let paths = Paths::resolve();
    let err = call(
        &paths,
        "leer",
        &json!({ "archivo": "test.PAN", "base64": "yes" }),
    )
    .expect_err("should reject string for boolean property");
    let msg = err.to_string();
    assert!(
        msg.contains("Tipo incorrecto para 'base64'")
            && msg.contains("esperado boolean")
            && msg.contains("recibido string"),
        "error must name the property, expected type and received type: {msg}"
    );
}

#[test]
fn should_reject_wrong_type_for_numeric_property() {
    let paths = Paths::resolve();
    let err = call(
        &paths,
        "proyecto",
        &json!({ "accion": "dimensionar", "proyecto": "Demo", "n_inversores": true }),
    )
    .expect_err("should reject boolean for integer property");
    let msg = err.to_string();
    assert!(
        msg.contains("Tipo incorrecto para 'n_inversores'")
            && msg.contains("esperado integer")
            && msg.contains("recibido boolean"),
        "error must name property, expected and received types: {msg}"
    );
}

#[test]
fn should_reject_missing_discriminator_when_required() {
    let paths = Paths::resolve();
    let err = call(&paths, "proyecto", &json!({})).expect_err("should require accion");
    let msg = err.to_string();
    assert!(
        msg.contains("Missing required argument 'accion'"),
        "should name the missing discriminator: {msg}"
    );
}

#[test]
fn should_apply_default_discriminator_when_omitted() {
    let paths = Paths::resolve();
    let res = call(&paths, "entorno", &json!({})).expect("should succeed with default diagnostico");
    assert!(res.get("entorno").is_some());
}

#[test]
fn should_reject_invalid_discriminator_value() {
    let paths = Paths::resolve();
    let err = call(&paths, "entorno", &json!({ "accion": "inexistente" }))
        .expect_err("should reject invalid accion");
    let msg = err.to_string();
    assert!(
        msg.contains("Valor 'inexistente' no válido"),
        "should reject invalid value: {msg}"
    );
}

#[test]
fn should_reject_wrong_type_for_discriminator() {
    let paths = Paths::resolve();
    let err = call(&paths, "entorno", &json!({ "accion": 12345 }))
        .expect_err("should reject number for accion");
    let msg = err.to_string();
    assert!(
        msg.to_lowercase().contains("tipo"),
        "error must explain the wrong type: {msg}"
    );
}

#[test]
fn should_accept_compatible_union_types() {
    let h = REGISTRO.iter().find(|h| h.nombre == "proyecto").unwrap();
    let val_int =
        json!({ "accion": "crear", "plantilla": "P", "nombre_nuevo": "N", "n_mod_serie": 20 });
    let val_str =
        json!({ "accion": "crear", "plantilla": "P", "nombre_nuevo": "N", "n_mod_serie": "20" });
    assert!(h.validar(&val_int).is_ok());
    assert!(h.validar(&val_str).is_ok());

    let val_bad =
        json!({ "accion": "crear", "plantilla": "P", "nombre_nuevo": "N", "n_mod_serie": false });
    assert!(h.validar(&val_bad).is_err());
}

#[test]
fn should_dispatch_leer_for_currency_mef_and_3d_scenes() {
    let paths = Paths::resolve();

    let res = call(&paths, "leer", &json!({ "archivo": "Currency.dat" }));
    assert!(
        res.is_ok(),
        "leer should route Currency.dat without schema rejection: {:?}",
        res.err()
    );

    let res = call(&paths, "leer", &json!({ "archivo": "PVsyst_ANE-NEDO.MEF" }));
    assert!(
        res.is_ok(),
        "leer should route .MEF without schema rejection: {:?}",
        res.err()
    );

    let res = call(
        &paths,
        "leer",
        &json!({ "archivo": "DEMO_Geneve_Grid.SHD" }),
    );
    assert!(
        res.is_ok(),
        "leer should route .SHD without schema rejection: {:?}",
        res.err()
    );
}

#[test]
fn should_not_demand_an_id_when_a_menu_command_is_named_by_position() {
    let paths = Paths::resolve();
    let args = json!({ "accion": "menu", "ventana": "0x1", "menu": 0, "item": 0 });
    let err = call(&paths, "gui_actuar", &args).unwrap_err().to_string();
    assert!(
        !err.contains("Missing required argument 'id'"),
        "unexpected 'id' requirement in: {err}"
    );
}

#[test]
fn should_demand_the_window_when_a_menu_command_names_none() {
    let paths = Paths::resolve();
    let error = call(&paths, "gui_actuar", &json!({ "accion": "menu", "id": 70 }))
        .unwrap_err()
        .to_string();
    assert!(
        error.contains("Missing required argument 'ventana'"),
        "missing expected 'ventana' error, got: {error}"
    );
}
