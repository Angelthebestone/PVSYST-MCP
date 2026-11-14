//! A component datasheet: the fields PVsyst's window takes, their validation and the
//! values to type. Pure: no GUI and no files.

use crate::error::{Error, Result};
use crate::idioma::Texto;
use crate::vista_agente::clave_para_archivo;
use serde_json::{json, Map, Value};

/// What differs between creating a module and an inverter. The values are typed
/// by the keys of the entity's schema (`gui_actuar accion:rellenar`), so no
/// coordinate lives here; a datasheet key is the entity field's `clave_archivo`.
pub(super) struct Tipo {
    pub(super) boton_base: &'static str,
    pub(super) formulario: &'static str,
    pub(super) entidad: &'static str,
    pub(super) carpeta: &'static str,
    pub(super) extension: &'static str,
    pub(super) obligatorias: &'static [&'static str],
    pub(super) leidas: &'static [&'static str],
    /// Datasheet keys PVsyst reads in kW, so a value in W would be written as MW.
    pub(super) potencias_kw: &'static [&'static str],
}

/// Largest CA power the inverter window accepts, in kW.
const MAX_POTENCIA_CA_KW: f64 = 10_000.0;
/// Above this the value is more likely a wattage typed into a kW field.
const POTENCIA_CA_SOSPECHOSA_KW: f64 = 1_000.0;

/// A power that looks like watts in a kW field: written, but the caller is told.
const POTENCIA_EN_KW: Texto = Texto {
    es: "'{campo}' se escribe en kW: {valor} son {mw} MW. Si la ficha venía en W, divídela entre 1000",
    en: "'{campo}' is written in kW: {valor} means {mw} MW. If the datasheet was in W, divide it by 1000",
};

const MODULO: Tipo = Tipo {
    boton_base: "Módulos PV",
    formulario: "TPVModuleForm",
    entidad: "components.panels",
    carpeta: "PVmodules",
    extension: "PAN",
    obligatorias: &[
        "Model",
        "Manufacturer",
        "PNom",
        "Technol",
        "Isc",
        "Voc",
        "Imp",
        "Vmp",
        "NCelS",
        // PVsyst refuses to save a module whose size is not defined. In mm, as the
        // window takes them (the file stores metres), so they are not read back.
        "Height",
        "Width",
    ],
    leidas: &[
        "PNom", "Isc", "Voc", "Imp", "Vmp", "NCelS", "RShunt", "RSerie", "Gamma", "muGamma",
        "Technol",
    ],
    potencias_kw: &[],
};

pub(super) const INVERSOR: Tipo = Tipo {
    boton_base: "Inversor de red",
    formulario: "TGridInverterForm",
    entidad: "components.inverters",
    carpeta: "Inverters",
    extension: "OND",
    obligatorias: &[
        "Model",
        "Manufacturer",
        "PNomConv",
        "VOutConv",
        "VMppMin",
        "VMPPMax",
        "VAbsMax",
        "PSeuil",
        "EfficMax",
        "EfficEuro",
        "MonoTri",
    ],
    leidas: &[
        "PNomConv",
        "PMaxOUT",
        "VOutConv",
        "VMppMin",
        "VMPPMax",
        "VAbsMax",
        "PSeuil",
        "EfficMax",
        "EfficEuro",
        "MonoTri",
    ],
    potencias_kw: &["PNomConv", "PMaxOUT"],
};

/// Grid phases as the file names them (`MonoTri`) and the schema option each one chooses.
const FASES: [(&str, &str); 3] = [
    ("Mono", "fase_monofasica"),
    ("Tri", "fase_trifasica"),
    ("Bi", "fase_bifasica"),
];

/// PVsyst's technology list: the value a `.PAN` stores and the label its combo offers.
/// Only the pair measured on 8.1.8 is mapped; any other value is typed as given and
/// PVsyst answers with the options it has.
const TECNOLOGIAS: &[(&str, &str)] = &[("mtSiMono", "Si-mono")];

/// A datasheet value as the window takes it: `Technol` accepts the file's own value.
fn valor_de_ventana(clave: &str, valor: &Value) -> Value {
    if clave != "tecnologia" {
        return valor.clone();
    }
    let Some(texto) = valor.as_str() else {
        return valor.clone();
    };
    TECNOLOGIAS
        .iter()
        .find(|(archivo, _)| archivo.eq_ignore_ascii_case(texto))
        .map_or_else(|| valor.clone(), |(_, etiqueta)| json!(etiqueta))
}

