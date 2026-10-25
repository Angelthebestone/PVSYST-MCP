use super::*;

#[test]
fn should_produce_same_preparar_args_for_alias_and_canonical() {
    let tool_leer = REGISTRO.iter().find(|h| h.nombre == "leer").unwrap();
    let args_can = json!({ "archivo": "Demo.PRJ" });
    let args_ali = json!({ "file": "Demo.PRJ" });
    assert_eq!(
        tool_leer.preparar_args(&args_can).unwrap(),
        tool_leer.preparar_args(&args_ali).unwrap()
    );

    let tool_listar = REGISTRO.iter().find(|h| h.nombre == "listar").unwrap();
    let args_can = json!({ "que": "modulos" });
    let args_ali = json!({ "que": "modules" });
    assert_eq!(
        tool_listar.preparar_args(&args_can).unwrap(),
        tool_listar.preparar_args(&args_ali).unwrap()
    );

    let tool_comp = REGISTRO.iter().find(|h| h.nombre == "componente").unwrap();
    let args_can = json!({ "accion": "crear", "plantilla": "A", "nombre_nuevo": "B" });
    let args_ali = json!({ "action": "create", "template": "A", "new_name": "B" });
    assert_eq!(
        tool_comp.preparar_args(&args_can).unwrap(),
        tool_comp.preparar_args(&args_ali).unwrap()
    );
}

#[test]
fn should_reject_conflicting_alias_and_canonical_with_different_values() {
    let tool_leer = REGISTRO.iter().find(|h| h.nombre == "leer").unwrap();
    let conflict = json!({ "archivo": "a.txt", "file": "b.txt" });
    let err = tool_leer
        .preparar_args(&conflict)
        .expect_err("conflicting alias and canonical must fail");
    let msg = err.to_string();
    assert!(
        msg.contains("Conflicto de argumentos") && msg.contains("archivo") && msg.contains("file"),
        "error must explain the conflict: {msg}"
    );

    let same_value = json!({ "archivo": "same.txt", "file": "same.txt" });
    assert!(
        tool_leer.preparar_args(&same_value).is_ok(),
        "identical values for alias and canonical should succeed"
    );
}

#[test]
fn should_reject_unknown_discriminator_value_and_list_admitted() {
    let paths = Paths::resolve();
    let err = call(&paths, "entorno", &json!({ "accion": "inexistente" }))
        .expect_err("unknown discriminator value should be rejected");
    let msg = err.to_string();
    assert!(
        msg.contains("Valor 'inexistente' no válido para entorno (accion)"),
        "error must identify invalid value: {msg}"
    );
    assert!(
        msg.contains("diagnostico") && msg.contains("licencia"),
        "error must list admitted values: {msg}"
    );
}

#[test]
fn should_announce_every_declared_alias_in_schema() {
    let mut paths = Paths::resolve();
    paths.aproximaciones = true;
    let schemas = list(&paths);

    for h in REGISTRO {
        let schema = schemas
            .iter()
            .find(|s| s["name"] == h.nombre)
            .unwrap_or_else(|| panic!("missing schema for {}", h.nombre));
        let props = &schema["inputSchema"]["properties"];
        verificar_alias_argumentos(h, props);
        verificar_alias_discriminador(h, props);
    }
}

fn verificar_alias_argumentos(h: &Herramienta, props: &Value) {
    for &(alias, canonico) in h.sinonimos.argumentos {
        let prop = props.get(canonico).unwrap_or_else(|| {
            panic!(
                "Tool '{}': alias '{}' points to undeclared property '{}'",
                h.nombre, alias, canonico
            )
        });
        let desc = prop["description"].as_str().unwrap();
        assert!(
            desc.contains(alias),
            "Tool '{}': property '{}' schema must announce alias '{}'",
            h.nombre,
            canonico,
            alias
        );
    }
}

fn verificar_alias_discriminador(h: &Herramienta, props: &Value) {
    let Some(disc) = &h.discriminador else {
        return;
    };
    let desc = props[disc.campo]["description"].as_str().unwrap();
    for &(val_alias, val_can) in h.sinonimos.valores {
        assert!(
            desc.contains(val_alias),
            "Tool '{}': discriminator schema must announce alias '{}' for '{}'",
            h.nombre,
            val_alias,
            val_can
        );
    }
}

#[test]
fn should_verify_all_tools_have_synonym_tables_without_argument_collisions() {
    for h in REGISTRO {
        verificar_colisiones_argumentos(h);
        verificar_colisiones_valores(h);
    }
}

fn verificar_colisiones_argumentos(h: &Herramienta) {
    for &(alias, canonico) in h.sinonimos.argumentos {
        assert_ne!(alias, canonico);
        assert!(
            h.propiedades.iter().any(|(p, _)| *p == canonico)
                || h.discriminador
                    .as_ref()
                    .is_some_and(|d| d.campo == canonico)
        );
        assert!(
            !h.propiedades.iter().any(|(p, _)| *p == alias),
            "Tool '{}': alias '{}' collides with an already declared property",
            h.nombre,
            alias
        );
    }
}

