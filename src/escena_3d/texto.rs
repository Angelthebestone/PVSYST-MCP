//! Text form of a scene: the `PVObject_=pvShading` tree that PVsyst 8.1.8 saves
//! (UTF-8 with a byte-order mark), instead of the binary-block form of the demos.

use super::dominio::{
    DatosEscena3d, DatosObjeto3d, Escena3d, GeometriaElemental, ObjetoEscena3d, Posicion3d,
    TipoArchivoEscena, TipoFormaElemental,
};
use crate::error::{Error, Result};
use crate::format::decodificar_texto_flexible;
use crate::format::object::{parse_first, PvNode};

const MARCA_UTF8: &[u8] = &[0xEF, 0xBB, 0xBF];
const PREFIJO_OBJETO: &str = "pvShd";

/// Whether `bytes` open with a `PVObject_` line, after an optional byte-order mark.
pub(super) fn es_texto(bytes: &[u8]) -> bool {
    let inicio = bytes.strip_prefix(MARCA_UTF8).unwrap_or(bytes);
    inicio.trim_ascii_start().starts_with(b"PVObject_")
}

pub(super) fn parse_escena_texto(bytes: &[u8], tipo: TipoArchivoEscena) -> Result<Escena3d> {
    let texto = decodificar_texto_flexible(bytes);
    let raiz = parse_first(&texto)
        .ok_or_else(|| Error::Invalid("Escena de texto sin objeto raíz".into()))?;
    if raiz.obj_type != tipo.clase_raiz() {
        return Err(Error::Invalid(format!(
            "Incoherencia entre extensión '{tipo:?}' y clase raíz '{}' (se esperaba {})",
            raiz.obj_type,
            tipo.clase_raiz()
        )));
    }
    let mut nodos = Vec::new();
    objetos_de(&raiz, &mut nodos);
    let objetos = nodos
        .into_iter()
        .map(objeto_de)
        .collect::<Result<Vec<_>>>()?;
    Escena3d::nueva(DatosEscena3d {
        tipo,
        clase_raiz: raiz.obj_type.clone(),
        descripcion: raiz.field("Comment").unwrap_or_default().to_string(),
        version: raiz.field("Version").unwrap_or_default().to_string(),
        calcul: raiz.field("Flags").unwrap_or_default().to_string(),
        objetos,
    })
}

/// The scene's objects (`pvShd*`), wherever the parser nested them.
fn objetos_de<'a>(nodo: &'a PvNode, salida: &mut Vec<&'a PvNode>) {
    for hijo in &nodo.children {
        if hijo.obj_type.starts_with(PREFIJO_OBJETO) {
            salida.push(hijo);
        } else {
            objetos_de(hijo, salida);
        }
    }
}

fn objeto_de(nodo: &PvNode) -> Result<ObjetoEscena3d> {
    ObjetoEscena3d::nuevo(DatosObjeto3d {
        clase: nodo.obj_type.clone(),
        nombre: nodo.field("Comment").unwrap_or_default().to_string(),
        version: nodo.field("Version").unwrap_or_default().to_string(),
        calcul: nodo.field("Flags").unwrap_or_default().to_string(),
        cuenta_linea: None,
        posicion: posicion_de(nodo),
        geometria: geometria_de(nodo),
    })
}

/// Numbers of a `1.0,2.0,3.0,` list; `None` when any is not a number.
fn numeros(texto: &str) -> Option<Vec<f64>> {
    texto
        .split(',')
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .map(|s| s.parse().ok())
        .collect()
}

fn posicion_de(nodo: &PvNode) -> Option<Posicion3d> {
    let origen = numeros(nodo.field("DOrigine")?)?;
    let &[x, y, z] = origen.as_slice() else {
        return None;
    };
    let angulo = |clave| nodo.field(clave)?.trim().parse().ok();
    Some(Posicion3d {
        x,
        y,
        z,
        rotacion_azimut: angulo("DAzimut")?,
        rotacion_inclinacion: angulo("DInclin")?,
    })
}

fn geometria_de(nodo: &PvNode) -> Option<GeometriaElemental> {
    let tipo = nodo.field("ObjetElemType")?;
    Some(GeometriaElemental {
        forma: TipoFormaElemental::de_texto(tipo),
        tipo_pvsyst: Some(tipo.to_string()),
        dimensiones: numeros(nodo.field("Params")?)?,
    })
}
