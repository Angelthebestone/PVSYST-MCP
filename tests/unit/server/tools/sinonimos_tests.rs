use super::*;

#[test]
fn should_accept_file_as_alias_for_archivo_in_leer() {
    let paths = Paths::resolve();
    let res = call(&paths, "leer", &json!({ "file": "Currency.dat" }));
    assert!(
        res.is_ok(),
        "alias 'file' should resolve to canonical 'archivo': {:?}",
        res.err()
    );
}

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
    let paths = Paths::resolve();
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
