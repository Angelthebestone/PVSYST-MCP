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
    for entrada in super::requeridos::NO_LEIDOS {
        let h = REGISTRO
            .iter()
            .find(|h| h.nombre == entrada.herramienta)
            .unwrap_or_else(|| {
                panic!(
                    "tabla de ignorados: tool desconocida '{}'",
                    entrada.herramienta
                )
            });
        let acciones: &[&str] = h.discriminador.as_ref().map_or(&[], |d| d.valores);
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
        if let Some(condicion) = entrada.leido_si {
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

#[test]
fn should_declare_every_key_a_tool_consumes_for_another_action() {
    for h in REGISTRO {
        for (accion, claves) in h.consumidos_por_accion {
            let acciones: &[&str] = h.discriminador.as_ref().map_or(&[], |d| d.valores);
            assert!(
                acciones.contains(accion),
                "{}: acción desconocida '{accion}'",
                h.nombre
            );
            for clave in *claves {
                assert!(
                    h.declara(clave),
                    "{} {accion}: '{clave}' se consume pero el esquema no la declara",
                    h.nombre
                );
            }
        }
    }
}
