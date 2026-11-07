//! Boundary validation for tool arguments: discriminator resolution, required
//! presence, and declared JSON types. Kept apart from the declarations so each
//! file stays small and the validation rules have one home.

use super::alias::{normalizar_alias_argumentos, normalizar_discriminador};
use super::declaracion::{nombre_tipo_recibido, Herramienta};
use super::tipos::validar_tipo_propiedad;
use crate::args::missing;
use crate::config::Paths;
use crate::error::{Error, Result};
use crate::idioma::{Idioma, Texto};
use serde_json::Value;

const VALOR_NO_VALIDO: Texto = Texto {
    es: "Valor '{valor}' no válido para {tool} ({campo}); valores: {valores}",
    en: "Value '{valor}' not valid for {tool} ({campo}); values: {valores}",
};
const ACCION_GATEADA: Texto = Texto {
    es: "Valor '{valor}' no válido para {tool} ({campo}): es una acción del MCP que solo existe con {variable}=1. Valores: {valores}",
    en: "Value '{valor}' not valid for {tool} ({campo}): it is an MCP in-house action that only exists with {variable}=1. Values: {valores}",
};
pub(super) const TIPO_INCORRECTO: Texto = Texto {
    es: "Tipo incorrecto para '{campo}' (esperado {esperado}, recibido {recibido})",
    en: "Wrong type for '{campo}' (expected {esperado}, got {recibido})",
};
const PISTA_DISCRIMINADOR: Texto = Texto {
    es: "para {tool}; valores: {valores}",
    en: "for {tool}; values: {valores}",
};

impl Herramienta {
    /// Validates arguments at the boundary and returns the resolved discriminator action.
    pub(crate) fn validar(&self, args: &Value, paths: &Paths) -> Result<String> {
        let accion = self.validar_discriminador(args, paths)?;
        self.validar_detalle(args, paths.idioma)?;
        self.validar_requeridos(args, &accion)?;
        self.validar_tipos(args, paths.idioma)?;
        self.validar_via(args, paths.idioma)?;
        Ok(accion)
    }

    /// Normalizes aliases to canonical names and injects default discriminator if omitted.
    pub(crate) fn preparar_args(&self, args: &Value, idioma: Idioma) -> Result<Value> {
        let mut mapa = match args {
            Value::Object(m) => m.clone(),
            _ => serde_json::Map::new(),
        };

        normalizar_alias_argumentos(self.sinonimos.argumentos, &mut mapa, idioma)?;
        normalizar_discriminador(
            self.discriminador.as_ref(),
            self.sinonimos.valores,
            &mut mapa,
        );

        Ok(Value::Object(mapa))
    }

    /// Resolves the discriminator: its value, its default, or the error naming
    /// the valid values of the tool.
    fn validar_discriminador(&self, args: &Value, paths: &Paths) -> Result<String> {
        let Some(disc) = &self.discriminador else {
            return Ok(String::new());
        };
        let idioma = paths.idioma;
        let valor = match args.get(disc.campo) {
            Some(v) => match v.as_str() {
                Some(s) => s.to_string(),
                None => {
                    return Err(Error::Invalid(TIPO_INCORRECTO.texto_con(
                        idioma,
                        &[
                            ("campo", disc.campo),
                            ("esperado", "string"),
                            ("recibido", nombre_tipo_recibido(v)),
                        ],
                    )));
                }
            },
            None => match disc.defecto {
                Some(def) => def.to_string(),
                None => {
                    let pista = PISTA_DISCRIMINADOR.texto_con(
                        idioma,
                        &[("tool", self.nombre), ("valores", &disc.valores.join(", "))],
                    );
                    return Err(missing(disc.campo, &pista));
                }
            },
        };

        if disc.valores.contains(&valor.as_str()) {
            return Ok(valor);
        }
        rechazar_accion_gateada(self, &valor, paths)?;
        let visibles: Vec<&str> = disc
            .valores
            .iter()
            .copied()
            .filter(|accion| {
                paths.aproximaciones || super::aproximadas::de(self.nombre, accion).is_none()
            })
            .collect();
        Err(Error::Invalid(VALOR_NO_VALIDO.texto_con(
            idioma,
            &[
                ("valor", &valor),
                ("tool", self.nombre),
                ("campo", disc.campo),
                ("valores", &visibles.join(", ")),
            ],
        )))
    }

    fn validar_detalle(&self, args: &Value, idioma: Idioma) -> Result<()> {
        let Some(v) = valor_no_nulo(args, "detalle") else {
            return Ok(());
        };
        let Some(d) = v.as_str() else {
            return Err(Error::Invalid(TIPO_INCORRECTO.texto_con(
                idioma,
                &[
                    ("campo", "detalle"),
                    ("esperado", "string"),
                    ("recibido", nombre_tipo_recibido(v)),
                ],
            )));
        };
        if !super::recorte::DETALLES.contains(&d) {
            return Err(Error::Invalid(VALOR_NO_VALIDO.texto_con(
                idioma,
                &[
                    ("valor", d),
                    ("tool", "detalle"),
                    ("campo", "detalle"),
                    ("valores", &super::recorte::DETALLES.join(", ")),
                ],
            )));
        }
        Ok(())
    }

    fn validar_via(&self, args: &Value, idioma: Idioma) -> Result<()> {
        let Some(v) = valor_no_nulo(args, "via") else {
            return Ok(());
        };
        let Some(via) = v.as_str() else {
            return Ok(());
        };
        if via == "pvsyst" || via == crate::aproximaciones::VIA_APROXIMACION {
            return Ok(());
        }
        Err(Error::Invalid(VALOR_NO_VALIDO.texto_con(
            idioma,
            &[
                ("valor", via),
                ("tool", "via"),
                ("campo", "via"),
                (
                    "valores",
                    &format!("pvsyst, {}", crate::aproximaciones::VIA_APROXIMACION),
                ),
            ],
        )))
    }

    fn validar_tipos(&self, args: &Value, idioma: Idioma) -> Result<()> {
        for prop in self.propiedades {
            validar_tipo_propiedad(prop, args, idioma)?;
        }
        Ok(())
    }
}

/// An action the server hides behind `PVSYST_MCP_APROXIMACIONES` is named as
/// such, so the caller can tell it from a typo.
fn rechazar_accion_gateada(h: &Herramienta, valor: &str, paths: &Paths) -> Result<()> {
    let Some(disc) = &h.discriminador else {
        return Ok(());
    };
    if super::aproximadas::de(h.nombre, valor).is_none() {
        return Ok(());
    }
    Err(Error::Invalid(ACCION_GATEADA.texto_con(
        paths.idioma,
        &[
            ("valor", valor),
            ("tool", h.nombre),
            ("campo", disc.campo),
            ("variable", crate::aproximaciones::VARIABLE_ENTORNO),
            ("valores", &disc.valores.join(", ")),
        ],
    )))
}

fn valor_no_nulo<'a>(args: &'a Value, clave: &str) -> Option<&'a Value> {
    args.get(clave).filter(|v| !v.is_null())
}
