use super::*;

#[test]
fn should_expose_all_fifteen_pvsyst_tools_by_default() {
    let paths = Paths::resolve();
    assert_eq!(list(&paths).len(), 15);
}

#[test]
fn should_include_permitir_fuera_del_workspace_property() {
    let paths = Paths::resolve();
    let schemas = list(&paths);
    for tool_name in [
        "entorno",
        "disenar_sistema",
        "listar",
        "leer",
        "editar",
        "proyecto",
        "componente",
        "sitio",
        "simular",
        "resultados",
        "workspace",
        "gui_observar",
        "gui_actuar",
    ] {
        let tool = schemas
            .iter()
            .find(|t| t["name"] == tool_name)
            .unwrap_or_else(|| panic!("missing {tool_name}"));
        assert_eq!(
            tool["inputSchema"]["properties"]["permitir_fuera_del_workspace"]["type"], "boolean",
            "{tool_name} must declare permitir_fuera_del_workspace as boolean"
        );
    }
}

#[test]
fn should_annotate_phase_three_schema_constraints() {
    let paths = Paths::resolve();
    let schemas = list(&paths);
    let proyecto = schemas
        .iter()
        .find(|t| t["name"] == "proyecto")
        .expect("proyecto");
    let potencia = &proyecto["inputSchema"]["properties"]["potencia_kw"];
    assert!(
        potencia["description"]
            .as_str()
            .unwrap()
            .contains("PLimGrid"),
        "potencia_kw must document removing limit"
    );

    let estrategia = &proyecto["inputSchema"]["properties"]["estrategia"];
    assert_eq!(
        estrategia["enum"],
        json!(["autoconsumo", "recorte_picos"]),
        "estrategia must declare enum"
    );

    let resultados = schemas
        .iter()
        .find(|t| t["name"] == "resultados")
        .expect("resultados");
    let medido = &resultados["inputSchema"]["properties"]["medido_kwh"];
    assert_eq!(medido["minItems"], 12);
    assert_eq!(medido["maxItems"], 12);
    assert_eq!(medido["items"]["type"], "number");

    let disenar = schemas
        .iter()
        .find(|t| t["name"] == "disenar_sistema")
        .expect("disenar_sistema");
    assert_eq!(
        disenar["inputSchema"]["properties"]["tipo_sistema"]["enum"],
        json!(["red", "aislado"])
    );
    assert_eq!(
        disenar["inputSchema"]["properties"]["n_baterias_serie"]["minimum"],
        1
    );
}

#[test]
fn should_expose_the_new_gui_field_actions() {
    let paths = Paths::resolve();
    let schemas = list(&paths);
    let tool = |name: &str| {
        schemas
            .iter()
            .find(|t| t["name"] == name)
            .unwrap_or_else(|| panic!("missing {name}"))
    };
    let actuar = tool("gui_actuar");
    let actuar_acciones = actuar["inputSchema"]["properties"]["accion"]["enum"]
        .as_array()
        .unwrap();
    for acc in [
        "funcion",
        "menu",
        "campo",
        "rellenar",
        "clic_texto",
        "reiniciar",
    ] {
        assert!(
            actuar_acciones.iter().any(|v| v == acc),
            "gui_actuar must expose {acc} in its schema"
        );
    }
    let actuar_props = &actuar["inputSchema"]["properties"];
    assert!(actuar_props.get("funcion").is_some());
    assert!(actuar_props.get("etiqueta").is_some());
    assert!(actuar_props.get("menu").is_some());
    assert!(actuar_props.get("item").is_some());
    assert!(actuar_props.get("entidad").is_some());
    assert!(actuar_props.get("valores").is_some());

    let observar = tool("gui_observar");
    let observar_que = observar["inputSchema"]["properties"]["que"]["enum"]
        .as_array()
        .unwrap();
    for q in ["funciones", "menus"] {
        assert!(
            observar_que.iter().any(|v| v == q),
            "gui_observar must expose {q} in its schema"
        );
    }
}

#[test]
fn should_expose_click_by_text_restart_and_menu_listing_in_the_gui_tools() {
    let paths = Paths::resolve();
    let schemas = list(&paths);
    let propiedades = |nombre: &str| {
        schemas
            .iter()
            .find(|t| t["name"] == nombre)
            .unwrap_or_else(|| panic!("missing {nombre}"))["inputSchema"]["properties"]
            .clone()
    };
    let actuar_acciones = propiedades("gui_actuar")["accion"]["enum"]
        .as_array()
        .cloned()
        .unwrap();
    assert!(actuar_acciones.iter().any(|v| v == "clic_texto"));
    assert!(actuar_acciones.iter().any(|v| v == "reiniciar"));

    let observar_que = propiedades("gui_observar")["que"]["enum"]
        .as_array()
        .cloned()
        .unwrap();
    assert!(observar_que.iter().any(|v| v == "menus"));
}

#[test]
fn should_match_manifest_tools_and_declarations() {
    let manifest_bytes = include_bytes!("../../../../mcpb/manifest.json");
    let manifest: Value = serde_json::from_slice(manifest_bytes).expect("valid manifest json");
    let manifest_tools = manifest["tools"].as_array().expect("manifest tools array");

    assert_eq!(
        manifest_tools.len(),
        REGISTRO.len(),
        "manifest tool count must match declaration count"
    );

    for h in REGISTRO {
        let m_tool = manifest_tools
            .iter()
            .find(|t| t["name"] == h.nombre)
            .unwrap_or_else(|| panic!("Tool '{}' declared but missing in manifest", h.nombre));
        assert_eq!(
            m_tool["description"].as_str().unwrap(),
            h.descripcion,
            "Description mismatch in manifest for tool '{}'",
            h.nombre
        );
    }

    let expected_aliases = tabla_alias();
    assert_eq!(
        manifest.get("tool_aliases"),
        Some(&expected_aliases),
        "mcpb/manifest.json tool_aliases must match tabla_alias()."
    );
}
