//! Application orchestration for reading and writing PVsyst `.MEF` meteo format files.

use super::codec::parse_mef;
use super::dominio::{DatosVariableMef, MefModerno, MeteoMef, ParametrosMefModerno, VariableMef};
use crate::args::{req_str, workspace_of};
use crate::config::{data_ro, Paths};
use crate::error::Result;
use crate::workspace::{
    comprobar_coincidencia_exacta, leer_bytes_acotados, resolver_archivo_con_fallback,
    PeticionResolucion, MAX_BYTES_MAXIMO,
};
use serde_json::{json, Value};

/// Reads and decodes a `.MEF` weather specification file.
pub(crate) fn leer_mef(paths: &Paths, args: &Value) -> Result<Value> {
    let nombre = req_str(args, &["archivo", "mef"], "p.ej. 'PVsyst_ANE-NEDO.MEF'")?;
    let ws = workspace_of(paths, args);
    let peticion = PeticionResolucion {
        nombre,
        subcarpeta_ws: Some("Meteo"),
        rutas_dataro: vec![
            data_ro().join("PVsyst8.1_Data").join("Meteo").join(nombre),
            data_ro()
                .join("PVsyst8.1_Data")
                .join("CLI")
                .join("Ressources")
                .join("Examples")
                .join("Sources")
                .join("WeatherData")
                .join(nombre),
        ],
    };
    let ruta = resolver_archivo_con_fallback(&ws, args, &peticion)?;
    let bytes = leer_bytes_acotados(&ruta, MAX_BYTES_MAXIMO)?;
    let mef = parse_mef(&bytes)?;

    let variables_json = variables_a_json(&mef);

    let tipo_formato = match &mef {
        MeteoMef::Moderno(_) => "pvMeteoFormat-moderno",
        MeteoMef::Legacy(_) => "legacy",
    };

    let mut res = json!({
        "archivo": ruta.file_name().map_or("formato.mef".into(), |n| n.to_string_lossy().to_string()),
        "tipo_formato": tipo_formato,
        "comentario": mef.comentario(),
        "version": mef.version(),
        "total_variables": mef.variables().len(),
        "variables": variables_json
    });
    if let Some(calcul) = mef.calcul() {
        res["calcul"] = json!(calcul);
    }
    Ok(res)
}
fn variables_a_json(mef: &MeteoMef) -> Vec<Value> {
    mef.variables()
        .iter()
        .map(|v| {
            json!({
                "ident": v.ident(),
                "tipo": v.tipo(),
                "columna": v.columna(),
                "multiplicador": v.multiplicador(),
                "unidad": v.unidad()
            })
        })
        .collect()
}

/// Generates standard byte content for PVGIS ANE-NEDO format MEF specification.
pub(crate) fn generar_mef_pvgis_nedo() -> Result<Vec<u8>> {
    let variables = vec![
        VariableMef::nueva(DatosVariableMef {
            ident: "GlobHor".into(),
            tipo: "Irrad".into(),
            columna: 6,
            multiplicador: 1.0,
            unidad: "W_m2".into(),
        })?,
        VariableMef::nueva(DatosVariableMef {
            ident: "DiffHor".into(),
            tipo: "Irrad".into(),
            columna: 7,
            multiplicador: 1.0,
            unidad: "W_m2".into(),
        })?,
        VariableMef::nueva(DatosVariableMef {
            ident: "T_Amb".into(),
            tipo: "Temper".into(),
            columna: 8,
            multiplicador: 1.0,
            unidad: "DegC".into(),
        })?,
        VariableMef::nueva(DatosVariableMef {
            ident: "WindVel".into(),
            tipo: "WindVel".into(),
            columna: 9,
            multiplicador: 1.0,
            unidad: "m_s".into(),
        })?,
    ];
    let cols_fecha = vec![2, 3, 4, 5];
    let params = ParametrosMefModerno {
        comentario: "ANE-NEDO known format".into(),
        version: "8.1.2".into(),
        formato_fecha: "MM_DD_hh_mm".into(),
        separador: "$003B".into(),
        paso_medicion_min: 60,
        tiempo_referencia: "LegalTime".into(),
        albedo: 0.20,
        columnas_fecha: cols_fecha,
        variables,
    };
    let moderno = MefModerno::nuevo(params)?;
    super::encoder::encode_mef_moderno(&moderno)
}

/// Validates that raw bytes represent a valid `.MEF` specification and that
/// re-serializing them reproduces 100% identical bytes. Legacy MEF is rejected.
pub(crate) fn validar_mef(bytes: &[u8]) -> Result<()> {
    let mef = parse_mef(bytes)?;
    let reencoded = super::codec::encode_mef(&mef)?;
    comprobar_coincidencia_exacta(bytes, &reencoded, "MEF")
}
