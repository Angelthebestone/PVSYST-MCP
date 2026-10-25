//! `gui_observar que:vista` and `que:esquema`: the form as keyed fields.

use super::campos_vivos::{campos_vivos, localizar, CampoVivo};
use super::dominio::{CampoEsquema, Entidad, TipoValor};
use super::esquemas::{de_formulario, entidad, todas};
use super::plan::es_valido;
use super::ventana::ventana_de;
use crate::adapters::gui::ffi;
use crate::adapters::gui::ocr;
use crate::adapters::gui::ventanas::{class_of, hwnd_de, text_of};
use crate::args::{flag, opt_str};
use crate::error::Result;
use serde_json::{json, Value};

fn campo_json(campo: &CampoEsquema, vivo: Option<&CampoVivo>) -> Value {
    let mut map = serde_json::Map::new();
    map.insert("clave".into(), json!(campo.clave));
    map.insert("etiqueta".into(), json!(campo.etiqueta));
    map.insert("unidad".into(), json!(campo.unidad));
    map.insert("clave_archivo".into(), json!(campo.clave_archivo));
    map.insert("editable".into(), json!(campo.es_editable()));
    map.insert("tipo".into(), json!(campo.tipo.nombre()));
    let Some(vivo) = vivo else {
        map.insert("presente".into(), json!(false));
        return Value::Object(map);
    };
    map.insert("presente".into(), json!(true));
    map.insert("valor".into(), json!(vivo.texto));
    map.insert("habilitado".into(), json!(vivo.habilitado));
    map.insert("valido".into(), json!(es_valido(campo, &vivo.texto)));
    if campo.tipo == TipoValor::Lista {
        map.insert("opciones".into(), json!(ffi::opciones_de_combo(vivo.hwnd)));
    }
    Value::Object(map)
}

/// Only what changes between calls: key, value, validity and whether it is enabled.
/// Fields the window does not show are left out.
fn campo_compacto(campo: &CampoEsquema, vivo: Option<&CampoVivo>) -> Option<Value> {
    let vivo = vivo?;
    Some(json!({
        "clave": campo.clave,
        "valor": vivo.texto,
        "valido": es_valido(campo, &vivo.texto),
        "habilitado": vivo.habilitado,
    }))
}

fn sin_mapear(ent: &Entidad, vivos: &[CampoVivo]) -> Vec<Value> {
    vivos
        .iter()
        .filter(|v| {
            !ent.campos
                .iter()
                .any(|c| localizar(c, std::slice::from_ref(v)).is_some())
        })
        .map(|v| {
            json!({
                "clase": v.clase,
                "valor": v.texto,
                "habilitado": v.habilitado,
                "hwnd": format!("0x{:X}", v.hwnd),
            })
        })
        .collect()
}

/// Entity named by `entidad`, or the one whose window is on screen.
fn entidad_pedida(args: &Value) -> Result<Option<&'static Entidad>> {
    if let Some(clave) = opt_str(args, &["entidad"]) {
        return entidad(clave).map(Some);
    }
    if args.get("ventana").is_some() {
        return Ok(de_formulario(&class_of(hwnd_de(args, "ventana")?)));
    }
    Ok(todas()
        .iter()
        .copied()
        .find(|e| ventana_de(args, e).is_ok()))
}

/// The form as keyed fields read live from its controls. `ocr:true` also
/// returns the text PVsyst draws (validation messages, model summary).
pub(crate) fn vista(paths: &crate::config::Paths, args: &Value) -> Result<Value> {
    let Some(ent) = entidad_pedida(args)? else {
        return Ok(json!({
            "entidad": null,
            "aviso": "ninguna ventana conocida abierta; esquemas con gui_observar que:esquema",
        }));
    };
    let h = ventana_de(args, ent)?;
    let vivos = campos_vivos(h);
    let localizados: Vec<_> = ent
        .campos
        .iter()
        .map(|c| (c, localizar(c, &vivos)))
        .collect();
    let presentes = localizados
        .iter()
        .filter(|(_, vivo)| vivo.is_some())
        .count();
    let compacto = flag(args, &["compacto"]);
    let campos: Vec<Value> = localizados
        .iter()
        .filter_map(|&(c, vivo)| {
            if compacto {
                campo_compacto(c, vivo)
            } else {
                Some(campo_json(c, vivo))
            }
        })
        .collect();
    let mut map = serde_json::Map::new();
    map.insert("entidad".into(), json!(ent.clave));
    map.insert("ventana".into(), json!(format!("0x{h:X}")));
    map.insert("titulo".into(), json!(text_of(h)));
    map.insert("campos".into(), json!(campos));
    if !compacto {
        map.insert("sin_mapear".into(), json!(sin_mapear(ent, &vivos)));
    }
    if presentes == 0 {
        map.insert(
            "aviso".into(),
            json!("la ventana no muestra los campos del esquema: hay otra pestaña activa"),
        );
    }
    if flag(args, &["ocr"]) {
        let ficha = ocr::formulario(paths, &json!({"ventana": format!("0x{h:X}")}))?;
        if let Some(textos) = ficha.get("textos") {
            map.insert("textos".into(), textos.clone());
        }
    }
    Ok(Value::Object(map))
}

/// `gui_observar que:esquema`: every entity and its fields.
pub(crate) fn listar_esquemas() -> Value {
    let entidades: Vec<Value> = todas()
        .iter()
        .map(|e| {
            let campos: Vec<Value> = e.campos.iter().map(|c| campo_json(c, None)).collect();
            json!({
                "entidad": e.clave,
                "formulario": e.formulario,
                "titulo": e.titulo,
                "campos": campos,
            })
        })
        .collect();
    json!({ "count": entidades.len(), "entidades": entidades })
}
