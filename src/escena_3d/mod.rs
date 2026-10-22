//! Codec and workspace management for PVsyst 3D shading files (`.SHD`, `.BLD`, `.SHO`).

mod aplicacion;
mod binario;
mod codec;
mod dominio;

pub(crate) use aplicacion::leer_escena;

#[cfg(test)]
#[path = "../../tests/unit/escena_3d/tests.rs"]
mod tests;
