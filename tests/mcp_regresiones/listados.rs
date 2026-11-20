//! `listar` asks for the fields it needs: `campos` keeps only those in each item of
//! every list the chosen `que` answers, a name the `que` does not answer with is
//! refused with the ones it does, and leaving `campos` out keeps every field.

use super::cliente_mcp::{error_text, synthetic_workspace, tool_text, Client};
use serde_json::{json, Value};
use std::path::Path;

/// A field no `que` answers with, so a refusal has to name the ones it does.
const CAMPO_INEXISTENTE: &str = "campo_que_no_existe";

/// A client on the fixture workspace, with a read-only PVsyst database that holds one
/// demo project: `que:proyectos` then answers both of its lists, `projects` and
/// `templates`. It also numbers the requests, so a test never counts them by hand.
struct Sesion {
    cliente: Client,
    siguiente: u64,
    _data_ro: tempfile::TempDir,
}

impl Sesion {
    fn nueva(ws: &Path) -> Self {
        let data_ro = tempfile::tempdir().expect("DataRO temporal");
        let demos = data_ro.path().join("PVsyst8.1_Data").join("Projects");
        std::fs::create_dir_all(&demos).expect("carpeta de demos");
        std::fs::copy(
            ws.join("Projects").join("SYNTH_Plant.PRJ"),
            demos.join("DEMO_Ref.PRJ"),
        )
        .expect("prj demo");
        let ws_str = ws.to_string_lossy().to_string();
        let data_str = data_ro.path().to_string_lossy().to_string();
        let mut cliente = Client::start_env(Some(&ws_str), &[("PVSYST_DATARO", data_str.as_str())]);
        cliente.initialize();
        Self {
            cliente,
            siguiente: 2,
            _data_ro: data_ro,
        }
    }

    /// The raw answer of one `listar` call.
    fn listar(&mut self, args: Value) -> Value {
        let id = self.siguiente;
        self.siguiente += 1;
        self.cliente.call(id, "listar", args)
    }

    /// The answer of a call that succeeds.
    fn tool(&mut self, args: Value) -> Value {
        tool_text(&self.listar(args))
    }

    /// The text of a call that is refused.
    fn error(&mut self, args: Value) -> String {
        error_text(&self.listar(args))
    }
}

