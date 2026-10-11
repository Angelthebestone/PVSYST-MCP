//! Catalogue filters for `listar_modulos`: nominal power range and free text
//! over file name, vendor and model. Module metadata is parsed by `pan.rs`.

use crate::error::Result;
use serde_json::{json, Value};

/// Filters accepted by `listar_modulos` (all optional).
pub(crate) struct Criterios {
    pnom_min: Option<f64>,
    pnom_max: Option<f64>,
    texto: Option<String>,
}

/// Number from a JSON number or a numeric string (decimal comma allowed).
pub(crate) fn numero(v: &Value) -> Option<f64> {
    v.as_f64().or_else(|| {
        v.as_str()
            .and_then(|s| s.trim().replace(',', ".").parse::<f64>().ok())
    })
}

/// Module metadata shown in the listing; a parse failure becomes `error`.
fn metadatos(parsed: &Result<Value>) -> Value {
    match parsed {
        Ok(v) => json!({
            "pnom_w": numero(&v["pnom"]).map_or(Value::Null, |p| json!(p)),
            "vendor": v["vendor"],
            "model": v["model"],
        }),
        Err(e) => json!({ "error": e }),
    }
}

impl Criterios {
    /// Reads `pnom_min`, `pnom_max` and `texto` from the tool arguments.
    pub(crate) fn desde(args: &Value) -> Criterios {
        Criterios {
            pnom_min: args.get("pnom_min").and_then(numero),
            pnom_max: args.get("pnom_max").and_then(numero),
            texto: args
                .get("texto")
                .and_then(Value::as_str)
                .map(|s| s.trim().to_lowercase())
                .filter(|s| !s.is_empty()),
        }
    }

    /// Listing entry for one module when it passes every active filter.
    pub(crate) fn admite(&self, rel: &str, parsed: &Result<Value>) -> Option<Value> {
        let meta = metadatos(parsed);
        let pnom = meta["pnom_w"].as_f64();
        let ok_min = self.pnom_min.is_none_or(|m| pnom.is_some_and(|p| p >= m));
        let ok_max = self.pnom_max.is_none_or(|m| pnom.is_some_and(|p| p <= m));
        let ok_texto = match &self.texto {
            Some(t) => [
                rel,
                meta["vendor"].as_str().unwrap_or(""),
                meta["model"].as_str().unwrap_or(""),
            ]
            .iter()
            .any(|s| s.to_lowercase().contains(t.as_str())),
            None => true,
        };
        if !(ok_min && ok_max && ok_texto) {
            return None;
        }
        let mut entrada = json!({ "modulo": rel });
        if let (Value::Object(dst), Value::Object(src)) = (&mut entrada, meta) {
            dst.extend(src);
        }
        Some(entrada)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::error::Error;

    fn modulo(pnom: f64, vendor: &str) -> Result<Value> {
        Ok(json!({"pnom": pnom, "vendor": vendor, "model": "X1"}))
    }

    #[test]
    fn numero_accepts_decimal_comma_strings() {
        assert_eq!(numero(&json!("3,5")), Some(3.5));
        assert_eq!(numero(&json!(410)), Some(410.0));
        assert_eq!(numero(&json!("n/a")), None);
    }

    #[test]
    fn rango_de_potencia_excluye_modulos_fuera() {
        let c = Criterios::desde(&json!({"pnom_min": 350}));
        assert!(c.admite("a.pan", &modulo(300.0, "Acme")).is_none());
        assert!(c.admite("b.pan", &modulo(400.0, "Acme")).is_some());
    }

    #[test]
    fn texto_busca_en_fabricante_sin_distinguir_mayusculas() {
        let c = Criterios::desde(&json!({"texto": "ACME"}));
        let hit = c
            .admite("a.pan", &modulo(300.0, "Acme Solar"))
            .unwrap_or(Value::Null);
        assert_eq!(hit["vendor"], json!("Acme Solar"));
        assert!(c.admite("a.pan", &modulo(300.0, "Otro")).is_none());
    }

    #[test]
    fn modulo_ilegible_solo_pasa_sin_filtro_de_potencia() {
        let error: Result<Value> = Err(Error::Format("corrupto".into()));
        assert!(Criterios::desde(&json!({}))
            .admite("x.pan", &error)
            .is_some());
        assert!(Criterios::desde(&json!({"pnom_max": 500}))
            .admite("x.pan", &error)
            .is_none());
    }
}
