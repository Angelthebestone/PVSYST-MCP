//! Module (.PAN) and grid inverter (.OND) creation from datasheet values in
//! PVsyst's own component windows. PVsyst fits the one-diode model (RShunt,
//! RSerie, Gamma...) or builds the efficiency profile, and saves the file.

use crate::adapters::gui::conduccion::Formulario;
use crate::adapters::gui::ffi;
use crate::adapters::gui::rotulos::dialogos::{GUARDAR, GUARDAR_COMO, NUEVO, OK};
use crate::adapters::gui::rotulos::pvsyst::{BASES_DE_DATOS, INVERSOR_DE_RED, MODULOS_PV};
use crate::adapters::gui::rotulos::Rotulo;
use crate::adapters::gui::sesion_pvsyst::Sesion;
use crate::adapters::gui::ventanas::class_of;
use crate::components::ficha::datos::{avisos_de_unidades, tipo_de, validar, valores_de, Tipo};
use crate::config::Paths;
use crate::error::{Error, Result};
use crate::format::object;
use serde_json::{json, Map, Value};
use std::path::{Path, PathBuf};
use std::time::Duration;

/// Window classes of the component creation flow: the same in every language of PVsyst.
const FORMULARIO_GUARDAR_COMPONENTE: &str = "TSaveComponentForm";
const FORMULARIO_GUARDAR_OBJETO: &str = "TSaveObjPVForm";
const FORMULARIO_BASE_DATOS: &str = "TDatabaseForm";
const FORMULARIO_ELECCION_OBJETO: &str = "TChoixObjetPVForm";

/// The button of the database window that opens the list of `t`. The datasheet
/// type names it by its Spanish caption, which keys the caption table.
fn boton_de_base(t: &Tipo) -> Rotulo<'static> {
    [MODULOS_PV, INVERSOR_DE_RED]
        .into_iter()
        .find(|r| r.es == t.boton_base)
        .unwrap_or(Rotulo::igual(t.boton_base))
}

const PAUSA_FOCO: Duration = Duration::from_millis(300);

/// Types the datasheet; a field that failed or reads back differently stops before saving.
fn rellenar(paths: &Paths, t: &Tipo, valores: &Map<String, Value>) -> Result<()> {
    let pedido = json!({ "entidad": t.entidad, "valores": valores, "ventana": t.formulario });
    // A value PVsyst limited or did not take must not end up saved in the component.
    crate::vista_agente::rellenar_exacto(paths, &pedido).map(|_| ())
}

fn guardar(s: &Sesion, f: &Formulario) -> Result<()> {
    f.pulsar(OK)?;
    let confirmar = s.esperar(FORMULARIO_GUARDAR_COMPONENTE).map_err(|_| {
        Error::Invalid(format!(
            "PVsyst no acepta el componente: revisa la ficha (la ventana '{}' sigue abierta)",
            f.titulo()
        ))
    })?;
    pulsar_al_frente(&confirmar, GUARDAR_COMO)?;
    let destino = s.esperar(FORMULARIO_GUARDAR_OBJETO)?;
    pulsar_al_frente(&destino, GUARDAR)?;
    s.aceptar_mensajes();
    Ok(())
}

/// A click on these dialogs is ignored unless they are in front and hold the
/// focus: the file-name field takes it (the save dialog of a scene needed the same).
/// The focus is tried twice; if it still is not there the click is sent anyway (it has worked
/// without a confirmed focus) but the miss is logged, so a click PVsyst ignores leaves a trace.
fn pulsar_al_frente(dialogo: &Formulario, rotulo: Rotulo<'_>) -> Result<()> {
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
            rotulo = %rotulo,
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
        s.inicio.pulsar(BASES_DE_DATOS)?;
        s.esperar(FORMULARIO_BASE_DATOS)?.pulsar(boton_de_base(t))?;
        s.esperar(FORMULARIO_ELECCION_OBJETO)?.pulsar(NUEVO)?;
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
