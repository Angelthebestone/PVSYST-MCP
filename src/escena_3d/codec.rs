//! Codec for parsing PVsyst 3D shading scenes and objects (`.SHD`, `.BLD`, `.SHO`).

use super::binario::{
    parsear_bloque_geometria, parsear_bloque_posicion, LONGITUD_BLOQUE_GEOMETRIA_ELEMENTAL,
    LONGITUD_BLOQUE_POSICION, TERMINADOR_SLASH,
};
use super::dominio::{
    DatosEscena3d, DatosObjeto3d, Escena3d, GeometriaElemental, ObjetoEscena3d, Posicion3d,
    TipoArchivoEscena,
};
use crate::error::{Error, Result};
use crate::format::decodificar_texto_flexible;

const MAX_LONGITUD_LINEA_CONTEO: usize = 32;

/// Parses raw bytes of a `.SHD`, `.BLD`, or `.SHO` file into structured [`Escena3d`].
pub(crate) fn parse_escena_3d(bytes: &[u8], tipo: TipoArchivoEscena) -> Result<Escena3d> {
    if bytes.is_empty() {
        return Err(Error::Invalid("Archivo 3D vacío".into()));
    }
    let lineas = dividir_por_crlf(bytes);
    if lineas.len() < 2 {
        return Err(Error::Invalid("Cabecera 3D insuficiente".into()));
    }

    let (clase_raiz, descripcion) = parsear_linea_ident(lineas[0]);
    let clase_esperada = match tipo {
        TipoArchivoEscena::Shd => "pvShading",
        TipoArchivoEscena::Bld => "pvShdBuilding",
        TipoArchivoEscena::Sho => "pvShdObjects",
    };
    if clase_raiz != clase_esperada {
        return Err(Error::Invalid(format!(
            "Incoherencia entre extensión '{:?}' y clase raíz '{clase_raiz}' (se esperaba {clase_esperada})",
            tipo
        )));
    }
    let (version, calcul) = parsear_linea_version_raiz(lineas[1])?;
    let objetos = parsear_objetos_escena(&lineas)?;
    Escena3d::nueva(DatosEscena3d {
        tipo,
        clase_raiz,
        descripcion,
        version,
        calcul,
        objetos,
    })
}

fn parsear_objetos_escena(lineas: &[&[u8]]) -> Result<Vec<ObjetoEscena3d>> {
    let mut idx = 2;
    while idx < lineas.len() && !es_inicio_objeto(lineas[idx]) {
        idx += 1;
    }

    let mut objetos = Vec::new();
    while idx < lineas.len() {
        if es_inicio_objeto(lineas[idx]) {
            let (obj, prox_idx) = parsear_objeto_3d(lineas, idx)?;
            objetos.push(obj);
            idx = prox_idx;
        } else {
            idx += 1;
        }
    }
    Ok(objetos)
}

fn dividir_por_crlf(bytes: &[u8]) -> Vec<&[u8]> {
    let mut lineas = Vec::new();
    let mut inicio = 0;
    let mut i = 0;
    while i + 1 < bytes.len() {
        if bytes[i] == b'\r' && bytes[i + 1] == b'\n' {
            lineas.push(&bytes[inicio..i]);
            inicio = i + 2;
            i += 2;
        } else {
            i += 1;
        }
    }
    if inicio < bytes.len() {
        lineas.push(&bytes[inicio..]);
    }
    lineas
}

fn parsear_linea_ident(linea: &[u8]) -> (String, String) {
    let texto = decodificar_texto_flexible(linea);
    let mut partes = texto.split(';');
    let clase = partes.next().map_or("", str::trim).to_string();
    let desc = partes.next().map_or("", str::trim).to_string();
    (clase, desc)
}

