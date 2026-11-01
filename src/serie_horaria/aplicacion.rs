//! Application orchestration for reading and replacing hourly series in .MET and .DAM files.

use super::codificar::codificar;
use super::decodificar::decodificar;
use super::dominio::{
    fisico_a_crudo, validar_serie, MetadatosVariable, SerieHoraria, TipoVariable,
};
use crate::args::{req_str, workspace_of};
use crate::config::{data_ro, Paths};
use crate::error::{Error, Result};
use crate::format::object;
use crate::history;
use crate::workspace::{
    leer_bytes_acotados, primer_existente_en_bases, primer_existente_en_workspace,
};
use serde_json::{json, Value};
use std::collections::HashMap;
use std::path::PathBuf;

const MAX_BYTES_SERIE: usize = 64 * 1024 * 1024;
const LIMITE_HORAS_DEFECTO: usize = 24;
/// Extensions tried after the bare name: series live in `.MET` and `.DAM` files.
const EXTENSIONES: [&str; 3] = ["", ".MET", ".DAM"];

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ModoResolucion {
    SoloWorkspace,
    IncluirLecturaRo,
}

/// Reads hourly series from .MET or .DAM file with offset and limit pagination.
pub(crate) fn leer_serie(paths: &Paths, args: &Value) -> Result<Value> {
    let path = resolver_archivo(paths, args, ModoResolucion::IncluirLecturaRo)?;
    let bytes = leer_bytes_acotados(&path, MAX_BYTES_SERIE)?;
    let text = String::from_utf8_lossy(&bytes);
    let tipo_obj = object::parse_first(&text).map_or_else(|| "pvMeteo".to_string(), |r| r.obj_type);

    let serie = decodificar(&text)?;
    let total_horas = serie.horas.len();
    let offset = entero_paginacion(args, "offset", 0)?;
    let limite = entero_paginacion(args, "limite", LIMITE_HORAS_DEFECTO)?;

    let horas_paginadas: Vec<_> = serie
        .horas
        .iter()
        .skip(offset)
        .take(limite)
        .map(|h| json!({ "hora": h.hora, "dia": h.dia, "valores": h.valores }))
        .collect();

    let unidades: HashMap<_, _> = serie
        .metadatos
        .iter()
        .map(|m| (m.nombre.clone(), m.tipo.unidad()))
        .collect();

    Ok(json!({
        "archivo": path.to_string_lossy(),
        "tipo": tipo_obj,
        "total_horas": total_horas,
        "offset": offset,
        "limite": limite,
        "variables": serie.variables,
        "unidades": unidades,
        "horas": horas_paginadas,
    }))
}

/// Replaces hourly series data in an existing .MET or .DAM file, preserving header bytes with history.
pub(crate) fn escribir_serie(paths: &Paths, args: &Value) -> Result<Value> {
    let workspace = workspace_of(paths, args);
    let path = resolver_archivo(paths, args, ModoResolucion::SoloWorkspace)?;
    let bytes = leer_bytes_acotados(&path, MAX_BYTES_SERIE)?;
    let original = String::from_utf8_lossy(&bytes);

    let variables = extraer_nombres_variables(args)?;
    let valores_por_var = extraer_valores(args, &variables)?;
    validar_serie(&variables, &valores_por_var)?;

    let serie_original = decodificar(&original)?;
    let nueva_serie = actualizar_serie(serie_original, &variables, &valores_por_var)?;
    let nuevo_cuerpo = codificar(&nueva_serie)?;
    let nuevo_contenido = ensamblar_archivo(&original, &nuevo_cuerpo)?;

    let historial_id =
        history::snapshot(&workspace, std::slice::from_ref(&path), "escribir_serie")?;
    history::escribir_atomico(&path, nuevo_contenido.as_bytes())?;

    Ok(json!({
        "archivo": path.to_string_lossy(),
        "variables_actualizadas": variables,
        "total_horas": nueva_serie.horas.len(),
        "historial_id": historial_id,
        "bytes_escritos": nuevo_contenido.len(),
    }))
}

