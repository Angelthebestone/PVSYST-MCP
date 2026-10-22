//! Codec for reading and encoding PVsyst `.MEF` meteo format files.

use super::dominio::{
    DatosVariableMef, MefModerno, MeteoMef, ParametrosMefModerno, PresenciaBom, VariableMef,
};
use super::encoder::{encode_mef_moderno, BOM_UTF8};
use super::legacy::parsear_mef_legacy;
use crate::error::{Error, Result};

const PREFIJO_MODERNO: &str = "PVObject_=pvMeteoFormat";
const CIERRE_MODERNO: &str = "End of PVObject pvMeteoFormat";
const CIERRE_LISTA_VARS: &str = "End of List VarASCIIList";

/// Parses raw bytes of any `.MEF` file into structured [`MeteoMef`].
pub(crate) fn parse_mef(bytes: &[u8]) -> Result<MeteoMef> {
    if bytes.is_empty() {
        return Err(Error::Invalid("Archivo MEF vacío".into()));
    }
    let (bom, contenido_bytes) = if bytes.starts_with(BOM_UTF8) {
        (PresenciaBom::Presente, &bytes[BOM_UTF8.len()..])
    } else {
        (PresenciaBom::Ausente, bytes)
    };

    let texto_inicio = String::from_utf8_lossy(&contenido_bytes[..contenido_bytes.len().min(64)]);
    if texto_inicio.trim_start().starts_with(PREFIJO_MODERNO) {
        let moderno = parsear_mef_moderno(contenido_bytes, bom)?;
        Ok(MeteoMef::Moderno(moderno))
    } else {
        let legacy = parsear_mef_legacy(bytes)?;
        Ok(MeteoMef::Legacy(legacy))
    }
}

/// Encodes a [`MeteoMef`] into bytes. Legacy format is strictly read-only.
pub(crate) fn encode_mef(mef: &MeteoMef) -> Result<Vec<u8>> {
    match mef {
        MeteoMef::Moderno(m) => encode_mef_moderno(m),
        MeteoMef::Legacy(_) => Err(Error::Invalid("no hay escritor legacy demostrado".into())),
    }
}

fn parsear_mef_moderno(bytes: &[u8], bom: PresenciaBom) -> Result<MefModerno> {
    let texto = std::str::from_utf8(bytes)
        .map_err(|e| Error::Invalid(format!("MEF moderno contiene UTF-8 inválido: {e}")))?;
    let lineas: Vec<String> = texto
        .lines()
        .map(|s| s.trim_end_matches('\r').to_string())
        .collect();

    let lineas_slice: Vec<&str> = lineas.iter().map(String::as_str).collect();
    let params = recolectar_parametros_moderno(&lineas_slice)?;
    MefModerno::nuevo_con_crudos(params, Some(lineas), bom)
}

#[derive(Default)]
struct RecolectorMef {
    comentario: Option<String>,
    version: Option<String>,
    formato_fecha: Option<String>,
    separador: Option<String>,
    paso_medicion_min: Option<u32>,
    tiempo_referencia: Option<String>,
    albedo: Option<f64>,
    columnas_fecha: Option<Vec<usize>>,
    conteo_vars: Option<usize>,
    variables: Vec<VariableMef>,
    lista_cerrada: bool,
    objeto_cerrado: bool,
}

fn recolectar_parametros_moderno(lineas: &[&str]) -> Result<ParametrosMefModerno> {
    if lineas.is_empty() || lineas[0].trim() != PREFIJO_MODERNO {
        return Err(Error::Invalid(
            "MEF moderno debe iniciar con 'PVObject_=pvMeteoFormat'".into(),
        ));
    }
    let mut rec = RecolectorMef::default();
    let mut i = 1;
    while i < lineas.len() {
        let l = lineas[i].trim();
        if l.starts_with("VarASCIIList, list of=") {
            rec.conteo_vars = parsear_conteo_lista_vars(l);
        } else if l == CIERRE_LISTA_VARS {
            rec.lista_cerrada = true;
        } else if l == CIERRE_MODERNO || l == "End of PVObject" {
            rec.objeto_cerrado = true;
        } else if l.starts_with("VarASCII=") {
            let (var, prox_idx) = parsear_bloque_var_ascii(lineas, i)?;
            rec.variables.push(var);
            i = prox_idx;
            continue;
        } else {
            asignar_campo_recolector(&mut rec, l)?;
        }
        i += 1;
    }
    validar_recolector_completo(rec)
}

fn parsear_conteo_lista_vars(linea: &str) -> Option<usize> {
    let parte = linea.strip_prefix("VarASCIIList, list of=")?;
    parte.split_whitespace().next().and_then(|s| s.parse().ok())
}

fn asignar_campo_recolector(r: &mut RecolectorMef, l: &str) -> Result<()> {
    if l.is_empty() || l.starts_with("LargChamps") || l.starts_with("NChampsAff=") {
        return Ok(());
    }
    if let Some(val) = l.strip_prefix("Comment=").map(str::trim) {
        r.comentario = Some(val.to_string());
    } else if let Some(val) = l.strip_prefix("Version=").map(str::trim) {
        r.version = Some(val.to_string());
    } else if let Some(val) = l.strip_prefix("DateFmtType=").map(str::trim) {
        r.formato_fecha = Some(val.to_string());
    } else if let Some(val) = l.strip_prefix("Separ=").map(str::trim) {
        r.separador = Some(val.to_string());
    } else if let Some(val) = l.strip_prefix("MeasureStep=").map(str::trim) {
        let paso = val
            .parse::<u32>()
            .map_err(|_| Error::Invalid(format!("MeasureStep inválido: '{val}'")))?;
        r.paso_medicion_min = Some(paso);
    } else if let Some(val) = l.strip_prefix("RefTimeS=").map(str::trim) {
        r.tiempo_referencia = Some(val.to_string());
    } else if let Some(val) = l.strip_prefix("AlbedoSolMF=").map(str::trim) {
        let alb = val
            .parse::<f64>()
            .map_err(|_| Error::Invalid(format!("AlbedoSolMF inválido: '{val}'")))?;
        r.albedo = Some(alb);
    } else if let Some(val) = l.strip_prefix("NoFieldDate=").map(str::trim) {
        r.columnas_fecha = Some(parsear_columnas_fecha(val)?);
    } else if !l.contains('=') {
        return Err(Error::Invalid(format!(
            "Línea MEF corrupta o no reconocida: '{l}'"
        )));
    }
    Ok(())
}

