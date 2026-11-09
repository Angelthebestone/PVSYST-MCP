//! Traducción de los argumentos de `pvsyst_cli` a la línea de comandos.
//!
//! El comando va como primer `argv` y `opciones` como tokens adicionales. Todo lo
//! que no se pueda escribir como un token de PVsystCLI se rechaza aquí, en el
//! borde, en vez de llegar al proceso y fallar con un mensaje del CLI.

use super::linea_comando::validar_log_level;
use crate::args::{req_str, DRY_RUN};
use crate::error::{Error, Result};
use serde_json::Value;

/// Comandos que cambian la licencia. `pvsyst_cli` es un passthrough, pero el
/// servidor promete que nunca activa ni desactiva la licencia: la clave la pone el
/// usuario, fuera del servidor, con el comando que devuelve `entorno`.
const COMANDOS_DE_LICENCIA: [(&str, &str); 3] = [
    ("lic-activate", "licencia_activar"),
    ("lic-deactivate", "licencia_desactivar"),
    ("lic-sync", "licencia_sync"),
];

/// Argumentos de una llamada a `pvsyst_cli`, ya traducidos a lo que se ejecuta.
pub(crate) struct Entrada {
    pub(crate) comando: String,
    pub(crate) cli_args: Vec<String>,
    /// The call is a rehearsal: nothing is executed.
    pub(crate) dry_run: bool,
    /// The caller asked for the whole stdout instead of the tail.
    pub(crate) detalle_completo: bool,
}

impl Entrada {
    /// Whether this command is a simulation, which spends a trial execution.
    pub(crate) fn es_simulacion(&self) -> bool {
        self.comando.eq_ignore_ascii_case("run-simulation")
    }
}

/// Lee los argumentos una sola vez: comando, opciones, ensayo y detalle.
pub(crate) fn leer(args: &Value) -> Result<Entrada> {
    let comando = comando_de(args)?;
    rechazar_comando_de_licencia(comando)?;
    let mut cli_args: Vec<String> = vec![comando.to_string()];
    cli_args.extend(opciones_de(args)?);
    agregar_log_level(args, &mut cli_args)?;
    Ok(Entrada {
        comando: comando.to_string(),
        cli_args,
        dry_run: dry_run_de(args)?,
        detalle_completo: detalle_completo(args),
    })
}

/// `log_level` es el nombre de esta tool para el `-ll` de PVsystCLI; un `-ll` que
/// ya venga en `opciones` manda, para no escribir la opción dos veces.
fn agregar_log_level(args: &Value, cli_args: &mut Vec<String>) -> Result<()> {
    validar_log_level(args)?;
    let Some(nivel) = args.get("log_level").and_then(Value::as_i64) else {
        return Ok(());
    };
    let ya_puesto = cli_args.iter().any(|a| {
        a.starts_with("-ll:") || a.starts_with("--log-level:") || a == "-ll" || a == "--log-level"
    });
    if !ya_puesto {
        cli_args.push(format!("-ll:{nivel}"));
    }
    Ok(())
}

/// `comando`: el nombre del comando, recortado. Un nombre vacío o con espacios o
/// controles se rechaza en vez de llegar a PVsystCLI como un solo token.
fn comando_de(args: &Value) -> Result<&str> {
    let comando = req_str(
        args,
        &["comando"],
        "e.g. \"help\", \"version\", \"lic-info\"",
    )?;
    let nombre = comando.trim();
    if nombre.is_empty() {
        return Err(Error::Invalid(
            "'comando' no puede estar vacío (e.g. \"help\")".to_string(),
        ));
    }
    if nombre.chars().any(|c| c.is_whitespace() || c.is_control()) {
        return Err(Error::Invalid(format!(
            "'comando' es el nombre del comando, sin espacios ni controles (recibido {comando:?})"
        )));
    }
    Ok(nombre)
}

/// Rechaza un comando que cambia la licencia, nombrando la acción que sí lo hace.
fn rechazar_comando_de_licencia(comando: &str) -> Result<()> {
    let Some((_, accion)) = COMANDOS_DE_LICENCIA
        .iter()
        .find(|(clave, _)| comando.eq_ignore_ascii_case(clave))
    else {
        return Ok(());
    };
    Err(Error::Invalid(format!(
        "'{comando}' cambia la licencia y pvsyst_cli no la toca: usa entorno {{accion:\"{accion}\"}}, que devuelve el comando para que lo ejecutes tú con tu clave"
    )))
}

