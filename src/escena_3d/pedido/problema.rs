//! What can be wrong with one object of a scene request. The text is what the
//! agent reads; the variant is what the code and its tests match on.

#[derive(Debug, Clone, PartialEq, thiserror::Error)]
pub(crate) enum Problema {
    #[error("debe ser un objeto")]
    NoEsObjeto,
    #[error("falta 'tipo' (texto)")]
    FaltaTipo,
    #[error("'nombre' debe ser un texto de 1 a 64 caracteres sin saltos")]
    NombreInvalido,
    #[error("tipo '{tipo}' desconocido: usa {validos:?}")]
    TipoDesconocido {
        tipo: String,
        validos: Vec<&'static str>,
    },
    #[error("falta '{clave}'")]
    FaltaClave { clave: &'static str },
    #[error("'{clave}' debe ser un número")]
    NoEsNumero { clave: &'static str },
    #[error("'{clave}' = {valor} fuera de rango [{minimo}, {maximo}] (límites del MCP; los de PVsyst no están medidos)")]
    FueraDeRango {
        clave: &'static str,
        valor: f64,
        minimo: f64,
        maximo: f64,
    },
    #[error("'{clave}' no existe (claves de {tipo}: {validas:?})")]
    ClaveAjena {
        clave: String,
        tipo: &'static str,
        validas: Vec<&'static str>,
    },
}
