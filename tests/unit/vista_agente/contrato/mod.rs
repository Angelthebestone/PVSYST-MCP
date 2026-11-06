use super::tipos::{Campo, Estado, Origen, Recorte, Vista};
use super::violacion::Violacion;
use serde_json::json;

fn vista_vacia() -> Vista {
    Vista {
        estado: Estado::Ok,
        resumen: "hecho".into(),
        origen: Origen::Pvsyst,
        campos: Vec::new(),
        avisos: Vec::new(),
        siguiente: Vec::new(),
        recortado: Vec::new(),
        error: None,
    }
}

fn campo(clave: &str) -> Campo {
    Campo {
        clave: clave.into(),
        valor: json!(1),
        unidad: None,
        editable: false,
    }
}

#[test]
fn should_reject_a_view_with_repeated_field_keys() {
    let mut vista = vista_vacia();
    vista.campos = vec![campo("kwp"), campo("kwp")];
    let motivo = vista.validar(&[]).expect_err("repeated key");
    assert_eq!(motivo, Violacion::ClaveRepetida("kwp".into()));
}

#[test]
fn should_reject_a_view_that_cut_a_protected_key() {
    let mut vista = vista_vacia();
    vista.recortado = vec![Recorte {
        clave: "campos".into(),
        mostrados: 1,
        total: 2,
        pedir_resto: "detalle:completo",
    }];
    assert!(vista.validar(&["campos", "botones"]).is_err());
}