/// `dry_run` como booleano explícito. `null` significaba `false`, así que un
/// ensayo se ejecutaba de verdad; ahora se rechaza en vez de adivinarse.
fn dry_run_de(args: &Value) -> Result<bool> {
    match args.get(DRY_RUN) {
        None => Ok(false),
        Some(Value::Bool(ensayo)) => Ok(*ensayo),
        Some(otro) => Err(Error::Invalid(format!(
            "'dry_run' debe ser true o false; null ejecutaría el comando de verdad (recibido {otro})"
        ))),
    }
}

/// `detalle:completo` pide la salida entera en vez de la cola de 60 líneas.
fn detalle_completo(args: &Value) -> bool {
    args.get("detalle").and_then(Value::as_str) == Some("completo")
}

/// `opciones` como argumentos de PVsystCLI: una lista de textos o un objeto.
/// Cualquier otra cosa, o un elemento que no sea texto, se rechaza.
pub(crate) fn opciones_de(args: &Value) -> Result<Vec<String>> {
    match args.get("opciones") {
        None | Some(Value::Null) => Ok(Vec::new()),
        Some(Value::Array(lista)) => opciones_de_lista(lista),
        Some(Value::Object(mapa)) => opciones_de_objeto(mapa),
        Some(otro) => Err(Error::Invalid(format!(
            "'opciones' debe ser una lista o un objeto (recibido {otro})"
        ))),
    }
}

/// La forma lista lleva las opciones tal cual se escriben en la línea de comandos:
/// vacías o repetidas son un error, porque PVsystCLI toma cada opción una vez.
fn opciones_de_lista(lista: &[Value]) -> Result<Vec<String>> {
    let mut opciones: Vec<String> = Vec::with_capacity(lista.len());
    for elemento in lista {
        let texto = elemento.as_str().ok_or_else(|| {
            Error::Invalid(format!(
                "'opciones' solo admite textos (recibido {elemento})"
            ))
        })?;
        if texto.trim().is_empty() {
            return Err(Error::Invalid(
                "'opciones' no admite elementos vacíos".to_string(),
            ));
        }
        if opciones.iter().any(|v| v == texto) {
            return Err(Error::Invalid(format!(
                "'opciones' repite '{texto}': PVsystCLI toma cada opción una vez"
            )));
        }
        opciones.push(texto.to_string());
    }
    Ok(opciones)
}

/// La forma objeto escribe `--nombre:valor`. Una clave que empieza por `-` se usa
/// tal cual, para poder escribir los alias cortos documentados (`-w`, `-ocf`).
/// `true` es un flag sin valor; `false` y `null` dejan la opción fuera; una lista o
/// un objeto como valor se rechazan en vez de escribirse como texto JSON.
fn opciones_de_objeto(mapa: &serde_json::Map<String, Value>) -> Result<Vec<String>> {
    let mut opciones = Vec::with_capacity(mapa.len());
    for (clave, valor) in mapa {
        let nombre = clave.trim();
        if nombre.is_empty() {
            return Err(Error::Invalid(
                "'opciones' no admite claves vacías".to_string(),
            ));
        }
        let opcion = if nombre.starts_with('-') {
            nombre.to_string()
        } else {
            format!("--{nombre}")
        };
        match valor {
            Value::Bool(true) => opciones.push(opcion),
            Value::Bool(false) | Value::Null => {}
            Value::String(texto) => opciones.push(format!("{opcion}:{texto}")),
            Value::Number(n) => opciones.push(format!("{opcion}:{n}")),
            Value::Array(_) | Value::Object(_) => {
                return Err(Error::Invalid(format!(
                    "'opciones.{nombre}' debe ser texto, número o booleano (recibido {valor})"
                )))
            }
        }
    }
    Ok(opciones)
}

#[cfg(test)]
#[path = "../../../tests/unit/adapters/cli/entrada.rs"]
mod tests;