/// Every `que` of `listar` that answers a list of objects, with the arguments that
/// make the fixture workspace answer one and the lists each answer carries.
fn casos() -> Vec<(Value, &'static [&'static str])> {
    vec![
        (json!({ "que": "proyectos" }), &["projects", "templates"]),
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

/// The same call with `campos` set to `campos`.
fn con_campos(args: &Value, campos: Value) -> Value {
    let mut con = args.clone();
    if let Some(mapa) = con.as_object_mut() {
        mapa.insert("campos".to_string(), campos);
    }
    con
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

/// The valid names a refusal lists, read from the tail of its message.
fn campos_del_error(msg: &str) -> Vec<&str> {
    msg.rsplit_once(": ")
        .map_or_else(Vec::new, |(_, cola)| cola.split(", ").collect())
}

/// The items of `lista` that carry a key other than `campo`.
fn items_con_otras_claves(out: &Value, lista: &str, campo: &str) -> Vec<Value> {
    items_de(out, lista)
        .iter()
        .filter(|item| claves_de(item) != [campo])
        .cloned()
        .collect()
}

/// The keys the items of `listas` carry that the refusal does not list as valid.
fn claves_sin_tabla(sin_filtro: &Value, listas: &[&str], validos: &[&str]) -> Vec<String> {
    listas
        .iter()
        .flat_map(|lista| items_de(sin_filtro, lista))
        .flat_map(claves_de)
        .filter(|clave| !validos.contains(&clave.as_str()))
        .collect()
}

#[test]
fn should_keep_only_the_named_field_of_each_item_in_every_listing() {
    let ws = synthetic_workspace();
    let mut s = Sesion::nueva(ws.path());
    for (args, listas) in casos() {
        let sin_filtro = s.tool(args.clone());
        for &lista in listas {
            let campo = campo_comun(items_de(&sin_filtro, lista));
            let filtrado = s.tool(con_campos(&args, json!([campo.as_str()])));
            assert!(!ignora_campos(&filtrado), "campos se aplicó: {filtrado}");
            let otras = items_con_otras_claves(&filtrado, lista, &campo);
            assert!(
                otras.is_empty(),
                "con que:{lista} solo se conserva '{campo}': {otras:?}"
            );
            assert_eq!(
                filtrado["count"], sin_filtro["count"],
                "el contador no cambia: {filtrado}"
            );
        }
    }
}

#[test]
fn should_accept_a_field_that_only_some_answers_carry() {
    let ws = synthetic_workspace();
    let mut s = Sesion::nueva(ws.path());
    // `aviso` only rides a project whose .PRJ failed to parse and `error` a module
    // whose .PAN failed: the fixture has neither, and both are still fields.
    let casos = [
        json!({ "que": "proyectos", "campos": ["aviso"] }),
        json!({ "que": "proyectos", "campos": ["variants"], "include_variants": false }),
        json!({ "que": "proyectos", "campos": ["solo_lectura"] }),
        json!({ "que": "modulos", "campos": ["error"] }),
    ];
    for args in casos {
        let out = s.tool(args);
        assert!(!ignora_campos(&out), "campos se aplicó: {out}");
    }
}

#[test]
fn should_list_every_field_the_fixture_items_carry_when_campos_names_an_unknown_one() {
    let ws = synthetic_workspace();
    let mut s = Sesion::nueva(ws.path());
    for (args, listas) in casos() {
        let sin_filtro = s.tool(args.clone());
        let msg = s.error(con_campos(&args, json!([CAMPO_INEXISTENTE])));
        assert!(
            msg.contains(CAMPO_INEXISTENTE),
            "el error nombra el campo pedido: {msg}"
        );
        let faltan = claves_sin_tabla(&sin_filtro, listas, &campos_del_error(&msg));
        assert!(
            faltan.is_empty(),
            "con {args} la tabla omite {faltan:?}: {msg}"
        );
    }
}

#[test]
fn should_list_the_valid_fields_when_campos_names_an_unknown_one() {
    let ws = synthetic_workspace();
    let mut s = Sesion::nueva(ws.path());
    let msg = s.error(json!({ "que": "proyectos", "campos": ["name", "ruta_completa"] }));
    assert!(msg.contains("ruta_completa"), "{msg}");
    // A field this answer does not carry is still a field of the `que`: the list comes
    // from the contract, not from the items of the answer that is refused.
    let validos = campos_del_error(&msg);
    for campo in [
        "name",
        "variants",
        "meteo_file",
        "modified_unix",
        "aviso",
        "solo_lectura",
    ] {
        assert!(
            validos.contains(&campo),
            "el error lista los campos válidos: {msg}"
        );
    }
}

#[test]
fn should_refuse_a_campos_that_names_no_field_or_is_not_a_list_of_names() {
    let ws = synthetic_workspace();
    let mut s = Sesion::nueva(ws.path());
    let casos = [
        (json!({ "que": "proyectos", "campos": [] }), "name"),
        (json!({ "que": "proyectos", "campos": [1] }), "string"),
        (json!({ "que": "proyectos", "campos": "name" }), "array"),
    ];
    for (args, esperado) in casos {
        let msg = s.error(args);
        assert!(
            msg.contains(esperado),
            "el error nombra lo que se esperaba ({esperado}): {msg}"
        );
    }
}

#[test]
fn should_keep_every_project_field_when_campos_is_absent() {
    let ws = synthetic_workspace();
    let mut s = Sesion::nueva(ws.path());
    let out = s.tool(json!({ "que": "proyectos" }));
    let claves = items_de(&out, "projects").first().map(claves_de);
    let claves = claves.expect("un proyecto");
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
            claves.iter().any(|clave| clave == campo),
            "sin campos falta '{campo}': {out}"
        );
    }
}
