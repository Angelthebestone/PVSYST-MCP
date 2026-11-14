//! Messages a caller reads to fix its own call: a "not found" that names what
//! does exist, a refusal that names the missing key, and a hint that names a
//! parameter the tool really accepts.

use super::cliente_mcp::{error_text, synthetic_workspace, tool_text, Client};
use serde_json::json;

fn servidor(ws: &tempfile::TempDir) -> Client {
    let mut c = Client::start(Some(&ws.path().to_string_lossy()));
    c.initialize();
    c
}

#[test]
fn should_name_the_projects_that_exist_when_leer_misses_one() {
    let ws = synthetic_workspace();
    let mut c = servidor(&ws);

    let msg = error_text(&c.call(10, "leer", json!({ "archivo": "Projects/NO_EXISTE.PRJ" })));

    assert!(
        !msg.contains("no hay ninguno"),
        "afirma que no hay proyectos cuando sí los hay: {msg}"
    );
    assert!(
        msg.contains("SYNTH_Plant"),
        "nombra los proyectos que sí están: {msg}"
    );
}

#[test]
fn should_say_that_the_project_exists_when_crear_duplicates_it() {
    let ws = synthetic_workspace();
    let mut c = servidor(&ws);

    let msg = error_text(&c.call(
        11,
        "proyecto",
        json!({ "accion": "crear", "plantilla": "SYNTH_Plant", "nombre_nuevo": "SYNTH_Plant" }),
    ));

    assert!(
        msg.contains("already exists"),
        "the default language is English: {msg}"
    );
    assert!(msg.contains("sobrescribir"), "nombra la alternativa: {msg}");
}

#[test]
fn should_say_the_call_asked_for_one_variant_when_comparar_gets_only_one() {
    let ws = synthetic_workspace();
    let mut c = servidor(&ws);

    let msg = error_text(&c.call(
        12,
        "resultados",
        json!({ "que": "comparar", "proyecto": "SYNTH_Plant", "variantes": ["VC0"] }),
    ));

    assert!(
        msg.contains("1 were requested"),
        "el mensaje debe hablar de lo que pidió la llamada, no del proyecto: {msg}"
    );
}

#[test]
fn should_name_desde_archivo_when_insertar_objeto_lacks_it() {
    let ws = synthetic_workspace();
    let mut c = servidor(&ws);

    let msg = error_text(&c.call(
        13,
        "editar",
        json!({
            "accion": "insertar_objeto",
            "archivo": "Projects/SYNTH_Plant.VC0",
            "padre": "pvSubArray",
            "desde": { "objeto": "pvOrient" },
        }),
    ));

    assert!(
        msg.contains("desde.archivo"),
        "nombra la clave que falta, no el 'archivo' de nivel superior: {msg}"
    );
}

#[test]
fn should_point_the_leer_hint_at_detalle_completo() {
    let ws = synthetic_workspace();
    let mut c = servidor(&ws);

    let leido = c.call(14, "leer", json!({ "archivo": "Projects/SYNTH_Plant.VC0" }));
    let data = tool_text(&leido);
    let hint = data["hint"].as_str().unwrap_or_default();
    assert!(
        hint.contains("detalle"),
        "el hint nombra el parámetro que existe: {hint}"
    );
    assert!(
        !hint.contains("completo:true"),
        "el hint no inventa 'completo': {hint}"
    );

    // The hint must work as written: the same call with `detalle` returns the tree.
    let completo = tool_text(&c.call(
        15,
        "leer",
        json!({ "archivo": "Projects/SYNTH_Plant.VC0", "detalle": "completo" }),
    ));
    assert!(
        completo["tree"].is_array(),
        "detalle:completo devuelve el árbol: {completo}"
    );
}
