//! Deserialization of PVsyst `VarMeteo` hourly series blocks.

use super::base62::{decodificar_linea_dia, TOKENS_POR_DIA};
use super::dominio::{crudo_a_fisico, HoraValores, MetadatosVariable, SerieHoraria, TipoVariable};
use crate::error::{Error, Result};
use std::collections::HashMap;

const SEPARADOR_SIGNO_VALOR: char = '=';
const PREFIJO_VAR_SIMUL: &str = "VarSimul_";
const PREFIJO_DAY: &str = "Day_";
const MARCADOR_END_DATA: &str = "End of Data";
const MARCADOR_END_VAR: &str = "End of VarSimul";
const MARCADOR_END_LIST: &str = "End of the List VarMeteo";
const MARCADOR_VAR_METEO: &str = "VarMeteo, list of=";

struct BloqueVariableBruto {
    nombre: String,
    indice: usize,
    header_lines: Vec<String>,
    day_lines: Vec<String>,
}

struct DatosConstruccionHoras<'a> {
    variables: &'a [String],
    metadatos: &'a [MetadatosVariable],
    valores_crudos: &'a HashMap<String, Vec<i64>>,
    dias: &'a [String],
}

/// Decodes the `VarMeteo` section of a .MET or .DAM file into a `SerieHoraria`.
pub(crate) fn decodificar(cuerpo: &str) -> Result<SerieHoraria> {
    let seccion = aislar_seccion_varmeteo(cuerpo);
    let mut lineas_iter = seccion.lines();
    let top_line = lineas_iter
        .next()
        .ok_or_else(|| Error::Format("Cuerpo de serie horaria vacio".into()))?
        .to_string();

    let bloques = agrupar_bloques_variables(lineas_iter)?;
    if bloques.is_empty() {
        return Err(Error::Format(
            "No se encontraron variables en VarMeteo".into(),
        ));
    }

    let mut variables = Vec::with_capacity(bloques.len());
    let mut metadatos = Vec::with_capacity(bloques.len());
    let mut valores_crudos = HashMap::with_capacity(bloques.len());
    let mut dias = Vec::new();

    for bloque in bloques {
        let (meta, raws, dias_var) = procesar_bloque_variable(bloque)?;
        if dias.is_empty() {
            dias = dias_var;
        }
        variables.push(meta.nombre.clone());
        valores_crudos.insert(meta.nombre.clone(), raws);
        metadatos.push(meta);
    }

    let datos = DatosConstruccionHoras {
        variables: &variables,
        metadatos: &metadatos,
        valores_crudos: &valores_crudos,
        dias: &dias,
    };
    let horas = construir_horas(&datos)?;

    Ok(SerieHoraria {
        top_line,
        variables,
        metadatos,
        dias,
        horas,
        valores_crudos,
    })
}

fn aislar_seccion_varmeteo(cuerpo: &str) -> &str {
    let Some(pos) = cuerpo.find(MARCADOR_VAR_METEO) else {
        return cuerpo;
    };
    let inicio = cuerpo[..pos].rfind('\n').map_or(0, |p| p + 1);
    &cuerpo[inicio..]
}

fn agrupar_bloques_variables<'a>(
    lineas: impl Iterator<Item = &'a str>,
) -> Result<Vec<BloqueVariableBruto>> {
    let mut bloques = Vec::new();
    let mut actual: Option<BloqueVariableBruto> = None;

    for linea in lineas {
        let s = linea.trim();
        if s.is_empty() || s.starts_with(MARCADOR_END_LIST) {
            continue;
        }
        if let Some(rest) = s.strip_prefix(PREFIJO_VAR_SIMUL) {
            if let Some((idx_str, nom)) = rest.split_once(SEPARADOR_SIGNO_VALOR) {
                if let Some(b) = actual.take() {
                    bloques.push(b);
                }
                let idx = idx_str.parse().unwrap_or(bloques.len() + 1);
                actual = Some(BloqueVariableBruto {
                    nombre: nom.trim().to_string(),
                    indice: idx,
                    header_lines: Vec::new(),
                    day_lines: Vec::new(),
                });
                continue;
            }
        }
        let Some(b) = actual.as_mut() else {
            continue;
        };
        if s.starts_with(PREFIJO_DAY) {
            b.day_lines.push(s.to_string());
        } else if !s.starts_with(MARCADOR_END_DATA) && !s.starts_with(MARCADOR_END_VAR) {
            b.header_lines.push(s.to_string());
        }
    }
    if let Some(b) = actual {
        bloques.push(b);
    }
    Ok(bloques)
}

fn procesar_bloque_variable(
    bloque: BloqueVariableBruto,
) -> Result<(MetadatosVariable, Vec<i64>, Vec<String>)> {
    let mut factor = 0.0;
    let mut tipo_str = String::new();

    for hl in &bloque.header_lines {
        if let Some((k, v)) = hl.split_once(SEPARADOR_SIGNO_VALOR) {
            if k.trim() == "WRFactor" {
                factor = v.trim().parse().unwrap_or(0.0);
            } else if k.trim() == "VarType" {
                tipo_str = v.trim().to_string();
            }
        }
    }

    let tipo = TipoVariable::de_metadatos(&tipo_str, &bloque.nombre);
    let wr_factor = if factor > 0.0 {
        factor
    } else {
        tipo.wr_factor_defecto()
    };

    let mut raws = Vec::with_capacity(bloque.day_lines.len() * TOKENS_POR_DIA);
    let mut dias = Vec::with_capacity(bloque.day_lines.len());

    for dl in &bloque.day_lines {
        let (dia, vals) = decodificar_linea_dia(dl)?;
        dias.push(dia);
        raws.extend(vals);
    }

    let meta = MetadatosVariable {
        nombre: bloque.nombre,
        tipo,
        wr_factor,
        indice: bloque.indice,
        header_lines: bloque.header_lines,
    };
    Ok((meta, raws, dias))
}

fn construir_horas(datos: &DatosConstruccionHoras<'_>) -> Result<Vec<HoraValores>> {
    let total_horas = datos.dias.len() * TOKENS_POR_DIA;
    let mut horas = Vec::with_capacity(total_horas);

    for h in 0..total_horas {
        let dia_idx = h / TOKENS_POR_DIA;
        let dia = datos.dias.get(dia_idx).cloned().unwrap_or_default();
        let mut valores = HashMap::with_capacity(datos.variables.len());

        for (nombre, meta) in datos.variables.iter().zip(datos.metadatos.iter()) {
            let crudos_var = datos
                .valores_crudos
                .get(nombre)
                .ok_or_else(|| Error::Format(format!("Valores crudos ausentes para '{nombre}'")))?;
            let raw = crudos_var.get(h).copied().unwrap_or(0);
            let val = crudo_a_fisico(raw, meta.wr_factor, meta.tipo);
            valores.insert(nombre.clone(), val);
        }
        horas.push(HoraValores {
            hora: h,
            dia,
            valores,
        });
    }
    Ok(horas)
}
