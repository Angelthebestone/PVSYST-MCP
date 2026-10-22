//! Form-level GUI inspection via Win32 control enumeration, UIA tab items
//! and Windows built-in OCR label association.

pub(crate) mod ampliacion;
pub(crate) mod captura_ocr;
pub(crate) mod dominio;
pub(crate) mod formulario;
pub(crate) mod motor;
pub(crate) mod segmentacion;
pub(crate) mod seleccion;
pub(crate) mod ubicacion;

pub(crate) use formulario::{esta_marcado, formulario};