/// The schema option for the phase `fase` of a datasheet, when it is one PVsyst knows.
fn opcion_de_fase(fase: &str) -> Option<&'static str> {
    FASES
        .iter()
        .find(|(k, _)| k.eq_ignore_ascii_case(fase))
        .map(|(_, opcion)| *opcion)
}

pub(super) fn tipo_de(tipo: &str) -> Result<&'static Tipo> {
    match tipo.to_ascii_uppercase().as_str() {
        "PAN" => Ok(&MODULO),
        "OND" => Ok(&INVERSOR),
        otro => Err(Error::Invalid(format!(
            "tipo '{otro}' no soportado: PAN u OND"
        ))),
    }
}

fn texto(datos: &Map<String, Value>, clave: &str) -> Option<String> {
    datos.get(clave).map(|v| match v {
        Value::String(s) => s.clone(),
        otro => otro.to_string(),
    })
}

/// A CA power outside the window's range, or not a number, is refused before it is typed.
fn validar_potencias_kw(t: &Tipo, datos: &Map<String, Value>) -> Result<()> {
    for clave in t.potencias_kw {
        let Some(valor) = datos.get(*clave) else {
            continue;
        };
        let kw = crate::components::buscar::numero(valor)
            .filter(|n| *n > 0.0 && *n <= MAX_POTENCIA_CA_KW);
        if kw.is_none() {
            return Err(Error::Invalid(format!(
                "'{clave}' debe ser un número de kW entre 0 y {MAX_POTENCIA_CA_KW}: {valor}"
            )));
        }
    }
    Ok(())
}

/// Datasheet powers that are probably watts in a kW field, so the caller can check.
pub(super) fn avisos_de_unidades(t: &Tipo, datos: &Map<String, Value>) -> Vec<String> {
    t.potencias_kw
        .iter()
        .filter_map(|clave| {
            let kw = crate::components::buscar::numero(datos.get(*clave)?)?;
            (kw >= POTENCIA_CA_SOSPECHOSA_KW).then(|| {
                POTENCIA_EN_KW.texto_con(
                    crate::idioma::Idioma::de_entorno(),
                    &[
                        ("campo", clave),
                        ("valor", &kw.to_string()),
                        ("mw", &format!("{:.1}", kw / 1000.0)),
                    ],
                )
            })
        })
        .collect()
}

/// Text in a field PVsyst reads as a number would be typed into its window and
/// saved as a component no simulation can use.
fn validar_numericos(datos: &Map<String, Value>) -> Result<()> {
    let cambios: Vec<(String, Option<String>)> = datos
        .iter()
        .map(|(clave, valor)| (clave.clone(), Some(crate::format::value_text(valor))))
        .collect();
    crate::format::campos_numericos::validar_valores(&cambios)
}

pub(super) fn validar(t: &Tipo, datos: &Map<String, Value>) -> Result<()> {
    validar_potencias_kw(t, datos)?;
    validar_numericos(datos)?;
    let faltan: Vec<&str> = t
        .obligatorias
        .iter()
        .copied()
        .filter(|k| !datos.contains_key(*k))
        .collect();
    let conocidas =
        |k: &str| t.obligatorias.contains(&k) || clave_para_archivo(t.entidad, k).is_some();
    let extra: Vec<&String> = datos.keys().filter(|k| !conocidas(k)).collect();
    if let Some(fase) = texto(datos, "MonoTri").filter(|f| opcion_de_fase(f).is_none()) {
        return Err(Error::Invalid(format!("MonoTri '{fase}': Mono, Tri o Bi")));
    }
    if faltan.is_empty() && extra.is_empty() {
        return Ok(());
    }
    Err(Error::Invalid(format!(
        "Datos de la ficha: faltan {faltan:?}; no admitidos {extra:?}. PVsyst los pide en su ventana de componente"
    )))
}

/// The values to type, by the keys of the entity: the file name (without the
/// extension, which PVsyst adds and asks to overwrite when it is typed) and every
/// datasheet value that is a field of the schema. Pure.
pub(super) fn valores_de(t: &Tipo, datos: &Map<String, Value>, nombre: &str) -> Map<String, Value> {
    let mut valores = Map::new();
    valores.insert("nombre_archivo".into(), json!(nombre));
    if let Some(opcion) = texto(datos, "MonoTri").and_then(|f| opcion_de_fase(&f)) {
        valores.insert(opcion.into(), json!(true));
    }
    for (clave_archivo, valor) in datos {
        if let Some(clave) = clave_para_archivo(t.entidad, clave_archivo) {
            valores.insert(clave.into(), valor_de_ventana(clave, valor));
        }
    }
    valores
}
