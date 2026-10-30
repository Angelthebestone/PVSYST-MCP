//! Path resolution for the PVsyst installation and the user workspace.
//!
//! Everything is overridable through environment variables so the MCP works on
//! machines where PVsyst lives in a different place.

use crate::state::Estado;
use std::ffi::OsString;
use std::path::PathBuf;
use std::sync::Arc;

/// Where PVsyst lives on this machine and how the server may drive it.
#[derive(Debug, Clone)]
pub struct Paths {
    /// The PVsyst user workspace, e.g. `C:\Users\<user>\PVsyst8.1_Data`.
    pub workspace: PathBuf,
    /// The headless CLI, e.g. `C:\Program Files\PVsyst8.1.8\PVsystCLI.exe`.
    pub cli_exe: PathBuf,
    /// The GUI executable, used to drive PVsyst's desktop window.
    pub gui_exe: PathBuf,
    /// Enables the MCP's own approximations of PVsyst calculations
    /// (env PVSYST_MCP_APROXIMACIONES=1). Off: every result comes from PVsyst.
    pub aproximaciones: bool,
    /// Background jobs and caches owned by this configuration (and its clones).
    pub(crate) estado: Arc<Estado>,
}

impl Paths {
    /// Resolves every path from the environment, falling back to the default
    /// newest PVsyst 8.x install found under Program Files.
    ///
    /// Overrides: `PVSYST_WORKSPACE`, `PVSYST_CLI`, `PVSYST_GUI`, `PVSYST_DATARO`
    /// and `PVSYST_MCP_APROXIMACIONES` (`1` or `true`).
    ///
    /// ```
    /// let paths = pvsyst_mcp::Paths::resolve();
    /// assert!(!paths.cli_exe.as_os_str().is_empty());
    /// ```
    #[must_use]
    pub fn resolve() -> Paths {
        Paths {
            workspace: resolve_workspace(),
            cli_exe: resolve_cli(),
            gui_exe: resolve_gui(),
            aproximaciones: bandera_de_entorno(crate::aproximaciones::VARIABLE_ENTORNO),
            estado: Arc::default(),
        }
    }

    /// Construye una instancia para pruebas con rutas aisladas sin acceder al entorno.
    #[cfg(test)]
    #[must_use]
    pub fn para_test(workspace: impl Into<PathBuf>) -> Paths {
        Paths {
            workspace: workspace.into(),
            cli_exe: PathBuf::new(),
            gui_exe: PathBuf::new(),
            aproximaciones: false,
            estado: Arc::default(),
        }
    }

    /// Sub-directory helpers relative to the workspace.
    pub(crate) fn projects_dir(&self) -> PathBuf {
        self.workspace.join("Projects")
    }
    pub(crate) fn modules_dir(&self) -> PathBuf {
        // PVsyst is inconsistent about the case of this folder.
        let a = self.workspace.join("ComposPV").join("PVmodules");
        let b = self.workspace.join("ComposPV").join("PVModules");
        if a.exists() {
            a
        } else {
            b
        }
    }
    pub(crate) fn results_dir(&self) -> PathBuf {
        self.workspace.join("Results")
    }
}

/// A boolean switch from the environment: only `1` or `true` turn it on.
fn bandera_de_entorno(clave: &str) -> bool {
    std::env::var(clave).is_ok_and(|v| matches!(v.trim(), "1" | "true"))
}

/// Where a caller-supplied path may point.
///
/// It lives with the workspace root it governs, and not in `workspace`, so that
/// lower layers — `history` validating a restore manifest, for one — can take it
/// as a parameter without depending upwards.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Alcance {
    /// Confined to the workspace (default): absolute paths outside it and `..` are rejected.
    Workspace,
    /// Any path the process can reach; requires an explicit opt-in from the caller.
    Libre,
}

/// PVsyst's read-only data folder (`DataRO`, next to the GUI executable):
/// demo projects, the bundled component database and the NASA grids.
pub(crate) fn data_ro() -> PathBuf {
    if let Some(p) = ruta_de_entorno("PVSYST_DATARO") {
        return p;
    }
    resolve_gui()
        .parent()
        .map_or_else(
            || PathBuf::from(r"C:\Program Files\PVsyst8.1"),
            std::path::Path::to_path_buf,
        )
        .join("DataRO")
}

/// The bundled read-only component database (`DataRO\PVsyst8.1_Data\ComposPV`).
pub(crate) fn compos_ro() -> PathBuf {
    data_ro().join("PVsyst8.1_Data").join("ComposPV")
}

