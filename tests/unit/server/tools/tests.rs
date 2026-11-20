use super::*;
use crate::config::Paths;
use crate::progreso::SeguimientoTarea;
use serde_json::{json, Value};

fn call(paths: &Paths, name: &str, args: &Value) -> Result<Value> {
    call_seguido(&Solicitud {
        paths,
        name,
        args,
        seguimiento: &SeguimientoTarea::inerte(),
    })
}

mod aproximaciones_tests;
mod manifiesto_tests;
mod recorte_tests;
mod sinonimos_tests;
mod validacion_tests;

#[test]
fn should_keep_the_ignored_keys_table_in_sync_with_the_tool_declarations() {
    for entrada in super::no_leidos::NO_LEIDOS {
        let h = REGISTRO
            .iter()
            .find(|h| h.nombre == entrada.herramienta)
            .unwrap_or_else(|| {
                panic!(
                    "tabla de ignorados: tool desconocida '{}'",
                    entrada.herramienta
                )
            });
        // A tool without a discriminator has a single, unnamed action.
        let acciones: &[&str] = h.discriminador.as_ref().map_or(&[""], |d| d.valores);
        assert!(
            acciones.contains(&entrada.accion),
            "{}: acción desconocida '{}'",
            entrada.herramienta,
            entrada.accion
        );
        assert!(
            h.declara(entrada.clave),
            "{}: '{}' no está declarada",
            entrada.herramienta,
            entrada.clave
        );
        let consumidas: &[&str] = h
            .consumidos_por_accion
            .iter()
            .find(|(accion, _)| *accion == entrada.accion)
            .map_or(&[], |(_, claves)| *claves);
        assert!(
            !consumidas.contains(&entrada.clave),
            "{} {}: '{}' está declarada como consumida y como ignorada",
            entrada.herramienta,
            entrada.accion,
            entrada.clave
        );
        for condicion in entrada.leido_si {
            assert!(
                h.declara(condicion),
                "{} {}: la condición '{}' no está declarada",
                entrada.herramienta,
                entrada.accion,
                condicion
            );
        }
    }
}

#[test]
fn should_not_report_a_batch_key_as_ignored_when_the_argument_that_reads_it_is_present() {
    let simular = REGISTRO
        .iter()
        .find(|h| h.nombre == "simular")
        .expect("simular");

    let con_filas = json!({ "accion": "ejecutar", "filas": [{}], "variables": ["E_Grid"] });
    assert!(
        !simular
            .desconocidos(&con_filas, "ejecutar")
            .contains(&"variables".to_string()),
        "'variables' se lee cuando hay 'filas'"
    );

    let sin_filas = json!({ "accion": "ejecutar", "variables": ["E_Grid"] });
    assert!(
        simular
            .desconocidos(&sin_filas, "ejecutar")
            .contains(&"variables".to_string()),
        "sin 'filas' el handler no lee 'variables'"
    );
}

/// The decorator reads `vista`, not a handler: answering it as ignored would say
/// the switch was not applied when it was.
#[test]
fn should_not_report_the_view_switch_as_an_ignored_argument() {
    for h in REGISTRO {
        let args = json!({ "vista": false });
        assert!(
            !h.desconocidos(&args, "").contains(&"vista".to_string()),
            "{}: 'vista' reaches the caller as ignored",
            h.nombre
        );
    }
}

/// `gui_observar` has one schema for every `que`. The wire test names what an
/// `esquema` call ignores; `inventario` and the `que` that do read `compacto` and
/// `recorte` need an open window there, so that part is pinned here (B.5.4 and D.5).
#[test]
fn should_name_detalle_as_ignored_by_inventario_and_not_the_keys_the_other_que_read() {
    let observar = REGISTRO
        .iter()
        .find(|h| h.nombre == "gui_observar")
        .expect("gui_observar");

    let inventario = json!({ "que": "inventario", "detalle": "normal" });
    assert!(
        observar
            .desconocidos(&inventario, "inventario")
            .contains(&"detalle".to_string()),
        "el inventario se limita a sí mismo y 'detalle' no cambia nada"
    );

    for (que, clave) in [
        ("ventanas", "compacto"),
        ("controles", "compacto"),
        ("vista", "compacto"),
        ("capturar", "recorte"),
    ] {
        let mut args = json!({ "que": que });
        args[clave] = json!(true);
        let ignorados = observar.desconocidos(&args, que);
        assert!(!ignorados.contains(&clave.to_string()), "{que} lee {clave}");
    }
}

#[test]
fn should_declare_every_key_a_tool_consumes_for_another_action() {
    let consumos = REGISTRO.iter().flat_map(|h| {
        h.consumidos_por_accion
            .iter()
            .map(move |(accion, claves)| (h, *accion, *claves))
    });
    for (h, accion, claves) in consumos {
        let acciones: &[&str] = h.discriminador.as_ref().map_or(&[], |d| d.valores);
        assert!(
            acciones.contains(&accion),
            "{}: acción desconocida '{accion}'",
            h.nombre
        );
        let sin_declarar: Vec<&str> = claves.iter().copied().filter(|c| !h.declara(c)).collect();
        assert!(
            sin_declarar.is_empty(),
            "{} {accion}: el esquema no declara {sin_declarar:?}",
            h.nombre
        );
    }
}
