//! Alias and "required" annotations appended to property descriptions.

use super::{extraer_aliases, Anotador, Discriminador, Sinonimos};
use crate::server::tools::redaccion::{PREFIJO_VALORES, SUFIJO_REQUERIDO};
use serde_json::{json, Value};

impl Anotador<'_> {
    pub(super) fn propiedad(&self, prop_val: &mut Value, clave: &str) {
        let h = self.herramienta;
        let aliases = extraer_aliases(h.sinonimos.argumentos, clave);
        if aliases.is_empty() {
            return;
        }
        let Some(obj) = prop_val.as_object_mut() else {
            return;
        };
        let alias_str = format!("(alias: {})", aliases.join(", "));
        let mut desc = match obj.get("description").and_then(Value::as_str) {
            Some(prev) => format!("{prev} {alias_str}"),
            None => format!("{clave} {alias_str}"),
        };
        if h.requeridos.contains(&clave) {
            desc.push_str(SUFIJO_REQUERIDO.en_idioma(self.idioma));
        }
        obj.insert("description".to_string(), json!(desc));
    }

    pub(crate) fn discriminador(&self, prop: &mut Value, disc: &Discriminador) {
        let h = self.herramienta;
        let Some(obj) = prop.as_object_mut() else {
            return;
        };
        let valores_texto = obj
            .get("enum")
            .and_then(Value::as_array)
            .into_iter()
            .flatten()
            .filter_map(Value::as_str)
            .filter(|val| disc.valores.contains(val))
            .map(|val| formatear_valor_con_aliases(val, &h.sinonimos))
            .collect::<Vec<_>>()
            .join(", ");

        let arg_aliases = extraer_aliases(h.sinonimos.argumentos, disc.campo);

        let valores = PREFIJO_VALORES.en_idioma(self.idioma);
        let mut desc = if arg_aliases.is_empty() {
            format!("{valores}{valores_texto}")
        } else {
            format!(
                "{} (alias: {}). {valores}{valores_texto}",
                disc.campo,
                arg_aliases.join(", ")
            )
        };
        if disc.defecto.is_none() && !arg_aliases.is_empty() {
            desc.push_str(SUFIJO_REQUERIDO.en_idioma(self.idioma));
        }
        obj.insert("description".to_string(), json!(desc));
        h.anotar_requeridos_por_accion(prop, self.idioma);
    }
}

fn formatear_valor_con_aliases(val: &str, sinonimos: &Sinonimos) -> String {
    let aliases = extraer_aliases(sinonimos.valores, val);
    if aliases.is_empty() {
        val.to_string()
    } else {
        format!("{val} (alias: {})", aliases.join(", "))
    }
}
