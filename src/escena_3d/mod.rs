//! Codec and workspace management for PVsyst 3D shading files (`.SHD`, `.BLD`, `.SHO`).

mod aplicacion;
mod binario;
mod codec;
mod dominio;
mod pedido;
mod texto;
pub(crate) use pedido::{
    esquema_de_formas, leer_objetos, nombre_valido, ObjetoPedido, MAX_OBJETOS,
};

pub(crate) use aplicacion::{leer_escena, pedir_escena, verificar_archivo};

#[cfg(test)]
#[path = "../../tests/unit/escena_3d/tests.rs"]
mod tests;
