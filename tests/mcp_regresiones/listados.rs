//! `listar` asks for the fields it needs: `campos` keeps only those in each item of
//! the list the chosen `que` answers, a name the `que` does not answer with is
//! refused with the ones it does, and leaving `campos` out keeps every field.

use super::cliente_mcp::{error_text, synthetic_workspace, tool_text, Client};
use serde_json::{json, Value};
use std::path::Path;

/// A field no `que` answers with, so a refusal has to name the ones it does.
const CAMPO_INEXISTENTE: &str = "campo_que_no_existe";

fn cliente(ws: &Path) -> Client {
    let ws_str = ws.to_string_lossy().to_string();
    let mut c = Client::start(Some(&ws_str));
    c.initialize();
    c
}

/// Every `que` of `listar` that answers a list of objects, with the arguments that
/// make the fixture workspace answer one and the lists each answer carries.
fn casos() -> Vec<(Value, &'static [&'static str])> {
    vec![
        (json!({ "que": "proyectos" }), &["projects"]),
        (
            json!({ "que": "componentes", "tipo": "PAN" }),
            &["components"],
        ),
        (json!({ "que": "modulos" }), &["modules"]),
        (
            json!({ "que": "archivos", "carpeta": "Projects" }),
            &["files"],
        ),
        (
            json!({ "que": "objetos", "archivo": "Projects/SYNTH_Plant.VC0" }),
            &["objects"],
        ),
    ]
}

/// The list of objects an answer carries, failing with the answer when it has none.
fn items_de<'a>(out: &'a Value, lista: &str) -> &'a [Value] {
    out.get(lista)
        .and_then(Value::as_array)
        .map(Vec::as_slice)
        .unwrap_or_else(|| panic!("la respuesta no lleva la lista '{lista}': {out}"))
}

/// The keys of an item, in the order the answer writes them.
fn claves_de(item: &Value) -> Vec<String> {
    item.as_object()
        .map(|mapa| mapa.keys().cloned().collect())
        .unwrap_or_default()
}

/// The first key every item of the list carries, which is what the filtered call
/// asks for: a module whose .PAN failed carries `error` where another carries `model`.
fn campo_comun(items: &[Value]) -> String {
    let primero = items
        .first()
        .and_then(Value::as_object)
        .expect("se esperaba una lista con al menos un elemento");
    primero
        .keys()
        .find(|clave| items.iter().all(|item| item.get(clave.as_str()).is_some()))
        .cloned()
        .expect("se esperaba una clave presente en todos los elementos")
}

/// Whether the answer names `campos` as an argument the chosen `que` did not read.
fn ignora_campos(out: &Value) -> bool {
    out.get("ignorados")
        .and_then(Value::as_array)
        .is_some_and(|claves| claves.iter().any(|clave| clave.as_str() == Some("campos")))
}

/// The same call with a `campos` that names a field no `que` answers with.
fn con_campo_inexistente(args: &Value) -> Value {
    let mut con = args.clone();
    con["campos"] = json!([CAMPO_INEXISTENTE]);
    con
}

/// The valid names a refusal lists, read from the tail of its message.
fn campos_del_error(msg: &str) -> Vec<&str> {
    msg.rsplit_once(": ")
        .map_or_else(Vec::new, |(_, cola)| cola.split(", ").collect())
}

#[test]
fn should_keep_only_the_named_field_of_each_item_in_every_listing() {
    let ws = synthetic_workspace();
    let mut c = cliente(ws.path());
    let mut id = 1;
    for (args, listas) in casos() {
        let sin_filtro = tool_text(&c.call(id, "listar", args.clone()));
        id += 1;
        for &lista in listas {
            let campo = campo_comun(items_de(&sin_filtro, lista));
            let mut con_campos = args.clone();
            con_campos["campos"] = json!([campo.as_str()]);
            let filtrado = tool_text(&c.call(id, "listar", con_campos));
            id += 1;
            assert!(!ignora_campos(&filtrado), "campos se aplicó: {filtrado}");
            for item in items_de(&filtrado, lista) {
                assert_eq!(
                    claves_de(item),
                    vec![campo.clone()],
                    "con que:{lista} solo se conserva '{campo}': {filtrado}"
                );
            }
            assert_eq!(
                filtrado["count"],
                sin_filtro["count"],
                "el contador no cambia: {filtrado}"
            );
        }
    }
}

