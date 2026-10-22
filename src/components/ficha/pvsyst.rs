//! Module (.PAN) and grid inverter (.OND) creation from datasheet values in
//! PVsyst's own component windows. PVsyst fits the one-diode model (RShunt,
//! RSerie, Gamma...) or builds the efficiency profile, and saves the file.

use crate::adapters::gui::conduccion::{Campo, Formulario};
use crate::adapters::gui::sesion_pvsyst::Sesion;
use crate::config::Paths;
use crate::error::{Error, Result};
use crate::format::object;
use serde_json::{json, Map, Value};
use std::path::{Path, PathBuf};

/// One datasheet value: its key in the PVsyst file and where it is typed.
struct Entrada {
    clave: &'static str,
    campo: Campo,
    pagina: usize,
}

const fn e(clave: &'static str, x: i32, y: i32, pagina: usize) -> Entrada {
    Entrada {
        clave,
        campo: Campo::edit(x, y),
        pagina,
    }
}

/// What differs between creating a module and an inverter.
struct Tipo {
    boton_base: &'static str,
    formulario: &'static str,
    carpeta: &'static str,
    extension: &'static str,
    nombre_archivo: Campo,
    entradas: &'static [Entrada],
    obligatorias: &'static [&'static str],
    leidas: &'static [&'static str],
}

const MODULO: Tipo = Tipo {
    boton_base: "Módulos PV",
    formulario: "TPVModuleForm",
    carpeta: "PVmodules",
    extension: "PAN",
    nombre_archivo: Campo::edit(243, 79),
    entradas: &[
        e("Model", 243, 53, 0),
        e("Manufacturer", 621, 53, 0),
        e("PNom", 130, 157, 0),
        e("Isc", 250, 279, 0),
        e("Voc", 493, 279, 0),
        e("Imp", 250, 303, 0),
        e("Vmp", 493, 303, 0),
        e("muISC", 250, 327, 0),
        e("NCelS", 493, 327, 0),
    ],
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
    ],
    leidas: &[
        "PNom", "Isc", "Voc", "Imp", "Vmp", "NCelS", "RShunt", "RSerie", "Gamma", "muGamma",
        "Technol",
    ],
};

/// Technology list of the module window.
const TECNOLOGIA: Campo = Campo::combo(166, 189);

const INVERSOR: Tipo = Tipo {
    boton_base: "Inversor de red",
    formulario: "TGridInverterForm",
    carpeta: "Inverters",
    extension: "OND",
    nombre_archivo: Campo::edit(241, 84),
    entradas: &[
        e("Model", 241, 58, 0),
        e("Manufacturer", 621, 58, 0),
        e("VMppMin", 286, 171, 0),
        e("VMPPMax", 286, 267, 0),
        e("VAbsMax", 286, 291, 0),
        e("PSeuil", 240, 332, 0),
        e("VOutConv", 546, 250, 0),
        e("PNomConv", 545, 275, 0),
        e("PMaxOUT", 545, 299, 0),
        e("EfficMax", 493, 105, 1),
        e("EfficEuro", 493, 132, 1),
    ],
    obligatorias: &[
        "Model",
        "Manufacturer",
        "PNomConv",
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
        "VMppMin",
        "VMPPMax",
        "VAbsMax",
        "PSeuil",
        "EfficMax",
        "EfficEuro",
        "MonoTri",
    ],
};

/// Grid phases as PVsyst names them in the file and in the window.
const FASES: [(&str, &str); 3] = [
    ("Mono", "Monofásico"),
    ("Tri", "Trifásico"),
    ("Bi", "Bifásico"),
];

fn tipo_de(tipo: &str) -> Result<&'static Tipo> {
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

fn validar(t: &Tipo, datos: &Map<String, Value>) -> Result<()> {
    let faltan: Vec<&str> = t
        .obligatorias
        .iter()
        .copied()
        .filter(|k| !datos.contains_key(*k))
        .collect();
    let conocidas =
        |k: &str| t.obligatorias.contains(&k) || t.entradas.iter().any(|en| en.clave == k);
    let extra: Vec<&String> = datos.keys().filter(|k| !conocidas(k)).collect();
    if faltan.is_empty() && extra.is_empty() {
        return Ok(());
    }
    Err(Error::Invalid(format!(
        "Datos de la ficha: faltan {faltan:?}; no admitidos {extra:?}. PVsyst los pide en su ventana de componente"
    )))
}

