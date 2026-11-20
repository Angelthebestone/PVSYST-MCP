//! `listar` asks for the fields it needs: `campos` keeps only those in each item,
//! a name that is not a field is refused with the ones that are, and leaving
//! `campos` out keeps every field.

use super::cliente_mcp::{error_text, synthetic_workspace, tool_text, Client};
use serde_json::{json, Value};
use std::path::Path;

fn cliente(ws: &Path) -> Client {
    let ws_str = ws.to_string_lossy().to_string();
    let mut c = Client::start(Some(&ws_str));
    c.initialize();
    c
}

/// The first key of the item, which is the field the filtered call asks for.
fn primer_campo(item: &Value) -> String {
    item.as_object()
        .and_then(|mapa| mapa.keys().next().cloned())
        .expect("una lista con al menos un elemento")
}

fn claves_de(item: &Value) -> Vec<String> {
    item.as_object()
        .map(|mapa| mapa.keys().cloned().collect())
        .unwrap_or_default()
}

#[test]
fn should_keep_only_the_named_field_of_each_project_when_campos_asks_for_it() {
    let ws = synthetic_workspace();
    let mut c = cliente(ws.path());
    let sin_filtro = tool_text(&c.call(40, "listar", json!({ "que": "proyectos" })));
    let item = sin_filtro["projects"][0].clone();
    let campo = primer_campo(&item);

    let filtrado = tool_text(&c.call(
        41,
        "listar",
        json!({ "que": "proyectos", "campos": [campo.as_str()] }),
    ));

    assert_eq!(
        claves_de(&filtrado["projects"][0]),
        vec![campo.clone()],
        "conserva solo '{campo}': {filtrado}"
    );
    assert_eq!(
        filtrado["projects"][0][campo.as_str()],
        item[campo.as_str()],
        "el valor del campo se conserva: {filtrado}"
    );
    assert_eq!(
        filtrado["count"], sin_filtro["count"],
        "el contador no cambia: {filtrado}"
    );
    assert!(
        filtrado.get("ignorados").is_none(),
        "campos se aplicó y no se ignora: {filtrado}"
    );
}

#[test]
fn should_keep_only_the_named_field_of_each_item_in_every_other_listing() {
    let ws = synthetic_workspace();
    let mut c = cliente(ws.path());
    let casos = [
        (
            42,
            json!({ "que": "componentes", "tipo": "PAN" }),
            "components",
        ),
        (43, json!({ "que": "modulos" }), "modules"),
        (
            44,
            json!({ "que": "archivos", "carpeta": "Projects" }),
            "files",
        ),
        (
            45,
            json!({ "que": "objetos", "archivo": "Projects/SYNTH_Plant.VC0" }),
            "objects",
        ),
    ];
    for (id, args, lista) in casos {
        let sin_filtro = tool_text(&c.call(id, "listar", args.clone()));
        let campo = primer_campo(&sin_filtro[lista][0]);

        let mut con_campos = args.clone();
        con_campos["campos"] = json!([campo.as_str()]);
        let filtrado = tool_text(&c.call(id + 100, "listar", con_campos));

        assert_eq!(
            claves_de(&filtrado[lista][0]),
            vec![campo.clone()],
            "que:{lista} conserva solo '{campo}': {filtrado}"
        );
    }
}

#[test]
fn should_list_the_valid_fields_when_campos_names_an_unknown_one() {
    let ws = synthetic_workspace();
    let mut c = cliente(ws.path());
    let msg = error_text(&c.call(
        46,
        "listar",
        json!({ "que": "proyectos", "campos": ["name", "ruta_completa"] }),
    ));
    assert!(msg.contains("ruta_completa"), "{msg}");
    for campo in ["name", "variants", "meteo_file", "modified_unix"] {
        assert!(
            msg.contains(campo),
            "el error lista los campos válidos: {msg}"
        );
    }
}

#[test]
fn should_keep_every_project_field_when_campos_is_absent() {
    let ws = synthetic_workspace();
    let mut c = cliente(ws.path());
    let out = tool_text(&c.call(47, "listar", json!({ "que": "proyectos" })));
    let item = out["projects"][0].as_object().expect("un proyecto");
    for campo in [
        "name",
        "prj",
        "modified_unix",
        "comment",
        "version",
        "meteo_file",
        "variants",
    ] {
        assert!(
            item.contains_key(campo),
            "sin campos falta '{campo}': {out}"
        );
    }
}