fn resolver_archivo(paths: &Paths, args: &Value, modo: ModoResolucion) -> Result<PathBuf> {
    let nombre = req_str(args, &["archivo"], "")?;
    let ws = workspace_of(paths, args);
    let alcance = crate::workspace::Alcance::de_args(args);
    let mut relativos = vec![nombre.to_string()];
    for sub in ["Meteo", "DataMeas"] {
        relativos.extend(EXTENSIONES.iter().map(|ext| format!("{sub}/{nombre}{ext}")));
    }
    if let Some(p) = primer_existente_en_workspace(&ws, &relativos, alcance)? {
        return Ok(p);
    }
    let en_ro = match modo {
        ModoResolucion::IncluirLecturaRo => {
            let ro = data_ro().join("PVsyst8.1_Data");
            let bases = [ro.join("Meteo"), ro.join("DataMeas")];
            let nombres: Vec<String> = EXTENSIONES.iter().map(|e| format!("{nombre}{e}")).collect();
            primer_existente_en_bases(&bases, &nombres)
        }
        ModoResolucion::SoloWorkspace => None,
    };
    en_ro.ok_or_else(|| Error::NotFound(format!("Archivo de serie '{nombre}' no encontrado")))
}

/// A paging argument: absent or null takes `defecto`; anything but a non-negative
/// integer is refused instead of silently replaced by the default.
pub(super) fn entero_paginacion(args: &Value, clave: &str, defecto: usize) -> Result<usize> {
    let Some(valor) = args.get(clave).filter(|v| !v.is_null()) else {
        return Ok(defecto);
    };
    valor
        .as_u64()
        .and_then(|n| usize::try_from(n).ok())
        .ok_or_else(|| {
            Error::Invalid(format!(
                "'{clave}' debe ser un entero no negativo (recibido {valor})"
            ))
        })
}

pub(super) fn extraer_nombres_variables(args: &Value) -> Result<Vec<String>> {
    let arr = args
        .get("variables")
        .and_then(Value::as_array)
        .ok_or_else(|| {
            Error::Invalid("Argumento 'variables' debe ser un arreglo de cadenas".into())
        })?;
    let nombres: Vec<String> = arr
        .iter()
        .map(|v| {
            v.as_str()
                .map(ToString::to_string)
                .ok_or_else(|| Error::Invalid("Elemento de 'variables' no es cadena".into()))
        })
        .collect::<Result<_>>()?;
    // A repeated name would have its second column silently overwrite the first. O(n²), n small.
    if let Some((i, n)) = nombres
        .iter()
        .enumerate()
        .find(|(i, n)| nombres[..*i].contains(n))
    {
        return Err(Error::Invalid(format!(
            "La variable '{n}' aparece repetida en 'variables' (posición {})",
            i + 1
        )));
    }
    Ok(nombres)
}

fn extraer_valores(args: &Value, variables: &[String]) -> Result<Vec<Vec<f64>>> {
    let val = args.get("valores").ok_or_else(|| Error::MissingArg {
        key: "valores".into(),
        hint: "matriz o mapa de valores horarios".into(),
    })?;
    if let Some(obj) = val.as_object() {
        return variables
            .iter()
            .map(|var| {
                let v = obj.get(var).ok_or_else(|| {
                    Error::Invalid(format!("Variable '{var}' ausente en 'valores'"))
                })?;
                extraer_vector_f64(v, var)
            })
            .collect();
    }
    let arr = val
        .as_array()
        .ok_or_else(|| Error::Invalid("Argumento 'valores' debe ser arreglo o mapa".into()))?;
    if arr.len() == variables.len() && arr.first().is_some_and(Value::is_array) {
        return arr
            .iter()
            .zip(variables.iter())
            .map(|(col, nom)| extraer_vector_f64(col, nom))
            .collect();
    }
    transponer_filas(arr, variables.len())
}