fn parsear_linea_version_raiz(linea: &[u8]) -> Result<(String, String)> {
    let texto = decodificar_texto_flexible(linea);
    let mut version = None;
    let mut calcul = None;
    for seg in texto.split(';') {
        let seg_trim = seg.trim();
        if let Some(v) = seg_trim.strip_prefix("Version ") {
            version = Some(v.trim().to_string());
        } else if let Some(c) = seg_trim.strip_prefix("Calcul=") {
            calcul = Some(c.trim().to_string());
        }
    }
    let (Some(v), Some(c)) = (version, calcul) else {
        return Err(Error::Invalid(format!(
            "Línea de versión de escena inválida (debe contener 'Version ' y 'Calcul='): '{texto}'"
        )));
    };
    Ok((v, c))
}

fn parsear_linea_version(linea: &[u8]) -> (String, String) {
    let texto = decodificar_texto_flexible(linea);
    let mut version = "4.0".to_string();
    let mut calcul = "$0000".to_string();
    for seg in texto.split(';') {
        let seg_trim = seg.trim();
        if let Some(v) = seg_trim.strip_prefix("Version ") {
            version = v.trim().to_string();
        } else if let Some(c) = seg_trim.strip_prefix("Calcul=") {
            calcul = c.trim().to_string();
        }
    }
    (version, calcul)
}

fn es_inicio_objeto(linea: &[u8]) -> bool {
    linea.starts_with(b"pvShd")
}

fn parsear_objeto_3d(lineas: &[&[u8]], inicio: usize) -> Result<(ObjetoEscena3d, usize)> {
    let (clase, nombre) = parsear_linea_ident(lineas[inicio]);
    let mut version = "4.0".to_string();
    let mut calcul = "$0000".to_string();
    let mut curr = inicio + 1;

    if curr < lineas.len() && lineas[curr].starts_with(b"Version ") {
        let (v, c) = parsear_linea_version(lineas[curr]);
        version = v;
        calcul = c;
        curr += 1;
    }

    let mut cuenta_linea = None;
    if curr < lineas.len() && es_linea_conteo(lineas[curr]) {
        cuenta_linea = Some(decodificar_texto_flexible(lineas[curr]));
        curr += 1;
    }

    let (posicion, geometria, prox_idx) = extraer_bloques_objeto(lineas, curr);
    let obj = ObjetoEscena3d::nuevo(DatosObjeto3d {
        clase,
        nombre,
        version,
        calcul,
        cuenta_linea,
        posicion,
        geometria,
    })?;
    Ok((obj, prox_idx))
}

fn extraer_bloques_objeto(
    lineas: &[&[u8]],
    mut curr: usize,
) -> (Option<Posicion3d>, Option<GeometriaElemental>, usize) {
    let mut posicion = None;
    let mut geometria = None;
    while curr < lineas.len() {
        if es_inicio_objeto(lineas[curr]) {
            break;
        }
        let raw = lineas[curr];
        if raw.len() == LONGITUD_BLOQUE_POSICION
            && raw.last() == Some(&TERMINADOR_SLASH)
            && posicion.is_none()
        {
            posicion = parsear_bloque_posicion(raw);
        } else if raw.len() == LONGITUD_BLOQUE_GEOMETRIA_ELEMENTAL
            && raw.last() == Some(&TERMINADOR_SLASH)
            && geometria.is_none()
        {
            geometria = parsear_bloque_geometria(raw);
        }
        curr += 1;
    }
    (posicion, geometria, curr)
}

fn es_linea_conteo(linea: &[u8]) -> bool {
    if linea.len() > MAX_LONGITUD_LINEA_CONTEO
        || !linea.ends_with(b"/")
        || !linea.contains(&b';')
        || linea.starts_with(b"pv")
    {
        return false;
    }
    let Ok(s) = std::str::from_utf8(linea) else {
        return false;
    };
    let s_trim = s.trim_end_matches('/').trim();
    let mut partes = s_trim.split(';');
    let Some(p0) = partes.next().map(str::trim) else {
        return false;
    };
    let Some(p1) = partes.next().map(str::trim) else {
        return false;
    };
    partes.next().is_none() && p0.parse::<u64>().is_ok() && p1.parse::<u64>().is_ok()
}
