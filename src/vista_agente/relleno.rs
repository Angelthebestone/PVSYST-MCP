//! `gui_actuar accion:rellenar`: types several values into a form by their
//! schema keys and reports what each field holds afterwards.

use super::campos_vivos::{campos_vivos, localizar, CampoVivo};
use super::dominio::{CampoEsquema, Entidad, TipoValor};
use super::esquemas::entidad;
use super::plan::{coincide, planificar, Cambio};
use super::ventana::ventana_de;
use crate::adapters::gui::conduccion::Formulario;
use crate::adapters::gui::ffi::{self, Hwnd};
use crate::args::{flag, missing, req_str};
use crate::error::{Error, Result};
use serde_json::{json, Map, Value};
use std::collections::BTreeMap;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ModoEjecucion {
    Aplicar,
    Simular,
}

struct EscrituraDestino<'a> {
    formulario: &'a Formulario,
    campo: &'a CampoEsquema,
    control: Hwnd,
}

/// A change tied to the live control that receives it.
struct Paso<'a> {
    cambio: &'a Cambio,
    control: &'a CampoVivo,
}

fn emparejar<'a>(cambios: &[&'a Cambio], vivos: &'a [CampoVivo]) -> Result<Vec<Paso<'a>>> {
    cambios
        .iter()
        .map(|&cambio| {
            let control = localizar(cambio.campo, vivos).ok_or_else(|| {
                Error::Gui(format!(
                    "El formulario no coincide con el esquema: no encuentro '{}' ({}). ¿Otra versión de PVsyst u otra pestaña?",
                    cambio.campo.clave, cambio.campo.etiqueta
                ))
            })?;
            if !control.habilitado {
                return Err(Error::Gui(format!(
                    "'{}' está deshabilitado en este momento",
                    cambio.campo.clave
                )));
            }
            Ok(Paso { cambio, control })
        })
        .collect()
}

/// A list value must be one of the items the live list offers, so nothing is
/// typed when the request would fail half way.
fn verificar_listas(pasos: &[Paso]) -> Result<()> {
    for paso in pasos
        .iter()
        .filter(|p| p.cambio.campo.tipo == TipoValor::Lista)
    {
        let opciones = ffi::opciones_de_combo(paso.control.hwnd);
        let pedido = paso.cambio.texto.to_lowercase();
        if opciones
            .iter()
            .any(|o| o.to_lowercase().starts_with(&pedido))
        {
            continue;
        }
        return Err(Error::Invalid(format!(
            "'{}' no ofrece '{}': opciones {opciones:?}",
            paso.cambio.campo.clave, paso.cambio.texto
        )));
    }
    Ok(())
}

fn escribir(destino: &EscrituraDestino<'_>, texto: &str) -> Result<()> {
    if destino.campo.tipo == TipoValor::Lista {
        if ffi::elegir_en_combo(destino.control, texto)? {
            return Ok(());
        }
        return Err(Error::Invalid(format!(
            "La lista '{}' no ofrece '{texto}'",
            destino.campo.clave
        )));
    }
    destino.formulario.escribir_en(destino.control, texto)?;
    Ok(())
}

fn resultado_del_paso(
    formulario: &Formulario,
    paso: &Paso<'_>,
    modo: ModoEjecucion,
) -> Result<Value> {
    let campo = paso.cambio.campo;
    let antes = ffi::texto_control(paso.control.hwnd);
    if modo == ModoEjecucion::Simular {
        return Ok(paso_json(campo, (&paso.cambio.texto, &antes), None));
    }
    let destino = EscrituraDestino {
        formulario,
        campo,
        control: paso.control.hwnd,
    };
    escribir(&destino, &paso.cambio.texto)?;
    let despues = ffi::texto_control(paso.control.hwnd);
    Ok(paso_json(
        campo,
        (&paso.cambio.texto, &antes),
        Some(&despues),
    ))
}

fn paso_json(campo: &CampoEsquema, (pedido, antes): (&str, &str), despues: Option<&str>) -> Value {
    let estado = match despues {
        None => "simulado",
        Some(d) if coincide(campo, pedido, d) => "ok",
        Some(_) => "distinto",
    };
    json!({
        "clave": campo.clave,
        "unidad": campo.unidad,
        "pedido": pedido,
        "antes": antes,
        "despues": despues,
        "estado": estado,
    })
}

