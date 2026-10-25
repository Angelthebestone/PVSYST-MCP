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
fn should_list_the_required_keys_of_each_action_in_the_discriminator() {
    let paths = Paths::resolve();
    let schemas = list(&paths);
    let editar = schemas
        .iter()
        .find(|t| t["name"] == "editar")
        .expect("editar schema");
    let descripcion = editar["inputSchema"]["properties"]["accion"]["description"]
        .as_str()
        .expect("discriminator description");
    assert!(
        descripcion.contains("insertar_objeto (archivo, padre, desde)"),
        "{descripcion}"
    );
    assert!(descripcion.contains("rvt (operacion)"), "{descripcion}");
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
}

#[test]
fn should_declare_items_for_all_array_schemas() {
    let mut paths = Paths::resolve();
    paths.aproximaciones = false;
    for tool in list(&paths) {
        let name = tool["name"].as_str().unwrap_or("unknown");
        assert_arrays_have_items(&tool, name);
    }

    paths.aproximaciones = true;
    for tool in list(&paths) {
        let name = tool["name"].as_str().unwrap_or("unknown");
        assert_arrays_have_items(&tool, name);
    }
}

fn assert_arrays_have_items(schema: &Value, path: &str) {
    match schema {
        Value::Object(obj) => check_object_arrays(obj, path),
        Value::Array(arr) => check_array_elements(arr, path),
        _ => {}
    }
}

fn check_object_arrays(obj: &serde_json::Map<String, Value>, path: &str) {
    if let Some(tipo) = obj.get("type") {
        validar_items_si_es_array(tipo, obj, path);
    }
    for (k, v) in obj {
        assert_arrays_have_items(v, &format!("{path}.{k}"));
    }
}

fn validar_items_si_es_array(tipo: &Value, obj: &serde_json::Map<String, Value>, path: &str) {
    let es_array = match tipo {
        Value::String(s) => s == "array",
        Value::Array(arr) => arr.iter().any(|v| v == "array"),
        _ => false,
    };
    if es_array {
        assert!(
            obj.get("items").is_some(),
            "Array schema at '{path}' is missing 'items': {obj:?}"
        );
    }
}

fn check_array_elements(arr: &[Value], path: &str) {
    for (i, v) in arr.iter().enumerate() {
        assert_arrays_have_items(v, &format!("{path}[{i}]"));
    }
}