fn verificar_colisiones_valores(h: &Herramienta) {
    let Some(disc) = &h.discriminador else {
        return;
    };
    for &(alias, canonico) in h.sinonimos.valores {
        assert_ne!(alias, canonico);
        assert!(disc.valores.contains(&canonico));
    }
}

#[test]
fn should_not_corrupt_declared_property_source_in_sitio() {
    let tool_sitio = REGISTRO.iter().find(|h| h.nombre == "sitio").unwrap();
    let args = json!({ "accion": "crear", "source": "NASA SSE" });
    let preparados = tool_sitio.preparar_args(&args).expect("preparar_args");
    assert_eq!(preparados.get("source"), Some(&json!("NASA SSE")));
    assert_eq!(preparados.get("origen"), None);
}

#[test]
fn should_normalize_simulation_short_flags_and_reject_conflicting_values() {
    let tool_sim = REGISTRO.iter().find(|h| h.nombre == "simular").unwrap();
    let args_alias = json!({ "csv": "out.csv", "pdf": "out.pdf", "sitio": "Demo" });
    let norm = tool_sim.preparar_args(&args_alias).expect("preparar_args");
    assert_eq!(norm.get("output_csv"), Some(&json!("out.csv")));
    assert_eq!(norm.get("report_pdf"), Some(&json!("out.pdf")));
    assert_eq!(norm.get("site"), Some(&json!("Demo")));
    assert_eq!(norm.get("csv"), None);

    let mut cli_args = Vec::new();
    crate::simulation::preparacion::push_options(&norm, &mut cli_args);
    assert_eq!(cli_args, vec!["-ocf:out.csv", "-rpf:out.pdf", "-s:Demo"]);

    let conflict = json!({ "output_csv": "a.csv", "csv": "b.csv" });
    assert!(tool_sim.preparar_args(&conflict).is_err());
}

#[test]
fn should_accept_loose_aliases_in_sitio_and_disenar_sistema() {
    let tool_sitio = REGISTRO.iter().find(|h| h.nombre == "sitio").unwrap();
    let args_sitio = json!({
        "accion": "crear",
        "fuente": "NASA SSE",
        "zona_horaria": 1.0,
        "sit_salida": "demo.SIT",
        "cof": "opts.txt"
    });
    let norm_sitio = tool_sitio
        .preparar_args(&args_sitio)
        .expect("preparar_args sitio");
    assert_eq!(norm_sitio.get("source"), Some(&json!("NASA SSE")));
    assert_eq!(norm_sitio.get("timezone"), Some(&json!(1.0)));
    assert_eq!(norm_sitio.get("output_sit_file"), Some(&json!("demo.SIT")));
    assert_eq!(norm_sitio.get("cmd_options_file"), Some(&json!("opts.txt")));

    let tool_dis = REGISTRO
        .iter()
        .find(|h| h.nombre == "disenar_sistema")
        .unwrap();
    let args_dis = json!({
        "latitude": 10.0,
        "longitude": 20.0,
        "potencia_kwp": 5.0,
        "inclinacion": 25.0,
        "azimuth": 0.0
    });
    let norm_dis = tool_dis
        .preparar_args(&args_dis)
        .expect("preparar_args disenar");
    assert_eq!(norm_dis.get("latitud"), Some(&json!(10.0)));
    assert_eq!(norm_dis.get("longitud"), Some(&json!(20.0)));
    assert_eq!(norm_dis.get("kwp"), Some(&json!(5.0)));
    assert_eq!(norm_dis.get("tilt"), Some(&json!(25.0)));
    assert_eq!(norm_dis.get("azimut"), Some(&json!(0.0)));
}

#[test]
fn should_have_synchronized_alias_blocks_in_readmes_and_manifest() {
    let bloque = generar_bloque_alias_markdown();
    let readme_en = include_str!("../../../../README.md");
    let readme_es = include_str!("../../../../README.es.md");
    assert!(
        readme_en.contains(&bloque),
        "README.md does not contain the generated alias block. Regenerate with generar_bloque_alias_markdown()."
    );
    assert!(
        readme_es.contains(&bloque),
        "README.es.md does not contain the generated alias block. Regenerate with generar_bloque_alias_markdown()."
    );

    let manifest_bytes = include_bytes!("../../../../mcpb/manifest.json");
    let manifest: Value = serde_json::from_slice(manifest_bytes).expect("valid manifest json");
    let expected_aliases = tabla_alias();
    assert_eq!(
        manifest.pointer("/_meta/pvsyst-mcp/tool_aliases"),
        Some(&expected_aliases),
        "mcpb/manifest.json _meta.pvsyst-mcp.tool_aliases does not match tabla_alias(). Regenerate it."
    );
}
