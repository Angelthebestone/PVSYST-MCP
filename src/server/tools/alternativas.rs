//! Required keys that are alternatives. `requeridos_por_accion` is a flat list,
//! so it cannot say "one of these" without demanding all of them; this table
//! names the groups, the schema lists them in the action's description and the
//! boundary check accepts any group that is complete.

use crate::error::Error;
use crate::idioma::{Idioma, Texto};
use serde_json::Value;

const CONECTOR_O: Texto = Texto {
    es: " o ",
    en: " or ",
};
const CONECTOR_Y: Texto = Texto {
    es: " y ",
    en: " and ",
};
const ALTERNATIVA: Texto = Texto {
    es: "o {claves}",
    en: "or {claves}",
};

/// One action whose required keys are alternatives: every key of at least one
/// `grupos` entry must be present.
pub(super) struct Alternativa {
    pub(super) herramienta: &'static str,
    pub(super) accion: &'static str,
    /// Keys that switch the action to a branch with its own requirements: when
    /// any is present the groups do not apply. Measured against the handler.
    pub(super) salvo_si: &'static [&'static str],
    pub(super) grupos: &'static [&'static [&'static str]],
}

/// The gaps measured against the handlers. The first group is the one the error
/// names and the example supplies.
pub(super) const ALTERNATIVAS: &[Alternativa] = &[
    Alternativa {
        herramienta: "simular",
        accion: "preparar",
        // `ejecutar_simular` sends `preparar` to the RVT manager when one of
        // these is present; that branch needs no rows or parameters.
        salvo_si: &["operacion", "batch_rvt_file", "archivo_rvt"],
        grupos: &[&["filas"], &["parametros"]],
    },
    Alternativa {
        herramienta: "workspace",
        accion: "exportar",
        // Without `destino` the zip lands next to the workspace, which is only
        // allowed with the confinement override.
        salvo_si: &[],
        grupos: &[&["destino"], &["permitir_fuera_del_workspace"]],
    },
    Alternativa {
        herramienta: "resultados",
        accion: "kpis",
        salvo_si: &[],
        grupos: &[&["proyecto"], &["archivo"], &["report_pdf"]],
    },
    Alternativa {
        herramienta: "resultados",
        accion: "resumen",
        salvo_si: &[],
        grupos: &[&["proyecto"], &["archivo"], &["report_pdf"]],
    },
    Alternativa {
        herramienta: "editar",
        accion: "campos",
        // `archivo` edits one PVObject file; `proyecto` + `ediciones` edits a
        // project variant.
        salvo_si: &[],
        grupos: &[&["archivo"], &["proyecto", "ediciones"]],
    },
];

/// The alternatives of `accion` for `herramienta`, if it has any.
pub(super) fn de(herramienta: &str, accion: &str) -> Option<&'static Alternativa> {
    ALTERNATIVAS
        .iter()
        .find(|a| a.herramienta == herramienta && a.accion == accion)
}

impl Alternativa {
    /// True when the action took another branch or one group is complete.
    pub(super) fn satisfecha(&self, args: &Value) -> bool {
        self.salvo_si.iter().any(|clave| !vacio(args, clave))
            || self
                .grupos
                .iter()
                .any(|grupo| grupo.iter().all(|clave| !vacio(args, clave)))
    }

    /// The missing group, with the other groups as its hint.
    pub(super) fn falta(&self, idioma: Idioma) -> Error {
        let resto: Vec<String> = self
            .grupos
            .iter()
            .skip(1)
            .map(|grupo| grupo_texto(grupo, idioma))
            .collect();
        Error::MissingArg {
            key: grupo_texto(self.grupos.first().copied().unwrap_or(&[]), idioma),
            hint: if resto.is_empty() {
                String::new()
            } else {
                ALTERNATIVA.texto_con(
                    idioma,
                    &[("claves", &resto.join(CONECTOR_O.en_idioma(idioma)))],
                )
            },
        }
    }

    /// "filas o parametros", for the action's description.
    pub(super) fn descripcion(&self, idioma: Idioma) -> String {
        self.grupos
            .iter()
            .map(|grupo| grupo_texto(grupo, idioma))
            .collect::<Vec<_>>()
            .join(CONECTOR_O.en_idioma(idioma))
    }
}

/// One group of keys that must all be present. Parentheses when there is more
/// than one, so `a o (b y c)` cannot be read as `(a o b) y c`.
fn grupo_texto(claves: &[&str], idioma: Idioma) -> String {
    let unidas = claves.join(CONECTOR_Y.en_idioma(idioma));
    if claves.len() > 1 {
        format!("({unidas})")
    } else {
        unidas
    }
}

fn vacio(args: &Value, clave: &str) -> bool {
    matches!(args.get(clave), None | Some(Value::Null))
}
