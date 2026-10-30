//! Entities of the agent view: a PVsyst form described by stable keys instead
//! of captions or coordinates. Pure data, no Win32.

/// Largest distance, in 96-DPI pixels, between a schema anchor and the control
/// found there: the same layout drifts by a pixel or two between PVsyst builds.
pub(crate) const TOLERANCIA_ANCLA_PX: i32 = 6;

/// Where a control sits in the form's own 96-DPI layout (top-left corner).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct Ancla {
    pub(crate) x: i32,
    pub(crate) y: i32,
}

/// What a field accepts.
#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) enum TipoValor {
    Numero {
        minimo: f64,
        maximo: f64,
    },
    Entero {
        minimo: i64,
        maximo: i64,
    },
    Texto,
    Lista,
    /// A check box or an option (radio button): `true` or `false`.
    Casilla,
}

impl TipoValor {
    pub(crate) const fn nombre(&self) -> &'static str {
        match self {
            Self::Numero { .. } => "numero",
            Self::Entero { .. } => "entero",
            Self::Texto => "texto",
            Self::Lista => "lista",
            Self::Casilla => "casilla",
        }
    }
}

/// Whether the agent may change the field.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Acceso {
    Escritura,
    SoloLectura,
}

/// One field of a form: its stable key, where PVsyst stores it and where it is.
#[derive(Debug, Clone, Copy)]
pub(crate) struct CampoEsquema {
    pub(crate) clave: &'static str,
    /// Key of the PVsyst file (`PNom` in a `.PAN`) when the field is stored.
    pub(crate) clave_archivo: Option<&'static str>,
    pub(crate) etiqueta: &'static str,
    pub(crate) unidad: &'static str,
    /// VCL class of the control.
    pub(crate) clase: &'static str,
    pub(crate) tipo: TipoValor,
    pub(crate) acceso: Acceso,
    pub(crate) ancla: Ancla,
    /// Tab (from 0) of the form's `TPageControl` that shows the control.
    pub(crate) pagina: usize,
    /// Key of the check box of this entity that has to be on for the control to take a value.
    pub(crate) depende_de: Option<&'static str>,
}

impl CampoEsquema {
    const fn base(clave: &'static str, etiqueta: &'static str, ancla: Ancla) -> Self {
        Self {
            clave,
            clave_archivo: None,
            etiqueta,
            unidad: "",
            clase: "TEdit",
            tipo: TipoValor::Texto,
            acceso: Acceso::Escritura,
            ancla,
            pagina: 0,
            depende_de: None,
        }
    }

    pub(crate) const fn texto(clave: &'static str, etiqueta: &'static str, ancla: Ancla) -> Self {
        Self::base(clave, etiqueta, ancla)
    }

    #[expect(
        clippy::too_many_arguments,
        reason = "constructor de campo numérico con rango obligatorio de dominio"
    )]
    pub(crate) const fn numero(
        clave: &'static str,
        etiqueta: &'static str,
        ancla: Ancla,
        rango: (f64, f64),
    ) -> Self {
        let mut c = Self::base(clave, etiqueta, ancla);
        c.tipo = TipoValor::Numero {
            minimo: rango.0,
            maximo: rango.1,
        };
        c
    }

    #[expect(
        clippy::too_many_arguments,
        reason = "constructor de campo entero con rango obligatorio de dominio"
    )]
    pub(crate) const fn entero(
        clave: &'static str,
        etiqueta: &'static str,
        ancla: Ancla,
        rango: (i64, i64),
    ) -> Self {
        let mut c = Self::base(clave, etiqueta, ancla);
        c.tipo = TipoValor::Entero {
            minimo: rango.0,
            maximo: rango.1,
        };
        c
    }

    /// A check box.
    pub(crate) const fn casilla(clave: &'static str, etiqueta: &'static str, ancla: Ancla) -> Self {
        let mut c = Self::base(clave, etiqueta, ancla);
        c.clase = "TCheckBox";
        c.tipo = TipoValor::Casilla;
        c
    }

    /// An option of a group (radio button): it can be chosen, and is cleared by choosing another.
    pub(crate) const fn opcion(clave: &'static str, etiqueta: &'static str, ancla: Ancla) -> Self {
        let mut c = Self::base(clave, etiqueta, ancla);
        c.clase = "TRadioButton";
        c.tipo = TipoValor::Casilla;
        c
    }

    pub(crate) const fn lista(clave: &'static str, etiqueta: &'static str, ancla: Ancla) -> Self {
        let mut c = Self::base(clave, etiqueta, ancla);
        c.clase = "TComboBox";
        c.tipo = TipoValor::Lista;
        c
    }

    pub(crate) const fn en_archivo(mut self, clave_archivo: &'static str) -> Self {
        self.clave_archivo = Some(clave_archivo);
        self
    }

    pub(crate) const fn en(mut self, unidad: &'static str) -> Self {
        self.unidad = unidad;
        self
    }

    pub(crate) const fn en_pagina(mut self, pagina: usize) -> Self {
        self.pagina = pagina;
        self
    }

    /// The field only takes a value once the check box `casilla` is on.
    pub(crate) const fn si(mut self, casilla: &'static str) -> Self {
        self.depende_de = Some(casilla);
        self
    }

    pub(crate) const fn solo_lectura(mut self) -> Self {
        self.acceso = Acceso::SoloLectura;
        self
    }

    pub(crate) fn es_editable(&self) -> bool {
        self.acceso == Acceso::Escritura
    }
}

/// A form of PVsyst: its window class and the fields it exposes.
#[derive(Debug)]
pub(crate) struct Entidad {
    /// Stable name the agent uses (`components.panels`).
    pub(crate) clave: &'static str,
    /// VCL class of the window.
    pub(crate) formulario: &'static str,
    pub(crate) titulo: &'static str,
    pub(crate) campos: &'static [CampoEsquema],
}

impl Entidad {
    pub(crate) fn campo(&self, clave: &str) -> Option<&CampoEsquema> {
        self.campos.iter().find(|c| c.clave == clave)
    }

    /// Keys the agent may write, in schema order.
    pub(crate) fn claves_editables(&self) -> Vec<&'static str> {
        self.campos
            .iter()
            .filter(|c| c.es_editable())
            .map(|c| c.clave)
            .collect()
    }
}