#[test]
fn should_accept_a_field_that_only_some_answers_carry() {
    let ws = synthetic_workspace();
    let mut c = cliente(ws.path());
    // `aviso` only rides a project whose .PRJ failed to parse and `error` a module
    // whose .PAN failed: the fixture has neither, and both are still fields.
    let casos = [
        json!({ "que": "proyectos", "campos": ["aviso"] }),
        json!({ "que": "proyectos", "campos": ["variants"], "include_variants": false }),
        json!({ "que": "modulos", "campos": ["error"] }),
    ];
    let mut id = 30;
    for args in casos {
        let out = tool_text(&c.call(id, "listar", args));
        id += 1;
        assert!(!ignora_campos(&out), "campos se aplicó: {out}");
    }
}

#[test]
fn should_list_every_field_the_fixture_items_carry_when_campos_names_an_unknown_one() {
    let ws = synthetic_workspace();
    let mut c = cliente(ws.path());
    let mut id = 40;
    for (args, listas) in casos() {
        let sin_filtro = tool_text(&c.call(id, "listar", args.clone()));
        id += 1;
        let msg = error_text(&c.call(id, "listar", con_campo_inexistente(&args)));
        id += 1;
        assert!(
            msg.contains(CAMPO_INEXISTENTE),
            "el error nombra el campo pedido: {msg}"
        );
        let validos = campos_del_error(&msg);
        for &lista in listas {
            for item in items_de(&sin_filtro, lista) {
                for clave in claves_de(item) {
                    assert!(
                        validos.iter().any(|valido| *valido == clave.as_str()),
                        "con {args} la tabla omite '{clave}', que el elemento de '{lista}' lleva: {msg}"
                    );
                }
            }
        }
    }
}

#[test]
fn should_list_the_valid_fields_when_campos_names_an_unknown_one() {
    let ws = synthetic_workspace();
    let mut c = cliente(ws.path());
    let msg = error_text(&c.call(
        50,
        "listar",
        json!({ "que": "proyectos", "campos": ["name", "ruta_completa"] }),
    ));
    assert!(msg.contains("ruta_completa"), "{msg}");
    // A field this answer does not carry is still a field of the `que`: the list comes
    // from the contract, not from the items of the answer that is refused.
    for campo in [
        "name",
        "variants",
        "meteo_file",
        "modified_unix",
        "aviso",
        "solo_lectura",
    ] {
        assert!(
            msg.contains(campo),
            "el error lista los campos válidos: {msg}"
        );
    }
}

#[test]
fn should_refuse_a_campos_that_names_no_field_or_is_not_a_list_of_names() {
    let ws = synthetic_workspace();
    let mut c = cliente(ws.path());
    let casos = [
        (json!({ "que": "proyectos", "campos": [] }), "name"),
        (json!({ "que": "proyectos", "campos": [1] }), "string"),
        (json!({ "que": "proyectos", "campos": "name" }), "array"),
    ];
    let mut id = 60;
    for (args, esperado) in casos {
        let msg = error_text(&c.call(id, "listar", args));
        id += 1;
        assert!(
            msg.contains(esperado),
            "el error nombra lo que se esperaba ({esperado}): {msg}"
        );
    }
}

#[test]
fn should_keep_every_project_field_when_campos_is_absent() {
    let ws = synthetic_workspace();
    let mut c = cliente(ws.path());
    let out = tool_text(&c.call(70, "listar", json!({ "que": "proyectos" })));
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