fn validar_recolector_completo(r: RecolectorMef) -> Result<ParametrosMefModerno> {
    if !r.objeto_cerrado {
        return Err(Error::Invalid(
            "Falta cierre 'End of PVObject pvMeteoFormat'".into(),
        ));
    }
    if let Some(conteo) = r.conteo_vars {
        if !r.lista_cerrada {
            return Err(Error::Invalid(
                "Falta cierre 'End of List VarASCIIList'".into(),
            ));
        }
        if conteo != r.variables.len() {
            return Err(Error::Invalid(format!(
                "Discrepancia en conteo de variables: esperado {conteo}, leídas {}",
                r.variables.len()
            )));
        }
    }
    if r.variables.is_empty() {
        return Err(Error::Invalid(
            "MEF moderno debe definir al menos una variable meteorológica".into(),
        ));
    }
    Ok(ParametrosMefModerno {
        comentario: r
            .comentario
            .ok_or_else(|| Error::Invalid("Falta Comment en MEF moderno".into()))?,
        version: r
            .version
            .ok_or_else(|| Error::Invalid("Falta Version en MEF moderno".into()))?,
        formato_fecha: r
            .formato_fecha
            .ok_or_else(|| Error::Invalid("Falta DateFmtType en MEF moderno".into()))?,
        separador: r
            .separador
            .ok_or_else(|| Error::Invalid("Falta Separ en MEF moderno".into()))?,
        paso_medicion_min: r
            .paso_medicion_min
            .ok_or_else(|| Error::Invalid("Falta MeasureStep en MEF moderno".into()))?,
        tiempo_referencia: r.tiempo_referencia.unwrap_or_else(|| "LegalTime".into()),
        albedo: r
            .albedo
            .ok_or_else(|| Error::Invalid("Falta AlbedoSolMF en MEF moderno".into()))?,
        columnas_fecha: r
            .columnas_fecha
            .ok_or_else(|| Error::Invalid("Falta NoFieldDate en MEF moderno".into()))?,
        variables: r.variables,
    })
}

fn parsear_columnas_fecha(val: &str) -> Result<Vec<usize>> {
    let mut cols = Vec::new();
    for s in val.split(',') {
        let s_trim = s.trim();
        if s_trim.is_empty() {
            continue;
        }
        let c = s_trim
            .parse::<usize>()
            .map_err(|_| Error::Invalid(format!("Valor de NoFieldDate inválido: '{s_trim}'")))?;
        if c > 0 {
            if cols.contains(&c) {
                return Err(Error::Invalid(format!(
                    "Columna duplicada en NoFieldDate: {c}"
                )));
            }
            cols.push(c);
        }
    }
    if cols.is_empty() {
        return Err(Error::Invalid(
            "NoFieldDate debe contener al menos una columna de fecha".into(),
        ));
    }
    Ok(cols)
}

fn parsear_bloque_var_ascii(lineas: &[&str], inicio: usize) -> Result<(VariableMef, usize)> {
    let mut d = DatosVariableMef {
        ident: String::new(),
        tipo: String::new(),
        columna: 0,
        multiplicador: 1.0,
        unidad: String::new(),
    };
    let mut cerrado = false;
    let mut curr = inicio + 1;
    while curr < lineas.len() {
        let l = lineas[curr].trim();
        if l == "End of TVarASCII" {
            cerrado = true;
            curr += 1;
            break;
        }
        asignar_campo_var_ascii(l, &mut d)?;
        curr += 1;
    }
    if !cerrado {
        return Err(Error::Invalid(
            "Bloque VarASCII no cerrado con 'End of TVarASCII'".into(),
        ));
    }
    if d.ident.is_empty() || d.tipo.is_empty() || d.unidad.is_empty() {
        return Err(Error::Invalid(
            "Bloque VarASCII con campos requeridos vacíos".into(),
        ));
    }
    Ok((VariableMef::nueva(d)?, curr))
}

fn asignar_campo_var_ascii(l: &str, d: &mut DatosVariableMef) -> Result<()> {
    if let Some(val) = l.strip_prefix("VarIdent=").map(str::trim) {
        d.ident = val.to_string();
    } else if let Some(val) = l.strip_prefix("VarType=").map(str::trim) {
        d.tipo = val.to_string();
    } else if let Some(val) = l.strip_prefix("NoField=").map(str::trim) {
        d.columna = val
            .parse::<usize>()
            .map_err(|_| Error::Invalid(format!("NoField inválido: '{val}'")))?;
    } else if let Some(val) = l.strip_prefix("FieldMult=").map(str::trim) {
        d.multiplicador = val
            .parse::<f64>()
            .map_err(|_| Error::Invalid(format!("FieldMult inválido: '{val}'")))?;
    } else if let Some(val) = l.strip_prefix("UnitId=").map(str::trim) {
        d.unidad = val.to_string();
    } else if !l.is_empty() && !l.contains('=') {
        return Err(Error::Invalid(format!(
            "Línea corrupta dentro de VarASCII: '{l}'"
        )));
    }
    Ok(())
}
