use super::cache::{self, ClaveCache};
use crate::adapters::ledger;
use crate::args::{flag, opt_str};
use crate::error::Result;
use serde_json::{json, Value};
use std::path::Path;

pub(crate) use crate::state::Plan;

/// Decisión previa a lanzar PVsystCLI: resultado directo o ejecución pendiente.
pub(crate) enum Previo {
    Listo(Value),
    Ejecutar(Plan),
}

const FILE_OPTIONS: [(&[&str], &str); 10] = [
    (&["output_csv", "csv"], "-ocf"),
    (&["report_pdf", "pdf"], "-rpf"),
    (&["batch_params_file", "bpf"], "-bpf"),
    (&["batch_rvt_file", "brf"], "-brf"),
    (&["input_met_file", "imf"], "-imf"),
    (&["input_sfi_file", "isf"], "-isf"),
    (&["input_param_file", "ipf"], "-ipf"),
    (&["site", "sitio"], "-s"),
    (&["input_rvt_file", "irf"], "-irf"),
    (&["cmd_options_file", "cof"], "-cof"),
];
const DATE_OPTIONS: [(&[&str], &str); 3] = [
    (&["time_step"], "-ts"),
    (&["start_date"], "-sd"),
    (&["end_date"], "-ed"),
];

pub(crate) fn push_options(args: &Value, cli_args: &mut Vec<String>) {
    let valued = |table: &[(&[&str], &str)], out: &mut Vec<String>| {
        for (keys, cli_flag) in table {
            if let Some(v) = opt_str(args, keys) {
                out.push(format!("{cli_flag}:{v}"));
            }
        }
    };
    valued(&FILE_OPTIONS, cli_args);
    if flag(args, &["synthetic_weather"]) {
        cli_args.push("-swg".to_string());
    }
    if let Some(v) = args.get("recalcular_sombreado").and_then(Value::as_bool) {
        cli_args.push(format!("-rst:{}", u8::from(v)));
    }
    if let Some(pags) = args.get("paginas_reporte").and_then(Value::as_array) {
        let lista: Vec<&str> = pags.iter().filter_map(Value::as_str).collect();
        if !lista.is_empty() {
            cli_args.push(format!("-rp:{}", lista.join(",")));
        }
    }
    if let Some(idioma) = opt_str(args, &["idioma_reporte"]) {
        cli_args.push(format!("-rl:{idioma}"));
    }
    valued(&DATE_OPTIONS, cli_args);
    if let Some(v) = args.get("log_level").and_then(Value::as_u64) {
        cli_args.push(format!("-ll:{}", v.min(3)));
    }
    if flag(args, &["overwrite_default_csv"]) {
        cli_args.push("-odc".to_string());
    }
    if let Some(extra) = args.get("extra_options").and_then(Value::as_array) {
        cli_args.extend(extra.iter().filter_map(Value::as_str).map(str::to_string));
    }
}

pub(crate) fn argumentos_cli(
    args: &Value,
    workspace: &Path,
    stem: &str,
    variante: &str,
) -> Vec<String> {
    let mut cli_args = vec![
        "run-simulation".to_string(),
        format!("-w:{}", workspace.display()),
        format!("-p:{stem}.PRJ"),
        format!("-v:{variante}"),
    ];
    push_options(args, &mut cli_args);
    cli_args
}

pub(crate) fn con_salidas_por_defecto(
    args: &Value,
    workspace: &Path,
    stem: &str,
    variante: &str,
) -> Value {
    let mut salidas = args.clone();
    let Some(obj) = salidas.as_object_mut() else {
        return salidas;
    };
    let base = format!("{stem}_{variante}");
    if opt_str(args, &["output_csv", "csv"]).is_none() {
        let ruta = workspace.join("Results").join(format!("{base}.csv"));
        obj.insert("output_csv".into(), json!(ruta.to_string_lossy()));
    }
    if opt_str(args, &["report_pdf", "pdf"]).is_none() && !flag(args, &["sin_pdf"]) {
        let ruta = workspace.join("Results").join(format!("{base}.pdf"));
        obj.insert("report_pdf".into(), json!(ruta.to_string_lossy()));
    }
    salidas
}

pub(crate) fn preparar(args: &Value, workspace: &Path, cli_args: &[String]) -> Result<Previo> {
    let ocf = opt_str(args, &["output_csv", "csv"]).map(str::to_string);
    let rpf = opt_str(args, &["report_pdf", "pdf"]).map(str::to_string);
    let lote = cli_args.iter().any(|a| a.starts_with("-bpf:"));
    let ejecuciones: u32 = if lote { 2 } else { 1 };
    let (clave, advertencia_cache) = if lote || flag(args, &["sin_cache"]) {
        (None, None)
    } else {
        match cache::clave(workspace, cli_args)? {
            ClaveCache::Valida(k) => (Some(k), None),
            ClaveCache::Inutilizable { razon } => (None, Some(razon)),
        }
    };
    if let Some(k) = &clave {
        let ocf_p = ocf.as_deref().map(Path::new);
        let rpf_p = rpf.as_deref().map(Path::new);
        if let Some(mut hit) = cache::buscar(workspace, k, ocf_p, rpf_p)? {
            let restantes = ledger::estado(workspace).and_then(|e| e.restantes);
            hit["costo"] = json!({ "ejecuciones": 0, "restantes": restantes });
            return Ok(Previo::Listo(hit));
        }
    }
    let plan_ledger = [if lote {
        "lote (-bpf)"
    } else {
        "run-simulation"
    }
    .to_owned()];
    if let Some(aviso) = ledger::requiere_confirmacion(workspace, args, ejecuciones, &plan_ledger) {
        return Ok(Previo::Listo(aviso));
    }
    let antes = (
        super::estado::mtime(ocf.as_deref()),
        super::estado::mtime(rpf.as_deref()),
    );
    Ok(Previo::Ejecutar(Plan {
        workspace: workspace.to_path_buf(),
        clave,
        advertencia_cache,
        ocf,
        rpf,
        antes,
        ejecuciones,
    }))
}
