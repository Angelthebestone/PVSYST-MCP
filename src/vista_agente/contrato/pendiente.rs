//! Answers that did not do the work yet: PVsyst's trial mode asks for a
//! confirmation, or the caller asked for a dry run. The next step is the same
//! call, completed.

use super::construccion::Contexto;
use super::tipos::{ErrorVista, Estado, PasoSiguiente};
use serde_json::{Map, Value};

const PRUEBA_CONFIRMAR: &str =
    "Repite la misma llamada con confirmar:true (primer paso de siguiente).";
/// Argument that makes a call a dry run, per tool (`gui_actuar` calls it `simular`).
const CLAVES_DE_ENSAYO: &[&str] = &["dry_run", "simular"];

/// Why an answer is not final.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum Pendiente {
    Confirmacion,
    Ensayo,
}

fn es_verdadero(valor: Option<&Value>) -> bool {
    valor.and_then(Value::as_bool) == Some(true)
}

fn pide_ensayo(ctx: &Contexto<'_>, salida: &Value) -> bool {
    es_verdadero(salida.get("dry_run"))
        || CLAVES_DE_ENSAYO
            .iter()
            .any(|clave| es_verdadero(ctx.args.get(clave)))
}

pub(super) fn pendiente_de(ctx: &Contexto<'_>, salida: &Value) -> Option<Pendiente> {
    if es_verdadero(salida.get("requiere_confirmacion")) {
        return Some(Pendiente::Confirmacion);
    }
    pide_ensayo(ctx, salida).then_some(Pendiente::Ensayo)
}

impl Pendiente {
    pub(super) fn estado(self) -> Estado {
        match self {
            Self::Confirmacion => Estado::Parcial,
            Self::Ensayo => Estado::Ok,
        }
    }

    pub(super) fn resumen(self, hecho: &str, salida: &Value) -> String {
        match self {
            Self::Confirmacion => {
                let mensaje = salida.get("mensaje").and_then(Value::as_str).unwrap_or("");
                format!("No se ejecutó nada: {mensaje}")
            }
            Self::Ensayo => format!("Ensayo, no se aplicó nada. Al aplicarlo: {hecho}"),
        }
    }

    pub(super) fn error(self, salida: &Value) -> Option<ErrorVista> {
        (self == Self::Confirmacion).then(|| ErrorVista {
            codigo: "requiere_confirmacion".into(),
            motivo: salida
                .get("mensaje")
                .and_then(Value::as_str)
                .unwrap_or("la acción gasta ejecuciones de prueba")
                .to_string(),
            causa_probable: "PVsyst está en modo PRUEBA y la acción gasta varias ejecuciones.",
            que_hacer: PRUEBA_CONFIRMAR,
            claves_validas: Vec::new(),
        })
    }

    /// The same call, completed: confirmed, or without the dry-run switch.
    pub(super) fn siguiente(self, ctx: &Contexto<'_>) -> PasoSiguiente {
        let mut argumentos: Map<String, Value> = ctx.args.as_object().cloned().unwrap_or_default();
        argumentos
            .retain(|clave, _| self != Self::Ensayo || !CLAVES_DE_ENSAYO.contains(&clave.as_str()));
        if self == Self::Confirmacion {
            argumentos.insert("confirmar".into(), Value::Bool(true));
        }
        PasoSiguiente {
            tool: ctx.tool.to_string(),
            argumentos: Value::Object(argumentos),
            motivo: match self {
                Self::Confirmacion => "Confirmar el gasto de ejecuciones de prueba",
                Self::Ensayo => "Aplicar de verdad lo ensayado",
            },
        }
    }
}
