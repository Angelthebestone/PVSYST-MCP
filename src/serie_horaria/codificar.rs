//! Serialization of `SerieHoraria` into PVsyst `VarMeteo` text.

use super::base62::{codificar_linea_dia, TOKENS_POR_DIA};
use super::dominio::{fisico_a_crudo, MetadatosVariable, SerieHoraria, TipoVariable};
use crate::error::{Error, Result};

const SEPARADOR_SIGNO_VALOR: char = '=';
const PREFIJO_VAR_SIMUL: &str = "VarSimul_";
const MARCADOR_END_DATA: &str = "End of Data";
const MARCADOR_END_VAR: &str = "End of VarSimul";
const MARCADOR_END_LIST: &str = "End of the List VarMeteo";

struct EntradaCodificacion<'a> {
    serie: &'a SerieHoraria,
    idx: usize,
    meta: &'a MetadatosVariable,
}

struct EntradaDias<'a> {
    dias: &'a [String],
    raws: &'a [i64],
    nombre: &'a str,
}

/// Encodes `SerieHoraria` back to byte-identical `VarMeteo` text.
pub(crate) fn codificar(serie: &SerieHoraria) -> Result<String> {
    let mut out = String::with_capacity(serie.variables.len() * 32768);
    out.push_str(&serie.top_line);
    out.push_str("\r\n");

    for (idx, meta) in serie.metadatos.iter().enumerate() {
        let entrada = EntradaCodificacion { serie, idx, meta };
        codificar_bloque_variable(&entrada, &mut out)?;
    }
    out.push_str("  ");
    out.push_str(MARCADOR_END_LIST);
    out.push_str("\r\n");
    Ok(out)
}

fn codificar_bloque_variable(entrada: &EntradaCodificacion<'_>, out: &mut String) -> Result<()> {
    escribir_encabezado_variable(entrada.idx, entrada.meta, out);
    let raws = obtener_raws_variable(entrada.serie, entrada.meta);
    let datos_dias = EntradaDias {
        dias: &entrada.serie.dias,
        raws: &raws,
        nombre: &entrada.meta.nombre,
    };
    escribir_dias_variable(&datos_dias, out)?;
    escribir_pie_variable(&entrada.meta.nombre, out);
    Ok(())
}

fn escribir_encabezado_variable(idx: usize, meta: &MetadatosVariable, out: &mut String) {
    out.push_str("\r\n    ");
    out.push_str(PREFIJO_VAR_SIMUL);
    let var_num = if meta.indice > 0 {
        meta.indice
    } else {
        idx + 1
    };
    out.push_str(&var_num.to_string());
    out.push(SEPARADOR_SIGNO_VALOR);
    out.push_str(&meta.nombre);
    out.push_str("\r\n");

    if meta.header_lines.is_empty() {
        escribir_encabezado_defecto(meta, out);
        return;
    }
    for hl in &meta.header_lines {
        out.push_str("      ");
        out.push_str(hl);
        out.push_str("\r\n");
    }
}

fn escribir_encabezado_defecto(meta: &MetadatosVariable, out: &mut String) {
    out.push_str("      Varident=");
    out.push_str(&meta.nombre);
    out.push_str("\r\n      VarAcc=Hour\r\n      VarType=");
    out.push_str(match meta.tipo {
        TipoVariable::Irradiacion => "Irrad",
        TipoVariable::Temperatura => "Temper",
        TipoVariable::Viento => "WindVel",
        TipoVariable::Factor => "Factor",
        TipoVariable::Energia => "Energy",
        TipoVariable::Tension => "Voltage",
        TipoVariable::Corriente => "Current",
        TipoVariable::Otro => "Other",
    });
    out.push_str("\r\n      VarStore=Meas\r\n      WRFactor=");
    out.push_str(&format!("{:.3}", meta.wr_factor));
    out.push_str("\r\n");
}

fn obtener_raws_variable(serie: &SerieHoraria, meta: &MetadatosVariable) -> Vec<i64> {
    if let Some(r) = serie.valores_crudos.get(&meta.nombre) {
        return r.clone();
    }
    serie
        .horas
        .iter()
        .map(|h| {
            let fisico = h.valores.get(&meta.nombre).copied().unwrap_or(0.0);
            fisico_a_crudo(fisico, meta.wr_factor, meta.tipo)
        })
        .collect()
}

fn escribir_dias_variable(datos: &EntradaDias<'_>, out: &mut String) -> Result<()> {
    for (d_idx, dia) in datos.dias.iter().enumerate() {
        let inicio = d_idx * TOKENS_POR_DIA;
        let fin = inicio + TOKENS_POR_DIA;
        let day_slice = datos.raws.get(inicio..fin).ok_or_else(|| {
            Error::Format(format!(
                "Rango horario [{inicio}..{fin}] fuera de limites para '{}'",
                datos.nombre
            ))
        })?;
        out.push_str(&codificar_linea_dia(dia, day_slice));
    }
    Ok(())
}

fn escribir_pie_variable(nombre: &str, out: &mut String) {
    out.push_str("      ");
    out.push_str(MARCADOR_END_DATA);
    out.push_str("\r\n    ");
    out.push_str(MARCADOR_END_VAR);
    out.push_str("=vi");
    out.push_str(nombre);
    out.push_str("\r\n");
}
