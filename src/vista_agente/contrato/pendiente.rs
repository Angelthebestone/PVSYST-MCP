//! Answers that did not do the work yet: PVsyst's trial mode asks for a
//! confirmation, or the caller asked for a dry run. The next step is the same
//! call, completed.

use super::catalogo::buscar;
use super::construccion::Contexto;
use super::tipos::{ErrorVista, Estado, PasoSiguiente};
use crate::args::{flag, DRY_RUN, REQUIERE_CONFIRMACION};

/// Answer key of the actions that validate without applying (`escena`, `rellenar`).
const SIMULADO: &str = "simulado";
use serde_json::{Map, Value};

const PRUEBA_CONFIRMAR: &str =
    "Repite la misma llamada con confirmar:true (primer paso de siguiente).";
/// The argument that makes a call a dry run, per tool. It is not the same name
/// everywhere: in `disenar_sistema`, `simular` means "run the simulation".
const CLAVE_DE_ENSAYO: &[(&str, &str)] = &[
    ("entorno", "dry_run"),
    ("pvsyst_cli", "dry_run"),
    ("simular", "dry_run"),
    ("sitio", "dry_run"),
    ("gui_actuar", "simular"),
];

pub(super) fn clave_de_ensayo(tool: &str) -> Option<&'static str> {
    CLAVE_DE_ENSAYO
        .iter()
        .find(|(nombre, _)| *nombre == tool)
        .map(|(_, clave)| *clave)
}

/// Why an answer is not final.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum Pendiente {
    Confirmacion,
    Ensayo,
}

/// Only the answer says it was a dry run: a request that asked for one can still
/// have been executed for real by an action that never read the switch.
fn es_ensayo_respondido(salida: &Value) -> bool {
    flag(salida, &[DRY_RUN]) || flag(salida, &[SIMULADO])
}

/// Whether the answer is not final. A dry run is only a dry run when it worked: a
/// failed one has nothing to complete. `estado` is the state the answer would have.
pub(super) fn pendiente_de(
    ctx: &Contexto<'_>,
    salida: &Value,
    estado: Estado,
) -> Option<Pendiente> {
    if flag(salida, &[REQUIERE_CONFIRMACION]) {
        return Some(Pendiente::Confirmacion);
    }
    let sin_version_real = buscar(ctx.tool, ctx.accion).is_some_and(|e| e.solo_ensayo);
    let es_ensayo = estado == Estado::Ok && es_ensayo_respondido(salida) && !sin_version_real;
    es_ensayo.then_some(Pendiente::Ensayo)
}

impl Pendiente {
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
        if let (Self::Ensayo, Some(clave)) = (self, clave_de_ensayo(ctx.tool)) {
            argumentos.remove(clave);
        }
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