/// The demo projects shipped with PVsyst (`DataRO\PVsyst8.1_Data\Projects`), read-only.
pub(crate) fn proyectos_demo_ro() -> PathBuf {
    data_ro().join("PVsyst8.1_Data").join("Projects")
}

/// Reads `clave` from the environment as a path, treating an empty or
/// whitespace-only value as unset: installers (the `.mcpb` form, MCP client
/// configs) often pass `""` for fields the user left blank.
pub(crate) fn ruta_de_entorno(clave: &str) -> Option<PathBuf> {
    valor_de_entorno_util(std::env::var_os(clave))
}

/// Pure decision behind [`ruta_de_entorno`]: `None` when the value is absent
/// or blank after trimming.
pub(crate) fn valor_de_entorno_util(valor: Option<OsString>) -> Option<PathBuf> {
    valor
        .filter(|v| !v.to_string_lossy().trim().is_empty())
        .map(PathBuf::from)
}

/// `%LOCALAPPDATA%`, where PVsyst keeps its per-user admin files. Blank counts
/// as undefined, as for every other variable.
pub(crate) fn local_app_data() -> crate::error::Result<PathBuf> {
    ruta_de_entorno("LOCALAPPDATA")
        .ok_or_else(|| crate::error::Error::NotFound("LOCALAPPDATA no está definido".into()))
}

fn home() -> Option<PathBuf> {
    ruta_de_entorno("USERPROFILE").or_else(|| ruta_de_entorno("HOME"))
}

fn first_existing(candidates: Vec<PathBuf>) -> Option<PathBuf> {
    candidates.into_iter().find(|p| p.exists())
}

fn resolve_workspace() -> PathBuf {
    if let Some(p) = ruta_de_entorno("PVSYST_WORKSPACE") {
        return p;
    }
    let mut candidates: Vec<PathBuf> = Vec::new();
    if let Some(h) = home() {
        candidates.push(h.join("PVsyst8.1_Data"));
        candidates.push(h.join("Documents").join("PVsyst8.1_Data"));
        candidates.push(h.join("Documents").join("PVsyst8.1"));
    }
    candidates.push(data_ro().join("PVsyst8.1_Data"));
    first_existing(candidates).unwrap_or_else(|| home().unwrap_or_default().join("PVsyst8.1_Data"))
}

/// Version numbers of an install folder name such as `PVsyst8.1.8` (`[8, 1, 8]`).
/// `None` for anything else, e.g. the data folder `PVsyst8.1_Data` or a major before 8.
pub(crate) fn version_de(carpeta: &str) -> Option<Vec<u32>> {
    let numeros: Vec<u32> = carpeta
        .strip_prefix("PVsyst")?
        .split('.')
        .map(str::parse)
        .collect::<Result<_, _>>()
        .ok()?;
    (numeros.first()? == &8).then_some(numeros)
}

/// The PVsyst 8 install folders found on this machine, newest version first, so an update
/// (8.1.7 → 8.1.8) needs no change here. A PVsyst 9 is not picked: the GUI map, the anchors and
/// the menu positions were measured on 8.1.x, so it needs an explicit `PVSYST_GUI`/`PVSYST_CLI`.
fn instalaciones() -> Vec<PathBuf> {
    let mut raices = vec![
        PathBuf::from(r"C:\Program Files"),
        PathBuf::from(r"C:\Program Files (x86)"),
    ];
    if let Some(h) = home() {
        raices.push(h.join(r"AppData\Local\Programs"));
    }
    let mut halladas: Vec<(Vec<u32>, PathBuf)> = raices
        .iter()
        .filter_map(|r| std::fs::read_dir(r).ok())
        .flatten()
        .flatten()
        .filter_map(|e| Some((version_de(e.file_name().to_str()?)?, e.path())))
        .collect();
    halladas.sort_by(|a, b| b.0.cmp(&a.0));
    halladas.into_iter().map(|(_, ruta)| ruta).collect()
}

fn resolve_exe(variable: &str, exe: &str) -> PathBuf {
    if let Some(p) = ruta_de_entorno(variable) {
        return p;
    }
    let candidates = instalaciones().into_iter().map(|d| d.join(exe)).collect();
    first_existing(candidates).unwrap_or_else(|| PathBuf::from(exe))
}

fn resolve_cli() -> PathBuf {
    resolve_exe("PVSYST_CLI", "PVsystCLI.exe")
}

fn resolve_gui() -> PathBuf {
    resolve_exe("PVSYST_GUI", "PVsyst.exe")
}

#[cfg(test)]
#[path = "../tests/unit/config.rs"]
mod tests;
