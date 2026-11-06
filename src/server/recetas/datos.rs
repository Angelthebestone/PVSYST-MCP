//! Static recipe catalogue: the playbooks and the prompt templates derived from them.

/// Whether a prompt argument must be supplied by the client.
pub(crate) enum Requerido {
    Si,
    No,
}

/// One argument of a prompt template.
pub(crate) struct ArgumentoReceta {
    pub(crate) nombre: &'static str,
    pub(crate) descripcion: crate::idioma::Texto,
    pub(crate) requerido: Requerido,
    /// Value substituted when the argument is optional and absent.
    pub(crate) defecto: &'static str,
}

/// A playbook with the tool sequence and the prompt template derived from it.
pub(crate) struct Receta {
    pub(crate) id: &'static str,
    pub(crate) nombre: crate::idioma::Texto,
    pub(crate) cuando: crate::idioma::Texto,
    pub(crate) tools: &'static [&'static str],
    /// JSON text of a sample call, parsed by `como_recurso`.
    pub(crate) ejemplo: Option<&'static str>,
    pub(crate) nota: Option<crate::idioma::Texto>,
    pub(crate) argumentos: &'static [ArgumentoReceta],
    pub(crate) plantilla: crate::idioma::Texto,
    /// The playbook needs the MCP's own calculations, not PVsyst's route.
    pub(crate) requiere_aproximaciones: bool,
}

pub(super) const INTRO: crate::idioma::Texto = crate::idioma::Texto {
    es: "Recetas listas para usar. Cada una indica cuándo usarla y la secuencia de tools.",
    en: "Ready-to-use recipes. Each one states when to use it and the tool sequence.",
};
pub(super) const CONSEJO: crate::idioma::Texto = crate::idioma::Texto {
    es: "Empieza con entorno accion:diagnostico si algo falla. Antes de cada llamada revisa en el esquema de la tool los argumentos obligatorios de su acción. Lo demás es opcional.",
    en: "Start with entorno accion:diagnostico if something fails. Before each call, check the tool's schema for the required arguments of its action. Everything else is optional.",
};

pub(super) static PROYECTO_Y_VARIANTE: [ArgumentoReceta; 2] = [
    ArgumentoReceta {
        nombre: "proyecto",
        descripcion: crate::idioma::Texto {
            es: "Nombre del proyecto",
            en: "Project name",
        },
        requerido: Requerido::Si,
        defecto: "",
    },
    ArgumentoReceta {
        nombre: "variante",
        descripcion: crate::idioma::Texto {
            es: "Variante del proyecto",
            en: "Project variant",
        },
        requerido: Requerido::No,
        defecto: "VC0",
    },
];

/// Every recipe, in the order shown to clients, grouped by intent.
pub(crate) static RECETAS: &[&[Receta]] = &[
    &super::recetas_diseno::GRUPO,
    &super::recetas_consulta::GRUPO,
];
