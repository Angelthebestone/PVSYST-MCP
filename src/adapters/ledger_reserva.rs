//! Trial reserve: a trial licence keeps a floor of executions, and a call that
//! would spend below that floor asks for confirmation even when it costs one run.
//! A licence has no budget, so the reserve never applies to it.

use crate::args::{flag, CONFIRMAR};
use crate::idioma::{Idioma, Texto};
use serde_json::{json, Value};
use std::path::Path;

use super::ledger::{estado_leido, Estado};

/// Estimated trial executions at or above which a trial run needs confirmation.
const UMBRAL_CONFIRMACION: u32 = 2;

/// Environment variable that sets how many trial executions stay in reserve.
pub(crate) const VARIABLE_RESERVA: &str = "PVSYST_RESERVA_TRIAL";

/// Reserve used when the variable is unset, blank or unusable.
pub(super) const RESERVA_POR_DEFECTO: u32 = 10;

const CONFIRMACION_TRIAL: Texto = Texto {
    es: "Esta acción consume {n} ejecuciones de prueba; repítela con confirmar:true",
    en: "This action spends {n} trial runs; repeat it with confirmar:true",
};
/// Confirmation whose cause is the reserve, not the number of runs.
const CONFIRMACION_RESERVA: Texto = Texto {
    es: "Esta acción gastaría {n} de las {restantes} ejecuciones de prueba que quedan y dejaría el saldo por debajo de la reserva de {reserva}; repítela con confirmar:true o baja {variable} (0 la desactiva)",
    en: "This action would spend {n} of the {restantes} trial runs left and leave the balance under the {reserva}-run reserve; repeat it with confirmar:true or lower {variable} (0 turns it off)",
};

/// Why a call that would spend trial runs is held back.
#[derive(Clone, Copy)]
enum Causa {
    /// It would leave the balance under the reserve.
    Reserva { restantes: u64, reserva: u32 },
    /// It spends [`UMBRAL_CONFIRMACION`] runs or more.
    Umbral,
}

/// A call that would spend trial executions, and the reserve that governs it.
pub(super) struct Gasto<'a> {
    estimadas: u32,
    plan: &'a [String],
    reserva: u32,
}

impl<'a> Gasto<'a> {
    /// `estimadas` runs of `plan`, held back only above `reserva` runs left so a
    /// test can state the reserve instead of reading the environment.
    pub(super) fn nuevo(estimadas: u32, plan: &'a [String], reserva: u32) -> Self {
        Self {
            estimadas,
            plan,
            reserva,
        }
    }
}

/// Reserve of [`VARIABLE_RESERVA`]'s value: blank counts as unset, and a value
/// that is not a whole number warns instead of quietly opening the budget.
pub(super) fn reserva_de(valor: Option<&str>) -> u32 {
    let Some(texto) = valor.map(str::trim).filter(|v| !v.is_empty()) else {
        return RESERVA_POR_DEFECTO;
    };
    texto.parse().unwrap_or_else(|_| {
        tracing::warn!(
            variable = VARIABLE_RESERVA,
            valor = texto,
            "no es un entero; uso la reserva por defecto"
        );
        RESERVA_POR_DEFECTO
    })
}

/// [`VARIABLE_RESERVA`] as it is set now, so a call follows it without a restart.
pub(super) fn reserva_de_entorno() -> u32 {
    reserva_de(std::env::var(VARIABLE_RESERVA).ok().as_deref())
}

/// Whether spending `estimadas` leaves fewer than `reserva` trial executions.
/// A ledger with no counter has nothing to compare: the threshold alone decides.
pub(super) fn bajo_reserva(restantes: Option<u64>, estimadas: u32, reserva: u32) -> bool {
    restantes.is_some_and(|r| r.saturating_sub(u64::from(estimadas)) < u64::from(reserva))
}

/// The reserve first: it is the stricter rule, and the one that names the balance.
fn causa(estado: &Estado, gasto: &Gasto<'_>) -> Option<Causa> {
    if bajo_reserva(estado.restantes, gasto.estimadas, gasto.reserva) {
        return estado.restantes.map(|restantes| Causa::Reserva {
            restantes,
            reserva: gasto.reserva,
        });
    }
    (gasto.estimadas >= UMBRAL_CONFIRMACION).then_some(Causa::Umbral)
}

fn mensaje(causa: Causa, estimadas: u32) -> String {
    let idioma = Idioma::de_entorno();
    let n = estimadas.to_string();
    match causa {
        Causa::Umbral => CONFIRMACION_TRIAL.texto_con(idioma, &[("n", &n)]),
        Causa::Reserva { restantes, reserva } => CONFIRMACION_RESERVA.texto_con(
            idioma,
            &[
                ("n", &n),
                ("restantes", &restantes.to_string()),
                ("reserva", &reserva.to_string()),
                ("variable", VARIABLE_RESERVA),
            ],
        ),
    }
}

/// The answer that replaces the run: the keys it always had, plus the `reserva` in
/// force, so the caller sees which number held it back.
fn retencion(estado: &Estado, gasto: &Gasto<'_>, causa: Causa) -> Value {
    json!({
        "requiere_confirmacion": true,
        "ejecuciones_estimadas": gasto.estimadas,
        "restantes": estado.restantes,
        "reserva": gasto.reserva,
        "plan": gasto.plan,
        "mensaje": mensaje(causa, gasto.estimadas),
    })
}

/// The gate itself, with the reserve given: several runs at once, or any run that
/// would eat into the reserve. A licence never reaches it.
pub(super) fn requiere_confirmacion_con(
    workspace: &Path,
    args: &Value,
    gasto: &Gasto<'_>,
) -> Option<Value> {
    // A call that estimates no run spends none (a cache hit): nothing to confirm.
    if gasto.estimadas == 0 || flag(args, &[CONFIRMAR]) {
        return None;
    }
    // An unreadable ledger may hide a trial licence: ask rather than spend blindly.
    let estado = match estado_leido(workspace) {
        Ok(Some(e)) if e.modo == "trial" => e,
        Ok(_) => return None,
        Err(e) => Estado {
            modo: format!("desconocido ({e})"),
            dias: None,
            restantes: None,
        },
    };
    Some(retencion(&estado, gasto, causa(&estado, gasto)?))
}
