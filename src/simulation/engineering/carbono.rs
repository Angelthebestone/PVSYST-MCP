//! Carbon balance read directly from the simulation PDF report.

use super::caso::{caso, con_cabecera};
use crate::config::Paths;
use crate::error::Result;
use crate::simulation::report_pdf;
use serde_json::{json, Value};

pub(crate) fn carbono(paths: &Paths, args: &Value) -> Result<Value> {
    let c = caso(paths, args)?;
    let mut v = report_pdf::carbono(&c.tokens)?;
    if let Some(obj) = v.as_object_mut() {
        obj.insert("origen".into(), json!("pvsyst"));
    }
    Ok(con_cabecera(&c, v))
}
