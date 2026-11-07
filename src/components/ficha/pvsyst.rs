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
use crate::idioma::Texto;
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
    /// Datasheet keys PVsyst reads in kW, so a value in W would be written as MW.
    potencias_kw: &'static [&'static str],
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
fn avisos_de_unidades(t: &Tipo, datos: &Map<String, Value>) -> Vec<String> {
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

pub(super) fn validar(t: &Tipo, datos: &Map<String, Value>) -> Result<()> {
    validar_potencias_kw(t, datos)?;
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
            valores.insert(clave.into(), valor.clone());
        }
    }
    valores
}

/// Types the datasheet; a field that failed or reads back differently stops before saving.
fn rellenar(paths: &Paths, t: &Tipo, valores: &Map<String, Value>) -> Result<()> {
    let pedido = json!({ "entidad": t.entidad, "valores": valores, "ventana": t.formulario });
    // A value PVsyst limited or did not take must not end up saved in the component.
    crate::vista_agente::rellenar_exacto(paths, &pedido).map(|_| ())
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
/// The focus is tried twice; if it still is not there the click is sent anyway (it has worked
/// without a confirmed focus) but the miss is logged, so a click PVsyst ignores leaves a trace.
fn pulsar_al_frente(dialogo: &Formulario, rotulo: &str) -> Result<()> {
    let campo = ffi::enum_children(dialogo.h)
        .into_iter()
        .find(|c| class_of(*c) == "TEdit");
    let con_foco = match campo {
        Some(c) => ffi::traer_y_focar(c, PAUSA_FOCO) || ffi::traer_y_focar(c, PAUSA_FOCO),
        None => {
            ffi::traer_al_frente(dialogo.h);
            true
        }
    };
    if !con_foco {
        tracing::warn!(
            ventana = dialogo.h,
            rotulo,
            "el diálogo no quedó delante ni con el foco: PVsyst puede ignorar el clic"
        );
    }
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
        guardar(s, &f)?;
        let avisos = s.mensajes();
        leer_guardado(t, &destino, (datos, &avisos))
    })
}

fn leer_guardado(
    t: &Tipo,
    destino: &Path,
    (enviados, avisos): (&Map<String, Value>, &[String]),
) -> Result<Value> {
    if !destino.exists() {
        return Err(Error::Gui(format!(
            "PVsyst no guardó {}",
            destino.display()
        )));
    }
    let bytes = crate::workspace::leer_bytes(destino)?;
    let raiz = object::parse_completo(&String::from_utf8_lossy(&bytes), destino)?
        .into_iter()
        .next()
        .ok_or_else(|| Error::Format("El archivo guardado no es un PVObject".into()))?;
    let valores: Map<String, Value> = t
        .leidas
        .iter()
        .filter_map(|k| raiz.find_deep(k).map(|v| ((*k).to_string(), json!(v))))
        .collect();
    let diferencias = super::contraste::diferencias(&raiz, t.leidas, enviados);
    let mut salida = json!({
        "origen": "pvsyst",
        "archivo": destino.display().to_string(),
        "valores_pvsyst": valores,
        "avisos_pvsyst": avisos,
    });
    let avisos_ficha = avisos_de_unidades(t, enviados);
    if !avisos_ficha.is_empty() {
        salida["avisos_ficha"] = json!(avisos_ficha);
    }
    if !diferencias.is_empty() {
        salida["completo"] = json!(false);
        salida["diferencias_con_la_ficha"] = Value::Array(diferencias);
    }
    Ok(salida)
}
