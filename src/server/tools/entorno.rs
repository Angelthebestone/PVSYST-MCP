//! Tools for environment diagnostics, licences, logs, preferences and E2E design.

use super::declaracion::{
    con, Acceso, Anotaciones, Discriminador, Ejecutor, Herramienta, TipoDato,
};
use crate::adapters::gui;
use crate::admin;
use crate::config::Paths;
use crate::easy_mode;
use crate::error::{Error, Result};
use crate::prefs;
use serde_json::{json, Value};

pub(crate) const TOOL_ENTORNO: Herramienta = Herramienta {
    nombre: "entorno",
    descripcion: "Entorno PVsyst: diagnostico, licencia, logs y preferencias. / EN: PVsyst environment: diagnostics, licence, logs and preferences.",
    acceso: Acceso::Normal,
    anotaciones: Anotaciones {
        titulo: "Entorno / Environment",
        es_solo_lectura: false,
        es_destructiva: false,
        es_idempotente: false,
        usa_red: false,
    },
    propiedades: &[
        ("n", TipoDato::Integer),
        ("solo_errores", TipoDato::Boolean),
        ("seccion", TipoDato::String),
        ("clave", TipoDato::String),
        ("valor", TipoDato::String),
        ("carpeta_destino", TipoDato::String),
        ("dry_run", TipoDato::Boolean),
        ("permitir_fuera_del_workspace", TipoDato::Boolean),
    ],
    requeridos: &[],
    discriminador: Some(Discriminador {
        campo: "accion",
        defecto: Some("diagnostico"),
        valores: &[
            "diagnostico",
            "licencia",
            "licencia_sync",
            "licencia_activar",
            "licencia_desactivar",
            "exportar_logs",
            "logs_cli",
            "preferencias",
            "preferencias_editar",
        ],
    }),
    requeridos_por_accion: &[("preferencias_editar", &["seccion", "clave", "valor"])],
    ejecutar: Ejecutor::Simple(ejecutar_entorno),
};

pub(crate) const TOOL_DISENAR_SISTEMA: Herramienta = Herramienta {
    nombre: "disenar_sistema",
    descripcion: "E2E en una llamada: coordenadas + kWp (red o aislado) -> sitio -> proyecto -> dimensionado -> simulacion -> KPIs. / EN: End-to-end in one call: coordinates + kWp (grid or stand-alone) -> site -> project -> sizing -> simulation -> KPIs.",
    acceso: Acceso::Normal,
    anotaciones: Anotaciones {
        titulo: "Diseñar sistema / Design system",
        es_solo_lectura: false,
        es_destructiva: true,
        es_idempotente: false,
        usa_red: false,
    },
    propiedades: &[
        ("latitud", TipoDato::Number),
        ("longitud", TipoDato::Number),
        ("kwp", TipoDato::Number),
        ("tipo_sistema", TipoDato::String),
        ("variante", TipoDato::String),
        ("n_baterias_serie", TipoDato::Integer),
        ("n_baterias_paralelo", TipoDato::Integer),
        ("nombre", TipoDato::String),
        ("plantilla", TipoDato::String),
        ("modulo", TipoDato::String),
        ("inversor", TipoDato::String),
        ("n_mod_serie", TipoDato::Integer),
        ("n_strings", TipoDato::Integer),
        ("pnom_modulo", TipoDato::Number),
        ("tilt", TipoDato::Number),
        ("azimut", TipoDato::Number),
        ("altitud", TipoDato::Number),
        ("timezone", TipoDato::Number),
        ("country_code", TipoDato::String),
        ("region", TipoDato::String),
        ("simular", TipoDato::Boolean),
        ("output_csv", TipoDato::String),
        ("report_pdf", TipoDato::String),
        ("sobrescribir", TipoDato::Boolean),
        ("synthetic_weather", TipoDato::Boolean),
        ("confirmar", TipoDato::Boolean),
        ("permitir_fuera_del_workspace", TipoDato::Boolean),
        ("workspace", TipoDato::String),
    ],
    requeridos: &["latitud", "longitud", "kwp"],
    discriminador: None,
    requeridos_por_accion: &[],
    ejecutar: Ejecutor::Simple(ejecutar_disenar_sistema),
};

fn ejecutar_entorno(paths: &Paths, args: &Value) -> Result<Value> {
    let valor = args
        .get("accion")
        .and_then(Value::as_str)
        .unwrap_or("diagnostico");
    match valor {
        "diagnostico" => {
            Ok(json!({ "preflight": easy_mode::preflight(paths), "entorno": info_entorno(paths) }))
        }
        "licencia" => admin::licencia(paths, &con(args, "accion", json!("info"))),
        "licencia_sync" => admin::licencia(paths, &con(args, "accion", json!("sync"))),
        "licencia_activar" => Ok(comando_licencia("lic-activate -k:<TU_CLAVE>")),
        "licencia_desactivar" => Ok(comando_licencia("lic-deactivate -c:<TU_CUSTOMER_ID>")),
        "exportar_logs" => admin::exportar_logs(paths, args),
        "logs_cli" => prefs::logs_cli(paths, args),
        "preferencias" => prefs::preferencias(paths, args),
        "preferencias_editar" => prefs::preferencias_editar(paths, args),
        _ => Err(Error::Invalid(format!(
            "Valor '{valor}' no soportado para entorno"
        ))),
    }
}

fn comando_licencia(argumentos: &str) -> Value {
    json!({
        "ejecutado": false,
        "comando": format!("PVsystCLI.exe {argumentos}"),
        "instrucciones": "Ejecuta este comando tú mismo en una terminal, sustituyendo el marcador por tu dato real. Esta tool no activa ni desactiva licencias."
    })
}

fn info_entorno(paths: &Paths) -> Value {
    json!({
        "server": { "name": "pvsyst-mcp", "version": env!("CARGO_PKG_VERSION") },
        "herramientas": super::list(paths).len(),
        "workspace": paths.workspace.to_string_lossy(),
        "workspace_exists": paths.workspace.exists(),
        "projects_dir": paths.projects_dir().to_string_lossy(),
        "modules_dir": paths.modules_dir().to_string_lossy(),
        "results_dir": paths.results_dir().to_string_lossy(),
        "cli_exe": paths.cli_exe.to_string_lossy(),
        "cli_exists": paths.cli_exe.exists(),
        "gui_exe": paths.gui_exe.to_string_lossy(),
        "gui_exists": paths.gui_exe.exists(),
        "gui_running": gui::gui_running(),
        "experto": paths.experto,
        "env_overrides": {
            "PVSYST_WORKSPACE": std::env::var("PVSYST_WORKSPACE").ok(),
            "PVSYST_CLI": std::env::var("PVSYST_CLI").ok(),
            "PVSYST_GUI": std::env::var("PVSYST_GUI").ok(),
            "PVSYST_MCP_EXPERTO": std::env::var("PVSYST_MCP_EXPERTO").ok()
        }
    })
}

fn ejecutar_disenar_sistema(paths: &Paths, args: &Value) -> Result<Value> {
    easy_mode::disenar_sistema(paths, args)
}
