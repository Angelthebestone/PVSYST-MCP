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

fn tipo_json(tipo: TipoValor) -> Value {
    match tipo {
        TipoValor::Numero { minimo, maximo } => {
            json!({"tipo": "numero", "minimo": minimo, "maximo": maximo})
        }
        TipoValor::Entero { minimo, maximo } => {
            json!({"tipo": "entero", "minimo": minimo, "maximo": maximo})
        }
        TipoValor::Texto => json!({"tipo": "texto"}),
        TipoValor::Lista => json!({"tipo": "lista"}),
    }
}

fn campo_json(campo: &CampoEsquema, vivo: Option<&CampoVivo>) -> Value {
    let mut v = json!({
        "clave": campo.clave,
        "etiqueta": campo.etiqueta,
        "unidad": campo.unidad,
        "clave_archivo": campo.clave_archivo,
        "editable": campo.es_editable(),
        "tipo": tipo_json(campo.tipo)["tipo"],
    });
    let Some(vivo) = vivo else {
        v["presente"] = json!(false);
        return v;
    };
    v["presente"] = json!(true);
    v["valor"] = json!(vivo.texto);
    v["habilitado"] = json!(vivo.habilitado);
    v["valido"] = json!(es_valido(campo, &vivo.texto));
    if campo.tipo == TipoValor::Lista {
        v["opciones"] = json!(ffi::opciones_de_combo(vivo.hwnd));
    }
    v
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
    let campos: Vec<Value> = ent
        .campos
        .iter()
        .map(|c| campo_json(c, localizar(c, &vivos)))
        .collect();
    let presentes = campos
        .iter()
        .filter(|c| c["presente"] == json!(true))
        .count();
    let mut v = json!({
        "entidad": ent.clave,
        "ventana": format!("0x{h:X}"),
        "titulo": text_of(h),
        "campos": campos,
        "sin_mapear": sin_mapear(ent, &vivos),
    });
    if presentes == 0 {
        v["aviso"] = json!("la ventana no muestra los campos del esquema: hay otra pestaña activa");
    }
    if flag(args, &["ocr"]) {
        let ficha = ocr::formulario(paths, &json!({"ventana": format!("0x{h:X}")}))?;
        v["textos"] = ficha["textos"].clone();
    }
    Ok(v)
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
