/// What an edit did to one field. `SinCambio` means the field already held
/// that value, so its line is left untouched.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum AccionCambio {
    Replaced,
    Inserted,
    Deleted,
    SinCambio,
    Modificado,
}

impl AccionCambio {
    pub(crate) fn as_str(self) -> &'static str {
        match self {
            Self::Replaced => "replaced",
            Self::Inserted => "inserted",
            Self::Deleted => "deleted",
            Self::SinCambio => "sin_cambio",
            Self::Modificado => "modificado",
        }
    }
}
