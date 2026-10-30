//! The shapes of the 3D editor as a schema an agent can read before it asks for
//! a scene (`gui_observar que:esquema entidad:escena_3d.formas`). Pure: it only
//! describes `catalogo.rs`.

use super::formas::{Forma, Parametro, FORMAS};
use serde_json::{json, Value};

/// Keys every object takes besides the values of its shape.
const COMUNES: &[(&str, &str, &str)] = &[
    ("tipo", "", "clave de la forma"),
    ("nombre", "", "opcional: nombre del objeto en PVsyst"),
    ("x", "m", "posición en X"),
    ("y", "m", "posición en Y"),
    ("z", "m", "posición en Z (0 si se omite)"),
    (
        "rotacion_azimut_deg",
        "deg",
        "giro del objeto (0 si se omite)",
    ),
];

fn parametro_json(p: &Parametro) -> Value {
    json!({
        "clave": p.clave,
        "etiqueta": p.etiqueta,
        "unidad": p.unidad,
        "defecto": p.defecto,
        "obligatorio": p.obligatorio,
        "minimo": p.minimo(),
        "maximo": p.maximo(),
    })
}

fn forma_json(forma: &Forma) -> Value {
    json!({
        "tipo": forma.tipo,
        "tipo_pvsyst": forma.tipo_pvsyst,
        "entrada_del_combo": forma.nombre_pvsyst,
        "campos": forma.parametros.iter().map(parametro_json).collect::<Vec<_>>(),
        "comprueba_colisiones": forma.tiene_caja(),
    })
}

/// Every shape with its parameters in the order PVsyst's dialog and file keep them.
pub(crate) fn esquema_de_formas() -> Value {
    json!({
        "entidad": "escena_3d.formas",
        "titulo": "Formas elementales del editor 3D de PVsyst 8.1.8",
        "uso": "gui_actuar accion:escena escena:<nombre> objetos:[{tipo, ...comunes, ...campos}]; simular:true solo valida",
        "comunes": COMUNES
            .iter()
            .map(|(clave, unidad, nota)| json!({"clave": clave, "unidad": unidad, "nota": nota}))
            .collect::<Vec<_>>(),
        "formas": FORMAS.iter().map(forma_json).collect::<Vec<_>>(),
        "avisos": ["Los rangos son límites del MCP; los de PVsyst no están medidos."],
    })
}
