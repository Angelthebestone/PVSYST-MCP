//! Economic evaluation read directly from the simulation PDF report.

use super::caso::{caso, con_cabecera, Caso};
use crate::config::Paths;
use crate::error::Result;
use crate::simulation::report_pdf;
use serde_json::{json, Value};

pub(crate) fn economia(paths: &Paths, args: &Value) -> Result<Value> {
    let c = caso(paths, args)?;
    economia_de(&c)
}

/// Economic evaluation from a case whose report is already parsed.
pub(crate) fn economia_de(c: &Caso) -> Result<Value> {
    let mut v = report_pdf::economia(&c.tokens)?;
    if let Some(obj) = v.as_object_mut() {
        obj.insert("origen".into(), json!("pvsyst"));
    }
    Ok(con_cabecera(c, v))
}
