//! Answers that did not do the work yet: PVsyst's trial mode asks for a
//! confirmation, or the caller asked for a dry run. The next step is the same
//! call, completed.

use super::catalogo;
use super::construccion::Contexto;
use super::tipos::{ErrorVista, Estado, PasoSiguiente};
use crate::args::{flag, DRY_RUN, REQUIERE_CONFIRMACION};
use crate::idioma::{Idioma, Texto};

/// Answer key of the actions that validate without applying (`escena`, `rellenar`).
const SIMULADO: &str = "simulado";
use serde_json::{Map, Value};

const PRUEBA_CONFIRMAR: Texto = Texto {
    es: "Repite la misma llamada con confirmar:true (primer paso de siguiente).",
    en: "Repeat the same call with confirmar:true (first step of siguiente).",
};
/// Summary of a call that only asked for confirmation; `{mensaje}` is PVsyst's.
const NADA_EJECUTADO: Texto = Texto {
    es: "No se ejecutó nada: {mensaje}",
    en: "Nothing was run: {mensaje}",
};
/// Summary of a rehearsal; `{hecho}` is what the action would have done.
const ENSAYO: Texto = Texto {
    es: "Ensayo, no se aplicó nada. Al aplicarlo: {hecho}",
    en: "Rehearsal, nothing was applied. Applying it: {hecho}",
};
const CAUSA_DE_CONFIRMACION: Texto = Texto {
    es: "PVsyst está en modo PRUEBA y la acción gasta varias ejecuciones.",
    en: "PVsyst is in TRIAL mode and the action spends several runs.",
};
/// Reason of a confirmation request that does not say what it costs.
const GASTA_EJECUCIONES: Texto = Texto {
    es: "la acción gasta ejecuciones de prueba",
    en: "the action spends trial runs",
};
const MOTIVO_CONFIRMAR: Texto = Texto {
    es: "Confirmar el gasto de ejecuciones de prueba",
    en: "Confirm spending the trial runs",
};
const MOTIVO_APLICAR: Texto = Texto {
    es: "Aplicar de verdad lo ensayado",
    en: "Really apply what was rehearsed",
};
/// What a pending answer says it would have done when no catalogue entry describes it.
const ACCION_PEDIDA: Texto = Texto {
    es: "la acción pedida.",
    en: "the requested action.",
};
/// The argument that makes a call a dry run, per tool. It is not the same name
/// everywhere: in `disenar_sistema`, `simular` means "run the simulation".
const CLAVE_DE_ENSAYO: &[(&str, &str)] = &[
    ("disenar_sistema", "dry_run"),
    ("entorno", "dry_run"),
    ("pvsyst_cli", "dry_run"),
    ("proyecto", "dry_run"),
    ("simular", "dry_run"),
    ("sitio", "dry_run"),
    ("workspace", "dry_run"),
    ("gui_actuar", "simular"),
    // `editar` has no rehearsal: its handler refuses the switch before writing.
    ("editar", "dry_run"),
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
pub(super) fn pendiente_de(salida: &Value, estado: Estado) -> Option<Pendiente> {
    if flag(salida, &[REQUIERE_CONFIRMACION]) {
        return Some(Pendiente::Confirmacion);
    }
    let es_ensayo = estado == Estado::Ok && es_ensayo_respondido(salida);
    es_ensayo.then_some(Pendiente::Ensayo)
}

impl Pendiente {
    pub(super) fn resumen(self, ctx: &Contexto<'_>, salida: &Value) -> String {
        let hecho = catalogo::buscar(ctx.tool, ctx.accion).map_or(ACCION_PEDIDA, |e| e.resumen);
        match self {
            Self::Confirmacion => {
                let mensaje = salida.get("mensaje").and_then(Value::as_str).unwrap_or("");
                NADA_EJECUTADO.texto_con(ctx.idioma, &[("mensaje", mensaje)])
            }
            Self::Ensayo => ENSAYO.texto_con(ctx.idioma, &[("hecho", hecho.en_idioma(ctx.idioma))]),
        }
    }

    pub(super) fn error(self, salida: &Value, idioma: Idioma) -> Option<ErrorVista> {
        (self == Self::Confirmacion).then(|| ErrorVista {
            codigo: "requiere_confirmacion".into(),
            motivo: salida
                .get("mensaje")
                .and_then(Value::as_str)
                .unwrap_or(GASTA_EJECUCIONES.en_idioma(idioma))
                .to_string(),
            causa_probable: CAUSA_DE_CONFIRMACION.en_idioma(idioma),
            que_hacer: PRUEBA_CONFIRMAR.en_idioma(idioma),
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
                Self::Confirmacion => MOTIVO_CONFIRMAR,
                Self::Ensayo => MOTIVO_APLICAR,
            }
            .en_idioma(ctx.idioma),
        }
    }
}
