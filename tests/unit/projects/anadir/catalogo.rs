use super::*;

fn demo(nombre: &str) -> Vec<u8> {
    std::fs::read(crate::config::proyectos_demo_ro().join(nombre)).expect("PVsyst demo data")
}

#[test]
fn should_list_elements_when_aplicar_elemento_gets_an_unknown_name() {
    let error = aplicar_elemento(Vec::new(), "nada").expect_err("elemento desconocido");
    assert!(error.to_string().contains("baterias"));
}

#[test]
#[ignore = "needs the PVsyst 8.1.7 demo data: run with --ignored"]
fn should_have_parent_in_demo_for_every_element() {
    for elemento in ELEMENTOS {
        let fuente = demo(elemento.demo);
        let spans = edit::spans_of(&fuente);
        for tipo in elemento.tipos {
            let si = edit::find_span(&spans, tipo)
                .unwrap_or_else(|| panic!("{tipo} ausente en {}", elemento.demo));
            assert!(
                padre_de(&spans, si).is_some(),
                "{tipo} sin padre en {}",
                elemento.demo
            );
        }
    }
}

#[test]
fn should_list_valid_elements_when_unknown_element_requested() {
    let mut paths = Paths::resolve();
    paths.workspace = std::env::temp_dir();
    let args = json!({"proyecto": "x", "elemento": "nada"});
    let err = anadir(&paths, &args).unwrap_err();
    assert!(err.to_string().contains("baterias"));
}

#[test]
#[ignore = "needs the PVsyst 8.1.7 demo data: run with --ignored"]
fn should_not_contain_meteonorm_in_any_catalogue_element() {
    for elemento in ELEMENTOS {
        let fuente = demo(elemento.demo);
        for tipo in elemento.tipos {
            let bloque = insert::extraer_objeto(&fuente, tipo).expect("bloque");
            let bloque = if elemento.nombre == "horizonte" {
                neutralizar_horizonte(&bloque).expect("neutralizado")
            } else {
                bloque
            };
            let texto = String::from_utf8_lossy(&bloque).to_ascii_lowercase();
            assert!(
                !texto.contains("meteonorm"),
                "{tipo} de {} contiene Meteonorm",
                elemento.demo
            );
        }
    }
}

#[test]
#[ignore = "needs the PVsyst 8.1.7 demo data: run with --ignored"]
fn should_insert_domo_and_produce_valid_structure() {
    let fuente = demo("_DEMO_Residential.VC0");
    let aplicado = aplicar_elemento(fuente, "domo").expect("aplicar domo");
    crate::format::object::validar_estructura(&aplicado.bytes).expect("estructura valida");
    let spans = crate::format::edit::spans_of(&aplicado.bytes);
    assert!(spans.iter().any(|s| s.ty == "pvDomeOrient"));
    assert!(crate::projects::doctor::sintaxis::buscar_duplicados(&aplicado.bytes).is_empty());
}

#[test]
#[ignore = "needs the PVsyst 8.1.7 demo data: run with --ignored"]
fn should_insert_zona_campo_and_produce_valid_structure() {
    let fuente = demo("_DEMO_Residential.VC0");
    let aplicado = aplicar_elemento(fuente, "zona_campo").expect("aplicar zona_campo");
    crate::format::object::validar_estructura(&aplicado.bytes).expect("estructura valida");
    let spans = crate::format::edit::spans_of(&aplicado.bytes);
    assert!(spans.iter().any(|s| s.ty == "pvShdFieldZone"));
}

#[test]
#[ignore = "needs the PVsyst 8.1.7 demo data: run with --ignored"]
fn should_insert_bifacial_seguidor_and_produce_valid_structure() {
    let fuente = demo("_DEMO_Tracking.VC0");
    let aplicado =
        aplicar_elemento(fuente, "bifacial_seguidor").expect("aplicar bifacial_seguidor");
    crate::format::object::validar_estructura(&aplicado.bytes).expect("estructura valida");
    let spans = crate::format::edit::spans_of(&aplicado.bytes);
    assert!(spans.iter().any(|s| s.ty == "pvBiFacial"));
    assert!(spans.iter().any(|s| s.ty == "pvBiFacialTrackers"));
}

#[test]
#[ignore = "needs the PVsyst 8.1.7 demo data: run with --ignored"]
fn should_insert_circuito_hidraulico_and_produce_valid_structure() {
    let fuente = demo("_DEMO_Residential.VC0");
    let aplicado =
        aplicar_elemento(fuente, "circuito_hidraulico").expect("aplicar circuito_hidraulico");
    crate::format::object::validar_estructura(&aplicado.bytes).expect("estructura valida");
    let spans = crate::format::edit::spans_of(&aplicado.bytes);
    assert!(spans.iter().any(|s| s.ty == "pvHydrCircuit"));
    let texto = String::from_utf8_lossy(&aplicado.bytes);
    assert!(texto.contains("SystemType=Pumping"));
}

#[test]
#[ignore = "needs the PVsyst 8.1.7 demo data: run with --ignored"]
fn should_not_flag_errors_in_doctor_when_domo_added_to_variant_copy() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let projects_dir = tmp.path().join("Projects");
    std::fs::create_dir_all(&projects_dir).expect("create Projects dir");

    let demo_prj = demo("_DEMO_Residential.PRJ");
    let demo_vc0 = demo("_DEMO_Residential.VC0");
    std::fs::write(projects_dir.join("TestDoctor.PRJ"), demo_prj).expect("write PRJ");
    std::fs::write(projects_dir.join("TestDoctor.VC0"), demo_vc0).expect("write VC0");

    let mut paths = Paths::resolve();
    paths.workspace = tmp.path().to_path_buf();

    let args_anadir = json!({
        "proyecto": "TestDoctor",
        "variante": "VC0",
        "elemento": "domo"
    });
    let res_anadir = anadir(&paths, &args_anadir).expect("anadir domo");
    assert_eq!(res_anadir["ok"], true);

    let args_doctor = json!({
        "proyecto": "TestDoctor",
        "variante": "VC0"
    });
    let res_doc = crate::projects::doctor::doctor(&paths, &args_doctor).expect("doctor");
    let errores = res_doc["errores"].as_array().expect("array errores");
    assert!(errores.is_empty(), "Doctor encontro errores: {errores:?}");
}
