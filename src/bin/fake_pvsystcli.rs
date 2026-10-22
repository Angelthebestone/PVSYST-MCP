//! Test double for `PVsystCLI.exe`: prints fixture stdout per command so the
//! MCP can be exercised without a PVsyst licence or trial executions.
//!
//! Env: `FAKE_PVSYST_FIXTURES` (required, fixture dir), `FAKE_PVSYST_SLEEP_MS`
//! (optional delay before output). Standard library only.

// A test double whose whole job is to print to stdout/stderr.
#![allow(clippy::print_stdout, clippy::print_stderr)]

use std::env;
use std::fs;
use std::io::Write;
use std::path::Path;
use std::process::ExitCode;
use std::thread;
use std::time::Duration;

/// Tamaño total de salida generada para probar el truncamiento en flujo continuo.
const BYTES_SALIDA_MASIVA: usize = 2 * 1024 * 1024; // 2 MiB

fn main() -> ExitCode {
    let args: Vec<String> = env::args().collect();
    let Some(dir) = env::var_os("FAKE_PVSYST_FIXTURES") else {
        eprintln!("FAKE_PVSYST_FIXTURES no definida");
        return ExitCode::from(2);
    };
    let dir = Path::new(&dir);
    let command = args.get(1).map(String::as_str).unwrap_or("");
    if let Some(ms) = env::var("FAKE_PVSYST_SLEEP_MS")
        .ok()
        .and_then(|v| v.parse::<u64>().ok())
    {
        thread::sleep(Duration::from_millis(ms));
    }

    if command == "run-simulation" && env::var("FAKE_PVSYST_PROGRESO").as_deref() == Ok("1") {
        emitir_progreso();
    }
    if emitir_casos_especiales(command, &args) {
        return ExitCode::SUCCESS;
    }
    // Success fixtures are stored as `<command>-ok.txt`; plain `<command>.txt` wins when present.
    let fixture = dir.join(format!("{command}.txt"));
    let ok_fixture = dir.join(format!("{command}-ok.txt"));
    let fallback = dir.join("unknown-command.txt");
    let stdout = fs::read(&fixture)
        .or_else(|_| fs::read(&ok_fixture))
        .or_else(|_| fs::read(&fallback))
        .unwrap_or_default();
    print!("{}", String::from_utf8_lossy(&stdout));

    match command {
        "run-simulation" => {
            copy_outputs(dir, &args);
            write_simulation_log();
        }
        "create-site" => touch_output(&args, "-osf:"),
        _ => {}
    }
    ExitCode::SUCCESS
}

/// Emits the percentages the real CLI prints while a simulation runs.
fn emitir_progreso() {
    let pausa_ms = env::var("FAKE_PVSYST_SLEEP_MS")
        .ok()
        .and_then(|v| v.parse::<u64>().ok())
        .unwrap_or(0)
        / 3;
    for porcentaje in [10, 50, 100] {
        println!("{porcentaje}%");
        let _ = std::io::stdout().flush();
        thread::sleep(Duration::from_millis(pausa_ms));
    }
}

/// `run-simulation`: copy the canned CSV/PDF to the `-ocf:` / `-rpf:` targets.
fn copy_outputs(dir: &Path, args: &[String]) {
    let results = dir.join("..").join("results");
    for (flag, source) in [("-ocf:", "MCP_E2E.csv"), ("-rpf:", "MCP_E2E.pdf")] {
        if let (Some(target), Ok(_)) = (option(args, flag), fs::metadata(results.join(source))) {
            let _ = fs::copy(results.join(source), target);
        }
    }
}
/// Writes a realistic PVsystCLI log on run-simulation if LOCALAPPDATA is set.
fn write_simulation_log() {
    let Some(local_app_data) = env::var_os("LOCALAPPDATA") else {
        return;
    };
    let admin_dir = Path::new(&local_app_data)
        .join("PVsystCLI")
        .join("8.1")
        .join("Admin");
    if fs::create_dir_all(&admin_dir).is_err() {
        return;
    }
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);
    let log_path = admin_dir.join(format!("{now}Err.log"));
    let log_content = concat!(
        "23:16:40|Norm|TSimulationHandl|ExecSimulation      |LErrorList.Count = 3Incident global = 657 W/m², :The MPP voltage is lower (by 3 V) than the minimum MPP voltage of the inverter.\n",
        "23:16:49|Warn|TSimulation     |HandleErrorMessages |Errors during the simulation:\n",
        "        |    |                |                    |ErrorList: 1882 error messages\n",
        "        |    |                |                    |The array Vmpp at 60 °C  is lower than the inverter minimum operating voltage.\n",
        "        |    |                |                    |This may result in some undervoltage loss. \n",
        "        |    |                |                    |\n",
        "        |    |                |                    |1879 occurrences of MPP voltage lower than min. voltage input of the inverter\n",
        "        |    |                |                    |Incident global = 657 W/m², :The MPP voltage is lower (by 3 V) than the minimum MPP voltage of the inverter.\n",
        "23:16:49|Warn|TEnvManager     |InitLicense         |No license file\n",
        "23:16:49|Err |PVsystCLI       |CMD OUTPUT          |The license file C:\\license.lic is missing.\n",
    );
    let _ = fs::write(log_path, log_content);
}

/// Create an empty file at the value of `flag` (e.g. the `-osf:` output).
fn touch_output(args: &[String], flag: &str) {
    if let Some(target) = option(args, flag) {
        let _ = fs::write(target, b"");
    }
}

fn option<'a>(args: &'a [String], flag: &str) -> Option<&'a str> {
    args.iter().find_map(|a| a.strip_prefix(flag))
}
/// Emite respuestas sintéticas para pruebas de volumen masivo y ambigüedad.
fn emitir_casos_especiales(command: &str, args: &[String]) -> bool {
    if command == "huge-output" || args.iter().any(|a| a == "--huge-output") {
        let linea = "Log de simulación para verificar truncamiento seguro en adaptador CLI.\n";
        let repeticiones = (BYTES_SALIDA_MASIVA / linea.len()) + 1;
        for _ in 0..repeticiones {
            print!("{linea}");
        }
        return true;
    }
    if command == "ambiguous-contradictory" || args.iter().any(|a| a == "--ambiguous-contradictory")
    {
        println!("Checking license...");
        println!("Simulation done in 0min 3sec");
        println!("A fatal error occurred");
        return true;
    }
    if command == "ambiguous-sin-confirmacion"
        || args.iter().any(|a| a == "--ambiguous-sin-confirmacion")
    {
        println!("Checking license...");
        println!("PVsystCLI is running in TRIAL mode, remaining days: 58");
        println!("Comando ejecutado pero sin confirmacion esperada ni error");
        return true;
    }
    false
}
