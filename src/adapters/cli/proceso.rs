//! Ejecución de procesos de `PVsystCLI.exe`.
//!
//! Gestiona el ciclo de vida del subproceso, la captura con límite de memoria,
//! la cancelación cooperativa y la publicación de avance.

use super::captura::{crear_sink, es_truncado, esperar_lector, texto_sink, Sink};
use super::clasificacion::es_comando_informativo;
use super::linea_comando::{display_command_line, requerir_cli, timeout_de_comando};
use super::salida_json::output_json;
use crate::args::{req_str, DRY_RUN};
use crate::config::Paths;
use crate::error::{Error, Result};
use crate::progreso::SeguimientoTarea;
pub(crate) use crate::state::CliOutput;
use serde_json::{json, Value};
use std::process::{Child, Command, Stdio};
use std::sync::Arc;
use std::thread;
use std::time::{Duration, Instant};

/// Intervalo de sondeo activo para el estado de finalización del subproceso.
const INTERVALO_ESPERA_PROCESO: Duration = Duration::from_millis(50);

/// Valor inicial de "todavía no se informó ningún porcentaje".
const SIN_PROGRESO: f64 = -1.0;

/// Cantidad de bytes ya escaneados en la ronda anterior.
const SIN_BYTES_ESCANEADOS: usize = 0;

/// Plazo para que los lectores lleguen al EOF una vez terminado el proceso.
const PLAZO_LECTORES: Duration = Duration::from_secs(5);

/// Parámetros de ejecución agrupados para cumplir con el límite de parámetros.
pub(crate) struct ParametrosEjecucion<'a> {
    pub(crate) paths: &'a Paths,
    pub(crate) cli_args: &'a [String],
    pub(crate) timeout: Duration,
    pub(crate) seguimiento: &'a SeguimientoTarea<'a>,
}

/// Ejecuta PVsystCLI de forma síncrona esperando hasta alcanzar `timeout`.
pub(crate) fn run(paths: &Paths, cli_args: &[String], timeout: Duration) -> Result<CliOutput> {
    let seguimiento = SeguimientoTarea::inerte();
    run_seguido(&ParametrosEjecucion {
        paths,
        cli_args,
        timeout,
        seguimiento: &seguimiento,
    })
}

/// Ejecuta PVsystCLI reportando avance y atendiendo la cancelación del cliente.
pub(crate) fn run_seguido(params: &ParametrosEjecucion<'_>) -> Result<CliOutput> {
    let comando = params.cli_args.first().map_or("", String::as_str);
    tracing::info!(
        comando,
        timeout_s = params.timeout.as_secs(),
        "launching PVsystCLI"
    );
    let out_sink = crear_sink();
    let out = run_inner(params, &out_sink)?;
    if out.timed_out {
        tracing::warn!(comando, "PVsystCLI timed out and was killed");
    } else {
        tracing::info!(comando, exit_code = ?out.exit_code, "PVsystCLI finished");
    }
    Ok(out)
}

pub(crate) fn run_inner(params: &ParametrosEjecucion<'_>, out_sink: &Sink) -> Result<CliOutput> {
    requerir_cli(params.paths)?;
    let inicio = std::time::SystemTime::now();
    let mut child = Command::new(&params.paths.cli_exe)
        .args(params.cli_args)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|e| {
            Error::io(
                format!("Failed to launch {}", params.paths.cli_exe.display()),
                e,
            )
        })?;

    let err_sink = crear_sink();
    let reader_out = super::captura::drain(child.stdout.take(), Arc::clone(out_sink), "stdout");
    let reader_err = super::captura::drain(child.stderr.take(), Arc::clone(&err_sink), "stderr");

    let espera = EsperaProceso {
        timeout: params.timeout,
        sink: out_sink,
        seguimiento: params.seguimiento,
    };
    let resultado = esperar_proceso(&mut child, &espera);
    let fin_lectura = Instant::now() + PLAZO_LECTORES;
    esperar_lector(reader_out, (out_sink, "stdout"), fin_lectura)?;
    esperar_lector(reader_err, (&err_sink, "stderr"), fin_lectura)?;
    let (exit_code, timed_out) = resultado?;

    let comando = params.cli_args.first().map_or("", String::as_str);
    // A crashed worker thread leaves no trace in stdout, only in the Admin log.
    let crash = if es_comando_informativo(comando) {
        None
    } else {
        super::crash::crash_desde(inicio)
    };

    Ok(CliOutput {
        exit_code,
        stdout: texto_sink(out_sink)?,
        stderr: texto_sink(&err_sink)?,
        command_line: display_command_line(
            &params.paths.cli_exe.to_string_lossy(),
            params.cli_args,
        ),
        timed_out,
        timeout_s: params.timeout.as_secs(),
        stdout_truncado: es_truncado(out_sink)?,
        stderr_truncado: es_truncado(&err_sink)?,
        crash,
    })
}

/// What one wait loop needs: the deadline, the captured stdout and the tracker.
struct EsperaProceso<'a> {
    timeout: Duration,
    sink: &'a Sink,
    seguimiento: &'a SeguimientoTarea<'a>,
}

