//! How a `vista` block can break its own contract.

#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub(crate) enum Violacion {
    #[error("clave repetida en campos: {0}")]
    ClaveRepetida(String),
    #[error("se recortó una clave protegida: {0}")]
    ClaveProtegidaRecortada(String),
    #[error("resumen vacío")]
    ResumenVacio,
    #[error("estado error sin bloque error")]
    ErrorSinBloque,
    #[error("siguiente de '{0}' sin argumentos objeto")]
    SiguienteSinObjeto(String),
}
