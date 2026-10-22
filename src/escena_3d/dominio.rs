//! Domain models and invariants for PVsyst 3D shading scenes and objects (`.SHD`, `.BLD`, `.SHO`).

use crate::error::{Error, Result};

/// File format category for 3D shading data.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum TipoArchivoEscena {
    /// Complete 3D shading scene (`.SHD`).
    Shd,
    /// Building or obstacle cluster (`.BLD`).
    Bld,
    /// Standalone shading object or tracker array (`.SHO`).
    Sho,
}

impl TipoArchivoEscena {
    /// Determines the file category from a file name extension.
    pub(crate) fn de_extension(ext: &str) -> Result<Self> {
        match ext.to_ascii_uppercase().as_str() {
            "SHD" => Ok(Self::Shd),
            "BLD" => Ok(Self::Bld),
            "SHO" => Ok(Self::Sho),
            otro => Err(Error::Invalid(format!(
                "Extensión de escena 3D no soportada: '{otro}' (se esperaba SHD, BLD o SHO)"
            ))),
        }
    }

    /// Returns standard uppercase extension string representation.
    pub(crate) fn as_str(&self) -> &'static str {
        match self {
            Self::Shd => "SHD",
            Self::Bld => "BLD",
            Self::Sho => "SHO",
        }
    }
}

/// 3D placement coordinates and Euler rotations in meters and degrees.
#[derive(Debug, Clone, PartialEq)]
pub(crate) struct Posicion3d {
    pub(crate) x: f64,
    pub(crate) y: f64,
    pub(crate) z: f64,
    pub(crate) rotacion_azimut: f64,
    pub(crate) rotacion_inclinacion: f64,
}

/// Elementary 3D geometric shape kind.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum TipoFormaElemental {
    Parallelepipedo,
    Techo,
    Chimenea,
    Arbol,
    Desconocida(u16),
}

impl TipoFormaElemental {
    pub(crate) fn de_codigo(codigo: u16) -> Self {
        match codigo {
            11 => Self::Parallelepipedo,
            23 => Self::Techo,
            25 => Self::Chimenea,
            21 => Self::Arbol,
            otro => Self::Desconocida(otro),
        }
    }

    pub(crate) fn nombre(&self) -> &'static str {
        match self {
            Self::Parallelepipedo => "Parallélépipède",
            Self::Techo => "Toiture + pignons",
            Self::Chimenea => "Prism - chimney",
            Self::Arbol => "Tree",
            Self::Desconocida(_) => "FormaDesconocida",
        }
    }
}

/// Elementary 3D geometry parameters.
#[derive(Debug, Clone, PartialEq)]
pub(crate) struct GeometriaElemental {
    pub(crate) forma: TipoFormaElemental,
    pub(crate) dimensiones: Vec<f64>,
}

/// Parameters for creating a 3D scene object.
#[derive(Debug, Clone)]
pub(crate) struct DatosObjeto3d {
    pub(crate) clase: String,
    pub(crate) nombre: String,
    pub(crate) version: String,
    pub(crate) calcul: String,
    pub(crate) cuenta_linea: Option<String>,
    pub(crate) posicion: Option<Posicion3d>,
    pub(crate) geometria: Option<GeometriaElemental>,
}

/// Single 3D object inside a scene, building, or object file.
#[derive(Debug, Clone, PartialEq)]
pub(crate) struct ObjetoEscena3d {
    clase: String,
    nombre: String,
    version: String,
    calcul: String,
    cuenta_linea: Option<String>,
    posicion: Option<Posicion3d>,
    geometria: Option<GeometriaElemental>,
}

impl ObjetoEscena3d {
    /// Creates a 3D scene object.
    pub(crate) fn nuevo(datos: DatosObjeto3d) -> Result<Self> {
        if datos.clase.trim().is_empty() {
            return Err(Error::Invalid(
                "Clase de objeto 3D no puede ser vacía".into(),
            ));
        }
        Ok(Self {
            clase: datos.clase,
            nombre: datos.nombre,
            version: datos.version,
            calcul: datos.calcul,
            cuenta_linea: datos.cuenta_linea,
            posicion: datos.posicion,
            geometria: datos.geometria,
        })
    }

    pub(crate) fn clase(&self) -> &str {
        &self.clase
    }

    pub(crate) fn nombre(&self) -> &str {
        &self.nombre
    }

    pub(crate) fn version(&self) -> &str {
        &self.version
    }

    pub(crate) fn calcul(&self) -> &str {
        &self.calcul
    }

    pub(crate) fn cuenta_linea(&self) -> Option<&str> {
        self.cuenta_linea.as_deref()
    }

    pub(crate) fn posicion(&self) -> Option<&Posicion3d> {
        self.posicion.as_ref()
    }

    pub(crate) fn geometria(&self) -> Option<&GeometriaElemental> {
        self.geometria.as_ref()
    }
}

/// Parameters for creating a 3D scene container.
#[derive(Debug, Clone)]
pub(crate) struct DatosEscena3d {
    pub(crate) tipo: TipoArchivoEscena,
    pub(crate) clase_raiz: String,
    pub(crate) descripcion: String,
    pub(crate) version: String,
    pub(crate) calcul: String,
    pub(crate) objetos: Vec<ObjetoEscena3d>,
}

/// Parsed PVsyst 3D shading scene, building, or object container.
#[derive(Debug, Clone, PartialEq)]
pub(crate) struct Escena3d {
    tipo: TipoArchivoEscena,
    clase_raiz: String,
    descripcion: String,
    version: String,
    calcul: String,
    objetos: Vec<ObjetoEscena3d>,
}

impl Escena3d {
    /// Creates a 3D scene container.
    pub(crate) fn nueva(datos: DatosEscena3d) -> Result<Self> {
        if datos.clase_raiz.trim().is_empty() {
            return Err(Error::Invalid("Clase raíz 3D no puede ser vacía".into()));
        }
        Ok(Self {
            tipo: datos.tipo,
            clase_raiz: datos.clase_raiz,
            descripcion: datos.descripcion,
            version: datos.version,
            calcul: datos.calcul,
            objetos: datos.objetos,
        })
    }

    pub(crate) fn tipo(&self) -> TipoArchivoEscena {
        self.tipo
    }

    pub(crate) fn clase_raiz(&self) -> &str {
        &self.clase_raiz
    }

    pub(crate) fn descripcion(&self) -> &str {
        &self.descripcion
    }

    pub(crate) fn version(&self) -> &str {
        &self.version
    }

    pub(crate) fn calcul(&self) -> &str {
        &self.calcul
    }

    pub(crate) fn objetos(&self) -> &[ObjetoEscena3d] {
        &self.objetos
    }
}