fn extraer_vector_f64(v: &Value, nom: &str) -> Result<Vec<f64>> {
    let arr = v
        .as_array()
        .ok_or_else(|| Error::Invalid(format!("Valores para '{nom}' deben ser un arreglo")))?;
    arr.iter()
        .map(|elem| {
            elem.as_f64()
                .ok_or_else(|| Error::Invalid(format!("Valor no numerico en serie '{nom}'")))
        })
        .collect()
}

fn transponer_filas(filas: &[Value], num_vars: usize) -> Result<Vec<Vec<f64>>> {
    let mut cols: Vec<Vec<f64>> = (0..num_vars)
        .map(|_| Vec::with_capacity(filas.len()))
        .collect();
    for (h, f) in filas.iter().enumerate() {
        let fila = f
            .as_array()
            .ok_or_else(|| Error::Invalid(format!("Fila {h} de 'valores' no es un arreglo")))?;
        if fila.len() != num_vars {
            return Err(Error::Invalid(format!(
                "Fila {h} tiene {} elementos; esperado {num_vars}",
                fila.len()
            )));
        }
        for (i, val) in fila.iter().enumerate() {
            let num = val.as_f64().ok_or_else(|| {
                Error::Invalid(format!("Valor no numerico en fila {h}, columna {i}"))
            })?;
            cols[i].push(num);
        }
    }
    Ok(cols)
}

pub(crate) fn actualizar_serie(
    mut serie: SerieHoraria,
    variables: &[String],
    valores_por_var: &[Vec<f64>],
) -> Result<SerieHoraria> {
    for (nombre, vals) in variables.iter().zip(valores_por_var.iter()) {
        let (factor, tipo) = factor_y_tipo(&serie, nombre)?;
        let crudos: Vec<i64> = vals
            .iter()
            .map(|&v| fisico_a_crudo(v, factor, tipo))
            .collect();
        serie.valores_crudos.insert(nombre.clone(), crudos);
        if !serie.variables.contains(nombre) {
            serie.variables.push(nombre.clone());
            serie.metadatos.push(MetadatosVariable {
                nombre: nombre.clone(),
                tipo,
                wr_factor: factor,
                indice: serie.metadatos.len() + 1,
                header_lines: Vec::new(),
            });
        }
    }
    Ok(serie)
}

/// Scale and type of a variable: from the file when it already has the column,
/// otherwise from the PVsyst catalog. Names outside both are rejected, never added.
fn factor_y_tipo(serie: &SerieHoraria, nombre: &str) -> Result<(f64, TipoVariable)> {
    if let Some(m) = serie.metadatos.iter().find(|m| m.nombre == nombre) {
        return Ok((m.wr_factor, m.tipo));
    }
    let tipo = TipoVariable::de_metadatos("", nombre);
    if tipo == TipoVariable::Otro {
        return Err(Error::Invalid(format!(
            "Variable desconocida '{nombre}': no existe en el archivo ni en el catálogo PVsyst"
        )));
    }
    Ok((tipo.wr_factor_defecto(), tipo))
}

fn ensamblar_archivo(original: &str, nuevo_cuerpo: &str) -> Result<String> {
    let pos_start = original
        .find("VarMeteo, list of=")
        .ok_or_else(|| Error::Format("No se encontro seccion 'VarMeteo' en original".into()))?;
    let line_start = original[..pos_start].rfind('\n').map_or(0, |p| p + 1);

    let end_marker = "End of the List VarMeteo";
    let pos_end = original
        .find(end_marker)
        .ok_or_else(|| Error::Format("No se encontro 'End of the List VarMeteo'".into()))?;
    let line_end = original[pos_end..]
        .find('\n')
        .map_or(original.len(), |p| pos_end + p + 1);

    let header = &original[..line_start];
    let footer = &original[line_end..];
    Ok(format!("{header}{nuevo_cuerpo}{footer}"))
}
