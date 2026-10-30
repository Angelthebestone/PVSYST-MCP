use super::campos_vivos::{localizar, CampoVivo};
use super::dominio::{Ancla, TOLERANCIA_ANCLA_PX};
use super::esquemas::{de_formulario, entidad, todas};
use super::plan::{coincide, es_valido, planificar};
use crate::error::Error;
use serde_json::{json, Map, Value};

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
fn should_plan_changes_in_schema_order_when_values_are_valid() {
    let cambios = planificar(
        panel(),
        &valores(&json!({"voc_v": 49.0, "modelo": "X1", "potencia_nominal_w": 200})),
    )
    .expect("plan");
    let claves: Vec<&str> = cambios.iter().map(|c| c.campo.clave).collect();
    assert_eq!(claves, ["modelo", "potencia_nominal_w", "voc_v"]);
    assert_eq!(cambios[1].texto, "200");
}

#[test]
fn should_report_every_problem_when_several_values_are_wrong() {
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
fn should_reject_fractional_values_for_integer_fields() {
    let e = planificar(panel(), &valores(&json!({"celulas_en_serie": 72.5}))).expect_err("invalid");
    assert!(e.to_string().contains("entero"), "{e}");
}

#[test]
fn should_reject_an_empty_request() {
    let err = planificar(panel(), &Map::new()).unwrap_err();
    assert!(
        matches!(&err, Error::Invalid(msg) if msg.contains("no hay ningún valor")),
        "error inesperado para petición vacía: {err}"
    );
}

#[test]
fn should_match_numbers_by_value_when_pvsyst_reformats_them() {
    let campo = panel().campo("potencia_nominal_w").expect("field");
    assert!(coincide(campo, "200", "200.0"));
    assert!(!coincide(campo, "200", "201"));
}

#[test]
fn should_flag_a_field_value_outside_its_range() {
    let campo = panel().campo("potencia_nominal_w").expect("field");
    assert!(es_valido(campo, "0.1"));
    assert!(!es_valido(campo, "N/A"));
    assert!(!es_valido(campo, "5000"));
}

#[test]
fn should_locate_the_control_nearest_to_the_anchor_of_its_class() {
    let campo = panel().campo("isc_a").expect("field");
    let vivos = [
        vivo("TEdit", campo.ancla.x + 30, campo.ancla.y),
        vivo("TEdit", campo.ancla.x + 2, campo.ancla.y - 1),
        vivo("TComboBox", campo.ancla.x, campo.ancla.y),
    ];
    let hallado = localizar(campo, &vivos).expect("found");
    assert_eq!(hallado.posicion.x, campo.ancla.x + 2);
    assert_eq!(hallado.clase, "TEdit");
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

fn verificar_pares_de_campos(
    entidad_clave: &str,
    campos: &[crate::vista_agente::dominio::CampoEsquema],
) {
    for (i, a) in campos.iter().enumerate() {
        for b in &campos[i + 1..] {
            assert_ne!(a.clave, b.clave, "duplicate key in {entidad_clave}");
            let cerca = (a.ancla.x - b.ancla.x).abs() <= TOLERANCIA_ANCLA_PX
                && (a.ancla.y - b.ancla.y).abs() <= TOLERANCIA_ANCLA_PX;
            assert!(
                !(cerca && a.clase == b.clase),
                "{} and {} share an anchor",
                a.clave,
                b.clave
            );
        }
    }
}

#[test]
fn should_keep_schema_keys_and_anchors_unambiguous() {
    for e in todas() {
        verificar_pares_de_campos(e.clave, e.campos);
    }
}

#[test]
fn should_resolve_an_entity_from_its_window_class() {
    assert_eq!(
        de_formulario("TPVModuleForm").map(|e| e.clave),
        Some("components.panels")
    );
    assert!(de_formulario("TNoExiste").is_none());
}

#[test]
fn should_reject_an_unknown_schema_entity_instead_of_listing_all() {
    let r = crate::vista_agente::listar_esquemas(&json!({ "entidad": "zzz" }));
    assert!(r.is_err());
}

#[test]
fn should_return_only_the_requested_schema_entity() {
    let v = crate::vista_agente::listar_esquemas(&json!({ "entidad": "components.panels" }))
        .expect("schema");
    assert_eq!(v["count"], json!(1));
}

/// A module typed through `rellenar` (a different value in every field) and saved by PVsyst 8.1.8.
const MODULO_GUARDADO: &str =
    include_str!("../../fixtures/vista_agente/Modulo_distintos_8_1_8.PAN");

#[test]
fn should_find_every_file_key_of_the_panel_schema_in_the_module_pvsyst_saved() {
    let claves: Vec<&str> = MODULO_GUARDADO
        .lines()
        .filter_map(|l| l.trim().split_once('=').map(|(k, _)| k))
        .collect();
    let con_clave = panel().campos.iter().filter_map(|c| c.clave_archivo);
    for clave in con_clave {
        assert!(
            claves.contains(&clave),
            "{clave} no está en el archivo guardado"
        );
    }
}

#[test]
fn should_keep_the_units_the_saved_module_shows_for_the_values_typed() {
    let valor = |k: &str| {
        MODULO_GUARDADO
            .lines()
            .find_map(|l| l.trim().strip_prefix(&format!("{k}=")).map(str::to_string))
            .and_then(|v| v.trim().parse::<f64>().ok())
    };
    // The values `rellenar` typed: W, A, V, mA/°C, °C and a count.
    let tecleado = [
        ("PNom", 410.5),
        ("Isc", 11.2),
        ("Voc", 48.3),
        ("Imp", 10.4),
        ("Vmp", 39.5),
    ];
    for (clave, esperado) in tecleado {
        assert_eq!(valor(clave), Some(esperado), "{clave}");
    }
    assert_eq!(valor("muISC"), Some(4.2));
    assert_eq!(valor("TRef"), Some(20.0));
    assert_eq!(valor("NCelS"), Some(72.0));
}

/// An inverter typed through `rellenar` and saved by PVsyst 8.1.8.
const INVERSOR_GUARDADO: &str =
    include_str!("../../fixtures/vista_agente/Inversor_distintos_8_1_8.OND");

fn valor_guardado(archivo: &str, clave: &str) -> Option<f64> {
    archivo
        .lines()
        .find_map(|l| {
            l.trim()
                .strip_prefix(&format!("{clave}="))
                .map(str::to_string)
        })
        .and_then(|v| v.trim().parse::<f64>().ok())
}

#[test]
fn should_find_every_file_key_of_the_inverter_schema_in_the_inverter_pvsyst_saved() {
    let claves: Vec<&str> = INVERSOR_GUARDADO
        .lines()
        .filter_map(|l| l.trim().split_once('=').map(|(k, _)| k))
        .collect();
    let esquema = entidad("components.inverters").expect("inverter schema");
    for clave in esquema.campos.iter().filter_map(|c| c.clave_archivo) {
        assert!(
            claves.contains(&clave),
            "{clave} no está en el archivo guardado"
        );
    }
}

#[test]
fn should_keep_the_units_the_saved_inverter_shows_for_the_values_typed() {
    // Volts and kilowatts, as typed by `rellenar`.
    let tecleado = [
        ("VMppMin", 310.0),
        ("VmppNom", 600.0),
        ("VMPPMax", 820.0),
        ("VAbsMax", 1000.0),
        ("VOutConv", 400.0),
        ("PNomConv", 20.5),
        ("PMaxOUT", 22.5),
        ("PSeuil", 0.1),
    ];
    for (clave, esperado) in tecleado {
        assert_eq!(
            valor_guardado(INVERSOR_GUARDADO, clave),
            Some(esperado),
            "{clave}"
        );
    }
}

/// A battery typed through `rellenar` and saved by PVsyst 8.1.8.
const BATERIA_GUARDADA: &str =
    include_str!("../../fixtures/vista_agente/Bateria_distintos_8_1_8.BTR");

#[test]
fn should_find_every_file_key_of_the_battery_schema_in_the_battery_pvsyst_saved() {
    let claves: Vec<&str> = BATERIA_GUARDADA
        .lines()
        .filter_map(|l| l.trim().split_once('=').map(|(k, _)| k))
        .collect();
    let esquema = entidad("components.batteries").expect("battery schema");
    for clave in esquema.campos.iter().filter_map(|c| c.clave_archivo) {
        assert!(
            claves.contains(&clave),
            "{clave} no está en el archivo guardado"
        );
    }
}

#[test]
fn should_keep_the_scale_the_saved_battery_uses_for_the_values_typed() {
    // Typed: 6 cells, 2.1 V, 150.5 Ah, 25 °C, 3.5 mOhm (stored in ohm) and 95.5 % (stored as a fraction).
    let tecleado = [
        ("NCellSerie", 6.0),
        ("VNomCell", 2.1),
        ("CapNomC10", 150.5),
        ("BattTempRef", 25.0),
        ("ResInterne", 0.0035),
        ("EfficI", 0.955),
    ];
    for (clave, esperado) in tecleado {
        assert_eq!(
            valor_guardado(BATERIA_GUARDADA, clave),
            Some(esperado),
            "{clave}"
        );
    }
}

#[test]
fn should_put_only_the_efficiency_fields_of_the_inverter_on_the_second_tab() {
    let esquema = entidad("components.inverters").expect("inverter schema");
    let en_pagina_1: Vec<&str> = esquema
        .campos
        .iter()
        .filter(|c| c.pagina == 1)
        .map(|c| c.clave)
        .collect();
    assert_eq!(en_pagina_1, ["eficiencia_max_pct", "eficiencia_euro_pct"]);
    assert!(panel().campos.iter().all(|c| c.pagina == 0));
}

#[test]
fn should_describe_the_27_shapes_of_the_3d_editor_when_their_entity_is_asked_for() {
    let v = crate::vista_agente::listar_esquemas(&json!({ "entidad": "escena_3d.formas" }))
        .expect("shape schema");
    let formas = v["formas"].as_array().expect("shapes");
    assert_eq!(formas.len(), 27);
    let tree = formas.iter().find(|f| f["tipo"] == "arbol").expect("tree");
    assert_eq!(tree["tipo_pvsyst"], "oeTree");
    assert_eq!(tree["campos"][0]["unidad"], "m");
    let comunes: Vec<&str> = v["comunes"]
        .as_array()
        .expect("common keys")
        .iter()
        .map(|c| c["clave"].as_str().unwrap_or(""))
        .collect();
    assert!(comunes.contains(&"rotacion_azimut_deg"));
}

#[test]
fn should_point_to_the_shape_entity_in_the_unfiltered_schema_listing() {
    let v = crate::vista_agente::listar_esquemas(&json!({})).expect("listing");
    assert_eq!(v["otras_entidades"][0], "escena_3d.formas");
}

/// The orientation and the array layout typed through `rellenar` and saved by PVsyst 8.1.8 with
/// «Guardar la variante activa» (tilt 25, azimuth 10; `proyecto dimensionar` for 50 kWp).
const VARIANTE_GUARDADA: &str =
    include_str!("../../fixtures/vista_agente/Variante_orientacion_sistema_8_1_8.VC0");

#[test]
fn should_find_every_file_key_of_the_orientation_and_system_schemas_in_the_variant_pvsyst_saved() {
    let claves: Vec<&str> = VARIANTE_GUARDADA
        .lines()
        .filter_map(|l| l.trim().split_once('=').map(|(k, _)| k))
        .collect();
    for entidad_clave in ["variante.orientation", "variante.system"] {
        let esquema = entidad(entidad_clave).expect("schema");
        for clave in esquema.campos.iter().filter_map(|c| c.clave_archivo) {
            assert!(
                claves.contains(&clave),
                "{clave} no está en la variante guardada"
            );
        }
    }
}

#[test]
fn should_keep_the_values_the_saved_variant_shows_for_what_was_typed() {
    for (clave, tecleado) in [
        ("FieldTilt", 25.0),
        ("FieldAzim", 10.0),
        ("NModSerie", 28.0),
        ("NStringCh", 4.0),
        ("NInverter", 1.0),
    ] {
        assert_eq!(
            valor_guardado(VARIANTE_GUARDADA, clave),
            Some(tecleado),
            "{clave}"
        );
    }
}

#[test]
fn should_accept_true_or_false_for_an_option_and_reject_anything_else() {
    let sistema = entidad("variante.system").expect("system schema");

    let bien = planificar(
        sistema,
        &valores(&json!({ "usar_potencia_planeada": true })),
    );
    let mal = planificar(
        sistema,
        &valores(&json!({ "usar_potencia_planeada": "si" })),
    );

    assert_eq!(bien.expect("plan")[0].texto, "true");
    assert!(matches!(mal, Err(Error::Invalid(_))));
}

#[test]
fn should_list_the_option_as_a_checkbox_type_and_keep_its_schema_unambiguous() {
    let sistema = entidad("variante.system").expect("system schema");

    let tipo = sistema.campo("usar_area_disponible").expect("option").tipo;

    assert_eq!(tipo.nombre(), "casilla");
}
