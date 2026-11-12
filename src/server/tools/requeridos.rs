//! Required-argument checks: every missing key is reported at once together with
//! a ready-to-send example, the schema lists the keys each action needs, and
//! undeclared arguments are reported back instead of being silently dropped.

use super::declaracion::{extraer_aliases, Herramienta};
use super::no_leidos::no_leidos;
use super::redaccion::PREFIJO_REQUERIDOS_POR_ACCION;
use crate::error::{Error, Result};
use crate::idioma::Idioma;
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
    pub(crate) fn anotar_requeridos_por_accion(&self, prop: &mut Value, idioma: Idioma) {
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
        let prefijo = PREFIJO_REQUERIDOS_POR_ACCION.en_idioma(idioma);
        prop["description"] = json!(format!("{previo}{prefijo}{detalle}."));
    }

    /// Top-level `required` keeps the keys without aliases. Each aliased required key
    /// becomes an `anyOf` over its spellings, so a validating client accepts either one.
    pub(crate) fn anotar_requisitos(&self, esquema: &mut Value) {
        let discriminador = self
            .discriminador
            .as_ref()
            .filter(|d| d.defecto.is_none())
            .map(|d| d.campo);
        let (simples, con_alias): (Vec<&str>, Vec<&str>) = self
            .requeridos
            .iter()
            .copied()
            .chain(discriminador)
            .partition(|clave| extraer_aliases(self.sinonimos.argumentos, clave).is_empty());
        esquema["required"] = json!(simples);
        let alternativas: Vec<Value> = con_alias
            .iter()
            .map(|clave| self.alternativa_con_alias(clave))
            .collect();
        if !alternativas.is_empty() {
            esquema["allOf"] = json!(alternativas);
        }
    }

    fn alternativa_con_alias(&self, clave: &str) -> Value {
        let nombres =
            std::iter::once(clave).chain(extraer_aliases(self.sinonimos.argumentos, clave));
        let alternativas: Vec<Value> = nombres.map(|n| json!({ "required": [n] })).collect();
        json!({ "anyOf": alternativas })
    }

    /// Argument keys the handler never looks at: the ones the tool does not declare
    /// and the declared ones `accion` does not read.
    pub(crate) fn desconocidos(&self, args: &Value, accion: &str) -> Vec<String> {
        let Value::Object(mapa) = args else {
            return Vec::new();
        };
        let consumidos = self.consumidos(accion);
        let no_leidos = no_leidos(self.nombre, accion, args);
        mapa.keys()
            .filter(|clave| {
                no_leidos.contains(&clave.as_str())
                    || (!self.declara(clave) && !consumidos.contains(&clave.as_str()))
            })
            .cloned()
            .collect()
    }

    /// Keys `accion` reads that the tool declares for its other actions only.
    fn consumidos(&self, accion: &str) -> &'static [&'static str] {
        self.consumidos_por_accion
            .iter()
            .find(|(acc, _)| *acc == accion)
            .map_or(&[], |(_, claves)| *claves)
    }

    /// Every required key of `accion`: the tool-wide ones first, then the action's own.
    pub(super) fn requeridos_de(&self, accion: &str) -> Vec<&'static str> {
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

    pub(super) fn declara(&self, clave: &str) -> bool {
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
