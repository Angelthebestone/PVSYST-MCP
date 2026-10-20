//! Adaptador para la interfaz de línea de comandos `PVsystCLI.exe`.
//!
//! Organizado por conceptos de dominio:
//! - `captura`: drenaje de flujos con límite de bytes y detección de envenenamiento.
//! - `clasificacion`: categorización de salida (éxito, error, ambiguo).
//! - `linea_comando`: rutas, comprobación de binarios y formateo de argumentos.
//! - `proceso`: ciclo de vida de procesos síncronos y formato JSON.
//! - `trabajo`: gestión de procesos en segundo plano y seguimiento de avance.

pub(crate) mod captura;
pub(crate) mod clasificacion;
pub(crate) mod linea_comando;
pub(crate) mod proceso;
pub(crate) mod trabajo;

pub(crate) use clasificacion::classify;
pub(crate) use linea_comando::{
    cli_available, display_command_line, timeout_de, TIMEOUT_DEFECTO_S, TIMEOUT_SIMULACION_S,
};
pub(crate) use proceso::{ejecutar, output_json, run, run_seguido, CliOutput, ParametrosEjecucion};
pub(crate) use trabajo::{estado_trabajo, run_en_segundo_plano};
