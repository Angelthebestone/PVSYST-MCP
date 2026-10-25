//! Required-argument checks: every missing key is reported at once together with
//! a ready-to-send example, the schema lists the keys each action needs, and
//! undeclared arguments are reported back instead of being silently dropped.

use super::declaracion::Herramienta;
use crate::error::{Error, Result};
use serde_json::{json, Map, Value};

impl Herramienta {
    /// Rejects the call when any required key of `accion` is absent or null, naming all of them.
    pub(crate) fn validar_requeridos(&self, args: &Value, accion: &str) -> Result<()> {
        let todas = self.requeridos_de(accion);
        let claves: Vec<String> = todas
            .iter()
            .filter(|clave| esta_vacio_o_nulo(args, clave))
            .map(|clave| (*clave).to_string())
            .collect();
        if claves.is_empty() {
            return Ok(());
        }
        Err(Error::MissingArgs {
            claves,
            ejemplo: self.ejemplo(accion, &todas),
        })
    }

    /// Adds the per-action required keys of the admitted discriminator values to its description.
    pub(crate) fn anotar_requeridos_por_accion(&self, prop: &mut Value) {
        let admitidas: Vec<&str> = prop["enum"]
            .as_array()
            .into_iter()
            .flatten()
            .filter_map(Value::as_str)
            .collect();
        let detalle = self
            .requeridos_por_accion
            .iter()
            .filter(|(accion, _)| admitidas.contains(accion))
            .map(|(accion, claves)| format!("{accion} ({})", claves.join(", ")))
            .collect::<Vec<_>>()
            .join("; ");
        if detalle.is_empty() {
            return;
        }
        let previo = prop["description"].as_str().unwrap_or_default().to_owned();
        prop["description"] = json!(format!("{previo} Requeridos por acción: {detalle}."));
    }

    /// Argument keys the tool does not declare; the handler would never read them.
    pub(crate) fn desconocidos(&self, args: &Value) -> Vec<String> {
        let Value::Object(mapa) = args else {
            return Vec::new();
        };
        mapa.keys()
            .filter(|clave| !self.declara(clave))
            .cloned()
            .collect()
    }

    /// Every required key of `accion`: the tool-wide ones first, then the action's own.
    fn requeridos_de(&self, accion: &str) -> Vec<&'static str> {
        let propios = self
            .requeridos_por_accion
            .iter()
            .find(|(acc, _)| *acc == accion)
            .map_or(&[][..], |(_, claves)| *claves);
        self.requeridos.iter().chain(propios).copied().collect()
    }

    /// A call supplying every required key, with placeholders for the values.
    fn ejemplo(&self, accion: &str, claves: &[&str]) -> String {
        let discriminador = self
            .discriminador
            .as_ref()
            .filter(|_| !accion.is_empty())
            .map(|d| (d.campo.to_string(), json!(accion)));
        let resto = claves
            .iter()
            .map(|clave| ((*clave).to_string(), json!(format!("<{clave}>"))));
        let mapa: Map<String, Value> = discriminador.into_iter().chain(resto).collect();
        Value::Object(mapa).to_string()
    }

    fn declara(&self, clave: &str) -> bool {
        matches!(clave, "detalle" | "via")
            || self
                .discriminador
                .as_ref()
                .is_some_and(|d| d.campo == clave)
            || self.propiedades.iter().any(|(nombre, _)| *nombre == clave)
    }
}

fn esta_vacio_o_nulo(args: &Value, clave: &str) -> bool {
    matches!(args.get(clave), None | Some(Value::Null))
}
