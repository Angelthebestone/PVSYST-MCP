//! How much of a PVsyst window the agent view covers, window class by class.

/// The four ways a window can be covered; each of the 82 classes in
/// `docs/gui_mapa.json` ends in exactly one.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Cobertura {
    /// Schema with key, label, unit, range, file key, type, anchor and access.
    Completa,
    /// Key, control class, anchor and value read by handle; no unit or range.
    Basica,
    /// No fields: described by its buttons.
    SoloAcciones,
    /// Drawn or foreign content: capture only.
    NoAplica,
}

impl Cobertura {
    pub(crate) const TODAS: [Cobertura; 4] = [
        Self::Completa,
        Self::Basica,
        Self::SoloAcciones,
        Self::NoAplica,
    ];

    /// The state number the registry reports (1 to 4).
    pub(crate) const fn numero(self) -> u8 {
        match self {
            Self::Completa => 1,
            Self::Basica => 2,
            Self::SoloAcciones => 3,
            Self::NoAplica => 4,
        }
    }

    pub(crate) const fn nombre(self) -> &'static str {
        match self {
            Self::Completa => "entidad_completa",
            Self::Basica => "entidad_basica",
            Self::SoloAcciones => "solo_acciones",
            Self::NoAplica => "no_aplica",
        }
    }
}

/// One window class and how it is covered.
#[derive(Debug, Clone, Copy)]
pub(crate) struct Ventana {
    pub(crate) clase: &'static str,
    pub(crate) cobertura: Cobertura,
    /// Why, for the states that are not a field entity.
    pub(crate) motivo: &'static str,
    /// Key of the schema entity that describes (part of) its fields.
    pub(crate) esquema: Option<&'static str>,
}

pub(super) const fn basica(clase: &'static str) -> Ventana {
    Ventana {
        clase,
        cobertura: Cobertura::Basica,
        motivo: "",
        esquema: None,
    }
}

pub(super) const fn solo_acciones(clase: &'static str, motivo: &'static str) -> Ventana {
    Ventana {
        clase,
        cobertura: Cobertura::SoloAcciones,
        motivo,
        esquema: None,
    }
}

pub(super) const fn no_aplica(clase: &'static str, motivo: &'static str) -> Ventana {
    Ventana {
        clase,
        cobertura: Cobertura::NoAplica,
        motivo,
        esquema: None,
    }
}

impl Ventana {
    pub(super) const fn con_esquema(self, esquema: &'static str) -> Self {
        Ventana {
            esquema: Some(esquema),
            ..self
        }
    }
}
