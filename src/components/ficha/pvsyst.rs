//! Module (.PAN) and grid inverter (.OND) creation from datasheet values in
//! PVsyst's own component windows. PVsyst fits the one-diode model (RShunt,
//! RSerie, Gamma...) or builds the efficiency profile, and saves the file.

use crate::adapters::gui::conduccion::Formulario;
use crate::adapters::gui::ffi;
use crate::adapters::gui::rotulos::Idioma;
use crate::adapters::gui::sesion_pvsyst::Sesion;
use crate::adapters::gui::ventanas::class_of;
use crate::config::Paths;
use crate::error::{Error, Result};
use crate::format::object;
use crate::vista_agente::clave_para_archivo;
use serde_json::{json, Map, Value};
use std::path::{Path, PathBuf};
use std::time::Duration;

/// Declarative captions and form classes for PVsyst component creation.
struct RotulosFicha {
    ok: &'static str,
    formulario_guardar_componente: &'static str,
    guardar_como: &'static str,
    formulario_guardar_objeto: &'static str,
    guardar: &'static str,
    bases_de_datos: &'static str,
    formulario_base_datos: &'static str,
    formulario_eleccion_objeto: &'static str,
    nuevo: &'static str,
}

const fn rotulos_ficha(idioma: Idioma) -> RotulosFicha {
    match idioma {
        Idioma::Es => RotulosFicha {
            ok: "OK",
            formulario_guardar_componente: "TSaveComponentForm",
            guardar_como: "Guardar como",
            formulario_guardar_objeto: "TSaveObjPVForm",
            guardar: "Guardar",
            bases_de_datos: "Bases de datos",
            formulario_base_datos: "TDatabaseForm",
            formulario_eleccion_objeto: "TChoixObjetPVForm",
            nuevo: "Nuevo",
        },
    }
}

const ROTULOS: RotulosFicha = rotulos_ficha(Idioma::Es);

const PAUSA_FOCO: Duration = Duration::from_millis(300);

/// What differs between creating a module and an inverter. The values are typed
/// by the keys of the entity's schema (`gui_actuar accion:rellenar`), so no
/// coordinate lives here; a datasheet key is the entity field's `clave_archivo`.
pub(super) struct Tipo {
    boton_base: &'static str,
    formulario: &'static str,
    entidad: &'static str,
    carpeta: &'static str,
    extension: &'static str,
    obligatorias: &'static [&'static str],
    leidas: &'static [&'static str],
}

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
    ],
    leidas: &[
        "PNom", "Isc", "Voc", "Imp", "Vmp", "NCelS", "RShunt", "RSerie", "Gamma", "muGamma",
        "Technol",
    ],
};

const INVERSOR: Tipo = Tipo {
    boton_base: "Inversor de red",
    formulario: "TGridInverterForm",
    entidad: "components.inverters",
    carpeta: "Inverters",
    extension: "OND",
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

fn validar(t: &Tipo, datos: &Map<String, Value>) -> Result<()> {
    let faltan: Vec<&str> = t
        .obligatorias
        .iter()
        .copied()
        .filter(|k| !datos.contains_key(*k))
        .collect();
    let conocidas =
        |k: &str| t.obligatorias.contains(&k) || clave_para_archivo(t.entidad, k).is_some();
    let extra: Vec<&String> = datos.keys().filter(|k| !conocidas(k)).collect();
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
    for (clave_archivo, valor) in datos {
        if let Some(clave) = clave_para_archivo(t.entidad, clave_archivo) {
            valores.insert(clave.into(), valor.clone());
        }
    }
    valores
}

fn rellenar(paths: &Paths, t: &Tipo, valores: &Map<String, Value>) -> Result<()> {
    let pedido = json!({ "entidad": t.entidad, "valores": valores, "ventana": t.formulario });
    crate::vista_agente::rellenar(paths, &pedido).map(|_| ())
}

/// Inverter phases: a radio button, which the schema does not model.
fn fases(f: &Formulario, datos: &Map<String, Value>) -> Result<()> {
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
    f.pulsar(ROTULOS.ok)?;
    let confirmar = s
        .esperar(ROTULOS.formulario_guardar_componente)
        .map_err(|_| {
            Error::Invalid(format!(
                "PVsyst no acepta el componente: revisa la ficha (la ventana '{}' sigue abierta)",
                f.titulo()
            ))
        })?;
    pulsar_al_frente(&confirmar, ROTULOS.guardar_como)?;
    let destino = s.esperar(ROTULOS.formulario_guardar_objeto)?;
    pulsar_al_frente(&destino, ROTULOS.guardar)?;
    s.aceptar_mensajes();
    Ok(())
}

/// A click on these dialogs is ignored unless they are in front and hold the
/// focus: the file-name field takes it (the save dialog of a scene needed the same).
fn pulsar_al_frente(dialogo: &Formulario, rotulo: &str) -> Result<()> {
    let campo = ffi::enum_children(dialogo.h)
        .into_iter()
        .find(|c| class_of(*c) == "TEdit");
    match campo {
        Some(c) => ffi::traer_y_focar(c, PAUSA_FOCO),
        None => {
            ffi::traer_al_frente(dialogo.h);
            true
        }
    };
    dialogo.pulsar(rotulo)
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
        s.inicio.pulsar(ROTULOS.bases_de_datos)?;
        s.esperar(ROTULOS.formulario_base_datos)?
            .pulsar(t.boton_base)?;
        s.esperar(ROTULOS.formulario_eleccion_objeto)?
            .pulsar(ROTULOS.nuevo)?;
        let f = s.esperar(t.formulario)?;
        rellenar(paths, t, &valores_de(t, datos, nombre))?;
        fases(&f, datos)?;
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
