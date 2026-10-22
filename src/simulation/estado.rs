use crate::adapters::cli;
use crate::config::Paths;
use crate::error::{Error, Result};
pub(crate) use crate::state::Pendiente;
use serde_json::{json, Value};
use std::fs;
use std::time::UNIX_EPOCH;

pub(crate) fn mtime(path: Option<&str>) -> Option<u64> {
    fs::metadata(path?)
        .ok()?
        .modified()
        .ok()?
        .duration_since(UNIX_EPOCH)
        .ok()
        .map(|d| d.as_secs())
}

pub(crate) fn final_de(paths: &Paths, id: &str, out: &cli::CliOutput) -> Result<Value> {
    let mut lock = paths
        .estado
        .pendientes
        .lock()
        .map_err(|_| Error::Invalid("mutex pendientes envenenado".into()))?;
    let Some(pendiente) = lock.get_mut(id) else {
        return Ok(cli::output_json(out, "run-simulation", &json!({})));
    };
    if let Some(guardado) = &pendiente.resultado {
        return Ok(guardado.clone());
    }
    let res = super::ejecucion::finalizar(&pendiente.plan, out, &pendiente.contexto);
    pendiente.resultado = Some(res.clone());
    Ok(res)
}

pub(crate) fn estado_simulacion(paths: &Paths, args: &Value) -> Result<Value> {
    let id = crate::args::req_str(args, &["trabajo"], "sim-1")?;
    let estado = cli::estado_trabajo(paths, id).ok_or_else(|| {
        let activos: Vec<String> = paths
            .estado
            .pendientes
            .lock()
            .map(|m| m.keys().cloned().collect())
            .unwrap_or_default();
        let pista = if activos.is_empty() {
            "no hay trabajos en segundo plano en esta sesión; lanza uno con simular {segundo_plano:true}".to_string()
        } else {
            format!("trabajos de esta sesión: {}", activos.join(", "))
        };
        Error::NotFound(format!("No existe el trabajo '{id}'; {pista}"))
    })?;
    let comando = {
        let lock = paths.estado.pendientes.lock().ok();
        lock.and_then(|m| m.get(id).map(|p| p.comando.clone()))
            .unwrap_or_else(|| "run-simulation".to_string())
    };
    let mut v = json!({
        "id": id,
        "comando": comando,
        "estado": estado.estado,
        "terminado": estado.terminado,
        "progreso_pct": estado.progreso_pct,
    });
    if let Some(e) = estado.error {
        v["error"] = json!(e);
    }
    if let Some(out) = estado.salida {
        let fin = final_de(paths, id, &out)?;
        if let Some(avisos) = fin.get("avisos_pvsyst") {
            v["avisos_pvsyst"] = avisos.clone();
        }
        v["resultado"] = fin;
    }
    Ok(v)
}
