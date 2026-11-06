use super::*;
use crate::config::Paths;
use crate::idioma::Idioma;

fn paths_de_test() -> Paths {
    Paths::para_test(std::env::temp_dir())
}

#[test]
fn should_reject_missing_required_arg() {
    let paths = Paths::para_test(std::env::temp_dir());
    let err = call(&paths, "leer", &json!({})).expect_err("should reject missing required arg");
    let msg = err.to_string();
    assert!(
        msg.contains("Missing: archivo") && msg.contains("Example:"),
        "error message must name the missing field and give an example: {msg}"
    );
}

#[test]
fn should_name_every_missing_required_arg_of_the_action() {
    let paths = Paths::para_test(std::env::temp_dir());
    let err = call(&paths, "sitio", &json!({ "accion": "meteo_pvgis" }))
        .expect_err("meteo_pvgis needs latitud, longitud and nombre_sitio");
    let msg = err.to_string();
    assert!(
        msg.contains("latitud") && msg.contains("longitud") && msg.contains("nombre_sitio"),
        "{msg}"
    );
    assert!(
        msg.contains("Example:") && msg.contains("<nombre_sitio>"),
        "{msg}"
    );
}

#[test]
fn should_require_operacion_for_editar_rvt() {
    let paths = Paths::para_test(std::env::temp_dir());
    let err = call(&paths, "editar", &json!({ "accion": "rvt" }))
        .expect_err("rvt without operacion must not default to listar");
    assert!(err.to_string().contains("operacion"), "{err}");
}

#[test]
fn should_reject_wrong_type_arg_mentioning_tipo() {
    let paths = Paths::para_test(std::env::temp_dir());
    let err = call(
        &paths,
        "leer",
        &json!({ "archivo": "test.PAN", "base64": "yes" }),
    )
    .expect_err("should reject string for boolean property");
    let msg = err.to_string();
    assert!(
        msg.contains("Wrong type for 'base64'")
            && msg.contains("expected boolean")
            && msg.contains("got string"),
        "error must name the property, expected type and received type: {msg}"
    );
}

#[test]
fn should_reject_wrong_type_for_numeric_property() {
    let paths = Paths::para_test(std::env::temp_dir());
    let err = call(
        &paths,
        "proyecto",
        &json!({ "accion": "dimensionar", "proyecto": "Demo", "n_inversores": true }),
    )
    .expect_err("should reject boolean for integer property");
    let msg = err.to_string();
    assert!(
        msg.contains("Wrong type for 'n_inversores'")
            && msg.contains("expected integer")
            && msg.contains("got boolean"),
        "error must name property, expected and received types: {msg}"
    );
}

#[test]
fn should_reject_missing_discriminator_when_required() {
    let paths = Paths::para_test(std::env::temp_dir());
    let err = call(&paths, "proyecto", &json!({})).expect_err("should require accion");
    let msg = err.to_string();
    assert!(
        msg.contains("Missing required argument 'accion'"),
        "should name the missing discriminator: {msg}"
    );
}

#[test]
fn should_apply_default_discriminator_when_omitted() {
    let paths = Paths::para_test(std::env::temp_dir());
    let res = call(&paths, "entorno", &json!({})).expect("should succeed with default diagnostico");
    assert!(res.get("entorno").is_some());
}

#[test]
fn should_reject_wrong_type_for_discriminator() {
    let paths = Paths::para_test(std::env::temp_dir());
    let err = call(&paths, "entorno", &json!({ "accion": 12345 }))
        .expect_err("should reject number for accion");
    let msg = err.to_string();
    assert!(
        msg.to_lowercase().contains("type"),
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
    assert!(h.validar(&val_int, &paths_de_test()).is_ok());
    assert!(h.validar(&val_str, &paths_de_test()).is_ok());

    let val_bad =
        json!({ "accion": "crear", "plantilla": "P", "nombre_nuevo": "N", "n_mod_serie": false });
    assert!(matches!(
        h.validar(&val_bad, &paths_de_test()),
        Err(crate::error::Error::Invalid(_))
    ));
}

#[test]
#[ignore = "necesita Currency.dat de la instalación de PVsyst (DataRO): ejecutar con --ignored"]
fn should_dispatch_leer_for_currency_mef_and_3d_scenes() {
    let paths = Paths::para_test(std::env::temp_dir());

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
    // Only the argument check: calling the tool would send a menu command to window 0x1.
    let h = REGISTRO
        .iter()
        .find(|h| h.nombre == "gui_actuar")
        .expect("gui_actuar registrada");
    let args = json!({ "accion": "menu", "ventana": "0x1", "menu": 0, "item": 0 });

    let accion = h.validar(
        &h.preparar_args(&args, Idioma::En).expect("args"),
        &Paths::para_test(std::env::temp_dir()),
    );

    assert_eq!(accion.expect("argumentos válidos sin 'id'"), "menu");
}

#[test]
fn should_demand_the_window_when_a_menu_command_names_none() {
    let paths = Paths::para_test(std::env::temp_dir());
    let error = call(&paths, "gui_actuar", &json!({ "accion": "menu", "id": 70 }))
        .unwrap_err()
        .to_string();
    assert!(
        error.contains("Missing: ventana"),
        "missing expected 'ventana' error, got: {error}"
    );
}

#[test]
fn should_validate_every_configured_step_before_applying_the_first() {
    let ws = tempfile::tempdir().expect("ws");
    let paths = Paths::para_test(ws.path());
    let args = json!({
        "accion": "configurar",
        "proyecto": "Planta",
        "pasos": [
            { "accion": "perdidas", "suciedad_pct": 2 },
            { "accion": "limite_inyeccion" }
        ]
    });

    let err = call(&paths, "proyecto", &args).expect_err("paso 2 sin potencia_kw");

    let msg = err.to_string();
    assert!(msg.contains("potencia_kw"), "{msg}");
    assert!(!msg.contains("Paso 1 fallido"), "{msg}");
}

#[test]
fn should_keep_the_error_kind_of_a_failed_step() {
    let ws = tempfile::tempdir().expect("ws");
    let paths = Paths::para_test(ws.path());
    let args = json!({ "accion": "configurar", "proyecto": "NoExiste", "pasos": [ { "accion": "perdidas", "suciedad_pct": 2 } ] });

    let err = call(&paths, "proyecto", &args).expect_err("proyecto inexistente");

    assert!(
        matches!(&err, crate::error::Error::NotFound(m) if m.starts_with("Step 1 failed")),
        "{err:?}"
    );
}

#[test]
fn should_reject_wrong_type_in_spanish_when_language_is_es() {
    let h = REGISTRO.iter().find(|h| h.nombre == "leer").unwrap();
    let args = json!({ "archivo": "test.PAN", "base64": "yes" });

    let mut paths = Paths::para_test(std::env::temp_dir());
    paths.idioma = Idioma::Es;
    let err = h
        .validar(&args, &paths)
        .expect_err("string for boolean must fail");

    let msg = err.to_string();
    assert!(
        msg.contains("Tipo incorrecto para 'base64'")
            && msg.contains("esperado boolean")
            && msg.contains("recibido string"),
        "{msg}"
    );
}