/// Fields PVsyst recomputed on its own: mapped or not, those not asked for
/// whose text changed. O(controls).
fn recalculados(entidad: &Entidad, antes: &[CampoVivo], pedidas: &[Paso]) -> Vec<Value> {
    con_texto_actual(antes)
        .into_iter()
        .filter(|(a, _)| !pedidas.iter().any(|p| p.control.hwnd == a.hwnd))
        .filter(|(a, ahora)| a.texto != *ahora)
        .map(|(a, ahora)| {
            let clave = entidad
                .campos
                .iter()
                .find(|c| localizar(c, std::slice::from_ref(a)).is_some())
                .map(|c| c.clave);
            json!({ "clave": clave, "hwnd": format!("0x{:X}", a.hwnd), "antes": a.texto, "despues": ahora })
        })
        .collect()
}

fn con_texto_actual(antes: &[CampoVivo]) -> Vec<(&CampoVivo, String)> {
    antes
        .iter()
        .map(|a| (a, ffi::texto_control(a.hwnd)))
        .collect()
}

/// What a fill acts on: the form, its schema and whether it types or only reports.
struct Aplicacion<'a> {
    formulario: &'a Formulario,
    ent: &'a Entidad,
    modo: ModoEjecucion,
}

/// What a tab of the fill produced: the result of each field and the fields PVsyst recomputed.
type Producido = (Vec<Value>, Vec<Value>);

impl Aplicacion<'_> {
    /// The changes of one tab; the form is already showing it.
    fn aplicar(&self, grupo: &[&Cambio]) -> Result<Producido> {
        let vivos = campos_vivos(self.formulario.h);
        let pasos = emparejar(grupo, &vivos)?;
        verificar_listas(&pasos)?;
        let resultados = pasos
            .iter()
            .map(|p| resultado_del_paso(self.formulario, p, self.modo))
            .collect::<Result<Vec<_>>>()?;
        Ok((resultados, recalculados(self.ent, &vivos, &pasos)))
    }

    /// A simulation does not leave tab 0, so fields of other tabs are only reported.
    fn simular_otra_pagina(grupo: &[&Cambio]) -> Producido {
        let resultados = grupo
            .iter()
            .map(|c| paso_json(c.campo, (&c.texto, ""), None))
            .collect();
        (resultados, Vec::new())
    }
}

/// Applies the changes tab by tab, in tab order. The form is assumed to open on
/// tab 0, and is back on it when done.
fn por_paginas(app: &Aplicacion<'_>, cambios: &[Cambio]) -> Result<Producido> {
    let mut grupos: BTreeMap<usize, Vec<&Cambio>> = BTreeMap::new();
    for c in cambios {
        grupos.entry(c.campo.pagina).or_default().push(c);
    }
    let escribe = app.modo == ModoEjecucion::Aplicar;
    let (mut resultados, mut recalc) = (Vec::new(), Vec::new());
    for (pagina, grupo) in &grupos {
        let (r, rec) = match (*pagina, escribe) {
            (0, _) => app.aplicar(grupo)?,
            (_, false) => Aplicacion::simular_otra_pagina(grupo),
            (n, true) => {
                app.formulario.pagina(n)?;
                app.aplicar(grupo)?
            }
        };
        resultados.extend(r);
        recalc.extend(rec);
    }
    if escribe && grupos.keys().any(|p| *p != 0) {
        app.formulario.pagina(0)?;
    }
    Ok((resultados, recalc))
}

/// Validates, then types (or, with `simular`, only reports) the values.
pub(crate) fn rellenar(paths: &crate::config::Paths, args: &Value) -> Result<Value> {
    let ent = entidad(req_str(
        args,
        &["entidad"],
        "clave de entidad (gui_observar que:esquema)",
    )?)?;
    let valores: &Map<String, Value> = args
        .get("valores")
        .and_then(Value::as_object)
        .ok_or_else(|| missing("valores", "objeto {clave: valor}"))?;
    let cambios = planificar(ent, valores)?;
    let h = ventana_de(args, ent)?;
    let modo = if flag(args, &["simular"]) {
        ModoEjecucion::Simular
    } else {
        ModoEjecucion::Aplicar
    };
    let formulario = Formulario::de(h);
    let aplicacion = Aplicacion {
        formulario: &formulario,
        ent,
        modo,
    };
    let (resultados, recalculados) = por_paginas(&aplicacion, &cambios)?;
    let vista = super::vista::vista(
        paths,
        &json!({"entidad": ent.clave, "ventana": format!("0x{h:X}")}),
    )?;
    Ok(json!({
        "entidad": ent.clave,
        "ventana": format!("0x{h:X}"),
        "simulado": modo == ModoEjecucion::Simular,
        "resultados": resultados,
        "recalculados": recalculados,
        "formulario": vista,
    }))
}
