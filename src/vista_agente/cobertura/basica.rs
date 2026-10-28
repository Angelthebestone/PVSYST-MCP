//! Agent view of a window without a schema: the fields the window shows, keyed
//! by control class and 96-DPI position. No captions, no OCR.

use super::dominio::{Cobertura, Ventana};
use crate::adapters::gui::ffi::{self, Hwnd};
use crate::adapters::gui::ventanas::{hijos_visibles_con_clase, text_of};
use crate::vista_agente::campos_vivos::{campos_vivos, CampoVivo, CLASES_LISTA};
use serde_json::{json, Value};
use std::collections::HashMap;

const CLASES_BOTON: &[&str] = &["TButton", "TBitBtn", "TPngBitBtn", "TSpeedButton"];

/// `edit_97_120`: control class without its `T`, then row and column.
pub(super) fn clave_de(campo: &CampoVivo) -> String {
    format!(
        "{}_{}_{}",
        campo
            .clase
            .strip_prefix('T')
            .unwrap_or(&campo.clase)
            .to_lowercase(),
        campo.posicion.y,
        campo.posicion.x
    )
}

/// Keys in reading order (row, then column). Two controls on the same spot get a suffix.
pub(super) fn campos_con_clave(mut vivos: Vec<CampoVivo>) -> Vec<(String, CampoVivo)> {
    vivos.sort_by_key(|c| (c.posicion.y, c.posicion.x, c.clase.clone()));
    let mut repetidas: HashMap<String, usize> = HashMap::new();
    vivos
        .into_iter()
        .map(|campo| {
            let base = clave_de(&campo);
            let n = repetidas
                .entry(base.clone())
                .and_modify(|n| *n += 1)
                .or_insert(1);
            let clave = if *n == 1 { base } else { format!("{base}_{n}") };
            (clave, campo)
        })
        .collect()
}

fn campo_json((clave, campo): &(String, CampoVivo)) -> Value {
    let mut json = json!({
        "clave": clave,
        "clase": campo.clase,
        "valor": campo.texto,
        "habilitado": campo.habilitado,
        "x": campo.posicion.x,
        "y": campo.posicion.y,
    });
    let es_lista = CLASES_LISTA.contains(&campo.clase.as_str());
    if let (true, Some(obj)) = (es_lista, json.as_object_mut()) {
        obj.insert("opciones".into(), json!(ffi::opciones_de_combo(campo.hwnd)));
    }
    json
}

fn botones(h: Hwnd) -> Vec<Value> {
    hijos_visibles_con_clase(h)
        .filter(|(_, clase)| CLASES_BOTON.contains(&clase.as_str()))
        .map(|(b, _)| {
            json!({ "texto": text_of(b), "habilitado": ffi::estado_ventana(b).habilitada })
        })
        .collect()
}

/// Whether a window of this state shows its fields (by class and position), not only buttons.
pub(super) fn lee_campos_por_clase(cobertura: Cobertura) -> bool {
    matches!(cobertura, Cobertura::Basica | Cobertura::Completa)
}

/// The window as its registration describes it: fields, buttons, or only why it is capture-only.
pub(crate) fn vista_de_ventana(h: Hwnd, registro: &Ventana, titulo: &str) -> Value {
    let mut mapa = json!({
        "entidad": format!("ventana.{}", registro.clase),
        "cobertura": registro.cobertura.numero(),
        "tipo": registro.cobertura.nombre(),
        "ventana": format!("0x{h:X}"),
        "titulo": titulo,
    });
    let Some(obj) = mapa.as_object_mut() else {
        return mapa;
    };
    if registro.cobertura == Cobertura::NoAplica {
        obj.insert("aviso".into(), json!(registro.motivo));
        return mapa;
    }
    if lee_campos_por_clase(registro.cobertura) {
        let campos: Vec<Value> = campos_con_clave(campos_vivos(h))
            .iter()
            .map(campo_json)
            .collect();
        obj.insert("campos".into(), json!(campos));
    }
    obj.insert("botones".into(), json!(botones(h)));
    if !registro.motivo.is_empty() {
        obj.insert("aviso".into(), json!(registro.motivo));
    }
    mapa
}