/// Kills and reaps the child on every early exit, so no PVsystCLI is left running.
/// `kill` fails only when the process already ended, which is what is wanted.
fn terminar(child: &mut Child) {
    if let Err(e) = child.kill() {
        tracing::debug!(error = %e, "PVsystCLI ya había terminado");
    }
    if let Err(e) = child.wait() {
        tracing::warn!(error = %e, "no se pudo recoger el estado de PVsystCLI");
    }
}

fn esperar_proceso(child: &mut Child, espera: &EsperaProceso<'_>) -> Result<(Option<i32>, bool)> {
    let deadline = Instant::now() + espera.timeout;
    let mut ultimo_informado = SIN_PROGRESO;
    let mut bytes_escaneados = SIN_BYTES_ESCANEADOS;
    loop {
        if espera.seguimiento.esta_cancelado() {
            terminar(child);
            return Err(Error::Cli("Comando cancelado por el cliente".into()));
        }
        match child.try_wait() {
            Ok(Some(status)) => return Ok((status.code(), false)),
            Ok(None) => {}
            Err(e) => {
                terminar(child);
                return Err(Error::io("Waiting for PVsystCLI", e));
            }
        }
        if Instant::now() >= deadline {
            terminar(child);
            return Ok((None, true));
        }
        // Solo se copia el stdout acumulado cuando el lector añadió bytes nuevos.
        let longitud = super::captura::longitud_sink(espera.sink);
        if longitud != bytes_escaneados {
            bytes_escaneados = longitud;
            let avance = texto_sink(espera.sink)
                .ok()
                .as_deref()
                .and_then(super::trabajo::progreso);
            if let Some(porcentaje) = avance.filter(|p| *p > ultimo_informado) {
                espera
                    .seguimiento
                    .informar_progreso(porcentaje, "Simulando");
                ultimo_informado = porcentaje;
            }
        }
        thread::sleep(INTERVALO_ESPERA_PROCESO);
    }
}

/// `opciones` as PVsystCLI arguments: a list of texts, or an object `{opcion: valor}`.
/// Anything else, or a list item that is not text, is refused instead of dropped.
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

/// `comando`: the command name, trimmed. A blank name or one carrying spaces or
/// control characters is refused instead of reaching PVsystCLI as one token.
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

/// `dry_run` as an explicit boolean. `null` used to mean `false`, so a rehearsal
/// ran for real; it is refused now instead of guessed.
fn dry_run_de(args: &Value) -> Result<bool> {
    match args.get(DRY_RUN) {
        None => Ok(false),
        Some(Value::Bool(ensayo)) => Ok(*ensayo),
        Some(otro) => Err(Error::Invalid(format!(
            "'dry_run' debe ser true o false; null ejecutaría el comando de verdad (recibido {otro})"
        ))),
    }
}

/// Comandos que cambian la licencia. `pvsyst_cli` es un passthrough, pero el
/// servidor promete que nunca activa ni desactiva la licencia: la clave la pone el
/// usuario, fuera del servidor, con el comando que devuelve `entorno`.
const COMANDOS_DE_LICENCIA: [(&str, &str); 3] = [
    ("lic-activate", "licencia_activar"),
    ("lic-deactivate", "licencia_desactivar"),
    ("lic-sync", "licencia_sync"),
];

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

/// `detalle:completo` pide la salida entera en vez de la cola de 60 líneas.
fn detalle_completo(args: &Value) -> bool {
    args.get("detalle").and_then(Value::as_str) == Some("completo")
}

/// Ejecuta cualquier comando con argumentos en crudo (passthrough de `pvsyst_cli`),
/// atendiendo la cancelación del cliente como cualquier otra ejecución larga.
pub(crate) fn ejecutar(
    paths: &Paths,
    args: &Value,
    seguimiento: &SeguimientoTarea<'_>,
) -> Result<Value> {
    let comando = comando_de(args)?;
    rechazar_comando_de_licencia(comando)?;

    let mut cli_args: Vec<String> = vec![comando.to_string()];
    cli_args.extend(opciones_de(args)?);

    let dry_run = dry_run_de(args)?;
    let command_line = display_command_line(&paths.cli_exe.to_string_lossy(), &cli_args);
    if dry_run {
        return Ok(json!({ "dry_run": true, "command_line": command_line }));
    }
    requerir_cli(paths)?;
    let out = run_seguido(&ParametrosEjecucion {
        paths,
        cli_args: &cli_args,
        timeout: timeout_de_comando(args, comando)?,
        seguimiento,
    })?;
    let mut respuesta = output_json(&out, comando, &json!({ "command": comando }));
    if detalle_completo(args) {
        respuesta["stdout"] = json!(out.stdout);
        respuesta["stdout_tail_truncado"] = json!(false);
    }
    Ok(respuesta)
}

#[cfg(test)]
#[path = "../../../tests/unit/adapters/cli/proceso.rs"]
mod tests;
