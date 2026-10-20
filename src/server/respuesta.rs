//! JSON-RPC envelopes and MCP tool-result shaping.

use crate::error::Error;
use serde_json::{json, Value};

/// Whether tool results may carry `structuredContent`.
#[derive(Clone, Copy)]
pub(crate) enum FormatoResultado {
    ConEstructura,
    SoloTexto,
}

pub(crate) fn ok_response(id: &Value, result: &Value) -> String {
    json!({ "jsonrpc": "2.0", "id": id, "result": result }).to_string()
}

pub(crate) fn err_response(id: &Value, code: i64, message: &str) -> String {
    json!({
        "jsonrpc": "2.0",
        "id": id,
        "error": { "code": code, "message": message }
    })
    .to_string()
}

/// Builds the MCP tool result. A `_imagen_png_base64` key moves out of the JSON
/// text into its own image content item.
pub(crate) fn resultado_de_herramienta(mut value: Value, formato: FormatoResultado) -> Value {
    let imagen = value
        .as_object_mut()
        .and_then(|o| o.remove("_imagen_png_base64"));
    let estructurado = match formato {
        FormatoResultado::ConEstructura if value.is_object() => Some(value.clone()),
        _ => None,
    };
    let mut content = vec![json!({ "type": "text", "text": value.to_string() })];
    if let Some(data) = imagen {
        content.push(json!({ "type": "image", "data": data, "mimeType": "image/png" }));
    }
    let mut resultado = json!({ "content": content, "isError": false });
    if let Some(datos) = estructurado {
        resultado["structuredContent"] = datos;
    }
    resultado
}

pub(crate) fn error_de_herramienta(e: &Error, formato: FormatoResultado) -> Value {
    let mensaje = e.to_string();
    let mut resultado = json!({
        "content": [{ "type": "text", "text": mensaje }],
        "isError": true,
    });
    if matches!(formato, FormatoResultado::ConEstructura) {
        resultado["structuredContent"] = json!({ "error": mensaje, "tipo": e.kind() });
    }
    resultado
}