fn rellenar(t: &Tipo, f: &Formulario, datos: &Map<String, Value>, archivo: &str) -> Result<()> {
    f.escribir(t.nombre_archivo, archivo)?;
    for pagina in [0, 1] {
        let de_pagina: Vec<&Entrada> = t.entradas.iter().filter(|en| en.pagina == pagina).collect();
        if de_pagina.iter().all(|en| !datos.contains_key(en.clave)) {
            continue;
        }
        f.pagina(pagina)?;
        for en in de_pagina {
            if let Some(v) = texto(datos, en.clave) {
                f.escribir(en.campo, &v)?;
            }
        }
    }
    f.pagina(0)?;
    opciones(f, datos)
}

/// Lists and options: module technology, inverter phases and frequency.
fn opciones(f: &Formulario, datos: &Map<String, Value>) -> Result<()> {
    if let Some(tec) = texto(datos, "Technol") {
        f.seleccionar(TECNOLOGIA, &tec)?;
    }
    if let Some(fases) = texto(datos, "MonoTri") {
        let rotulo = FASES
            .iter()
            .find(|(k, _)| k.eq_ignore_ascii_case(&fases))
            .map(|(_, r)| *r)
            .ok_or_else(|| Error::Invalid(format!("MonoTri '{fases}': Mono, Tri o Bi")))?;
        f.elegir(rotulo)?;
    }
    Ok(())
}

fn guardar(s: &Sesion, f: &Formulario) -> Result<()> {
    f.pulsar("OK")?;
    let confirmar = s.esperar("TSaveComponentForm").map_err(|_| {
        Error::Invalid(format!(
            "PVsyst no acepta el componente: revisa la ficha (la ventana '{}' sigue abierta)",
            f.titulo()
        ))
    })?;
    confirmar.pulsar("Guardar como")?;
    s.esperar("TSaveObjPVForm")?.pulsar("Guardar")?;
    s.aceptar_mensajes();
    Ok(())
}

/// Creates the component in PVsyst and returns what PVsyst saved.
pub(crate) fn crear(
    paths: &Paths,
    tipo: &str,
    nombre: &str,
    datos: &Map<String, Value>,
) -> Result<Value> {
    let t = tipo_de(tipo)?;
    validar(t, datos)?;
    let archivo = format!("{nombre}.{}", t.extension);
    let destino: PathBuf = paths
        .workspace
        .join("ComposPV")
        .join(t.carpeta)
        .join(&archivo);
    if destino.exists() {
        return Err(Error::Invalid(format!("{} ya existe", destino.display())));
    }
    let sesion = Sesion::abrir(paths)?;
    sesion.ejecutar(|s| {
        s.inicio.pulsar("Bases de datos")?;
        s.esperar("TDatabaseForm")?.pulsar(t.boton_base)?;
        s.esperar("TChoixObjetPVForm")?.pulsar("Nuevo")?;
        let f = s.esperar(t.formulario)?;
        rellenar(t, &f, datos, &archivo)?;
        guardar(s, &f)?;
        let avisos = s.mensajes();
        leer_guardado(t, &destino, &avisos)
    })
}

fn leer_guardado(t: &Tipo, destino: &Path, avisos: &[String]) -> Result<Value> {
    if !destino.exists() {
        return Err(Error::Gui(format!(
            "PVsyst no guardó {}",
            destino.display()
        )));
    }
    let bytes = crate::workspace::leer_bytes(destino)?;
    let raiz = object::parse_first(&String::from_utf8_lossy(&bytes))
        .ok_or_else(|| Error::Format("El archivo guardado no es un PVObject".into()))?;
    let valores: Map<String, Value> = t
        .leidas
        .iter()
        .filter_map(|k| raiz.find_deep(k).map(|v| ((*k).to_string(), json!(v))))
        .collect();
    Ok(json!({
        "origen": "pvsyst",
        "archivo": destino.display().to_string(),
        "valores_pvsyst": valores,
        "avisos_pvsyst": avisos,
    }))
}
