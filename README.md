# PVsyst MCP Server

**English** | [Español](README.es.md)

[![License: Apache 2.0](https://img.shields.io/badge/license-Apache--2.0-blue.svg)](LICENSE)
[![Platform: Windows](https://img.shields.io/badge/platform-Windows-0078D4.svg)](#requirements)
[![PVsyst 8.1.7](https://img.shields.io/badge/PVsyst-8.1.7-2E7D32.svg)](#requirements)
[![Rust](https://img.shields.io/badge/Rust-stable-DEA584?logo=rust&logoColor=black)](#build-from-source)
[![MCP](https://img.shields.io/badge/MCP-2025--06--18%20%7C%202025--03--26%20%7C%202024--11--05-6E56CF.svg)](https://modelcontextprotocol.io)
[![Tests: hermetic](https://img.shields.io/badge/tests-hermetic%20(no%20PVsyst%20required)-3FB950.svg)](#development-checks)

**A [Model Context Protocol](https://modelcontextprotocol.io) (MCP) server that automates
PVsyst 8.1.7 from an AI agent** — batch simulations, project and variant creation,
component and weather handling, and result extraction — by reading and writing **only
PVsyst's data files** and running the **documented `PVsystCLI` commands** as a black box.

It is the missing API for PVsyst: there is no public scripting interface an agent can
call, so this server gives agents one through the artifacts PVsyst already reads and
writes.

> **Unofficial.** Not affiliated with, endorsed by or sponsored by PVsyst SA. "PVsyst"
> is a trademark of its respective owner and is used here only to describe
> interoperability. This project contains no PVsyst code and redistributes no PVsyst
> software: you need your own licensed installation. See
> [responsible use](#responsible-use-licensing-and-trademark).

> **Hard rules this project follows**
> * No PVsyst source code, no decompilation. `PVsyst.exe` is never opened in
>   Ghidra/IDA; `PVsystCLI.exe` is only ever *run* black-box.
> * No simulation engine is implemented here. When a simulation is needed the
>   MCP shells out to the **documented** `PVsystCLI run-simulation` command.
> * The MCP prepares, duplicates and inspects projects; PVsyst does the physics.

**Contents**

- [Why this exists](#why-this-exists)
- [Tools](#what-it-exposes-15-tools-12-by-default)
- [Coverage](#coverage)
- [Requirements](#requirements)
- [Build from source](#build-from-source)
- [Install into an MCP client](#install-into-an-mcp-client)
- [Usage examples](#usage-examples)
- [Supported file formats](#file-format-documentation)
- [Verification (evidence)](#verification-evidence)
- [Project layout](#layout)
- [FAQ](#faq)
- [Responsible use, licensing and trademark](#responsible-use-licensing-and-trademark)
- [Contributing](#contributing)
- [Development checks](#development-checks)

## Why this exists

PVsyst is the reference tool for photovoltaic system design and bankable yield
simulation, and it ships **no public API**: automation means either writing batch files
for `PVsystCLI` by hand, or clicking the desktop UI. That gap is exactly what an agent
cannot cross on its own, so this server closes it:

* **Headless execution.** An agent prepares a project, runs the simulation through the
  documented CLI and reads the results, with no window in the loop.
* **Byte-preserving edits.** Project files are plain UTF-8 `Key=Value` "PVObject"
  records; the server edits fields, inserts and removes objects without touching a byte
  it was not asked to change.
* **Everything is snapshotted.** Any write is preceded by a snapshot, so an experiment
  can be undone.
* **The GUI-only surface is reachable** through UI Automation, Win32 messages and form-field
  driving, for the dialogs that have no file or CLI equivalent.

## What it exposes (15 tools)

Each tool takes a discriminator (`accion` or `que`); omitted values use the default shown.
All 15 tools are available by default. The MCP's own calculations are the only opt-in surface.

**Every figure comes from PVsyst.** By default the MCP only returns what PVsyst computes
(`PVsystCLI`, the variant it simulates and its PDF report). The MCP's own calculations that
duplicate a PVsyst function sit behind `PVSYST_MCP_APROXIMACIONES=1`: without it, the actions
that only exist as MCP calculations are absent from `tools/list` and, if called, answer with the
PVsyst route that replaces them; with it, their result carries `origen: "aproximacion_mcp"`.
Those actions are `simular preestimar`/`barrido`, `sitio geometria_solar`,
`resultados medido_vs_simulado` and the automatic sizing of `disenar_sistema`. Actions that keep
PVsyst as their default route stay visible: `componente crear_desde_ficha`, `sitio
horizonte_pvgis`, `resultados p50_p90` and `proyecto dimensionar` from `kwp` use PVsyst's windows
and only run the MCP calculation when called with `via: "aproximacion"` and the flag is on.

| Tool | Discriminator (default) | Purpose |
|---|---|---|
| `entorno` | `accion`: `diagnostico` (default), `licencia`, `licencia_sync`, `licencia_activar`, `licencia_desactivar`, `exportar_logs`, `logs_cli`, `preferencias`, `preferencias_editar` | Environment, licence, logs, PVsyst preferences. `licencia_activar`/`licencia_desactivar` never execute; they return the command template. |
| `disenar_sistema` | — | Coordinates + kWp → site, project, sizing, simulation, KPIs in one call. |
| `listar` | `que`: `proyectos` (default), `componentes`, `modulos`, `archivos`, `objetos` | Lists projects, components, modules (with `pnom_min`/`pnom_max`/`texto` filters), files, objects. |
| `leer` | type detected from `archivo` | Reads a project, module (`.PAN`), site, meteo, component, PVObject file, or raw file bytes. |
| `editar` | `accion`: `campos` (default), `insertar_objeto`, `eliminar_objeto` | Edits fields of any object (byte-preserving, snapshot first) or inserts/removes objects. |
| `escribir_archivo` | — | Writes a workspace file (snapshot first; path-guarded). |
| `proyecto` | `accion`: `crear`, `eliminar`, `renombrar`, `crear_variante`, `dimensionar`, `doctor`, `anadir`, `seguidor`, `limite_inyeccion`, `almacenamiento`, `perdidas`, `bombeo`, `economia`, `p50_p90`, `limpiar_temporales` | Project and variant lifecycle, consistency checks and repairs, tracker conversion, injection limit, storage, losses, pumping, PVsyst economics, P50/P90 set-up and temp cleanup. `dimensionar` uses PVsyst's pre-sizing window; pass `n_mod_serie`/`n_strings` to write them as given. |
| `componente` | `accion`: `crear`, `borrar`, `importar`, `exportar`, `crear_desde_ficha`* | Component lifecycle; for real modules and inverters, import the manufacturer's `.PAN`/`.OND`. |
| `sitio` | `accion`: `crear` (`origen`: `coordenadas` (default) or `csv`), `convertir_meteo`, `meteo_pvgis`, `geometria_solar`*, `horizonte_pvgis`* | Sites (`PVsystCLI create-site`), weather conversion (`convert-meteo`) and PVGIS hourly TMY converted by PVsyst. |
| `simular` | `accion`: `ejecutar` (default), `preparar`, `estado`, `preestimar`*, `barrido`* | Runs simulations (CSV and PDF report default to `Results/<proyecto>_<variante>.csv` and `.pdf`; cache, trial ledger, background jobs), prepares batch files and returns `avisos_pvsyst`, the warnings PVsyst wrote to its log. Orientation sweeps use `filas` (batch mode `-bpf`). |
| `resultados` | `que`: `kpis` (default), `csv`, `perdidas`, `mensual`, `comparar`, `carbono`, `economia`, `entregable`, `p50_p90`*, `medido_vs_simulado`* | KPIs, losses, monthly table, carbon balance and economic evaluation read from PVsyst's PDF report; hourly CSV; Markdown deliverable. |
| `workspace` | `accion`: `crear`, `exportar`, `importar`, `exportar_proyecto`, `historial`, `restaurar` | Workspace admin and history (snapshots) restore. |
| `pvsyst_cli` | — | Runs any PVsystCLI command with raw options. |
| `gui_observar` | `que`: `ventanas` (default), `controles`, `leer`, `capturar`, `uia`, `arbol`, `mensajes`, `funciones`, `formulario`, `vista`, `esquema`, `menus` | Observes PVsyst windows, controls, UI Automation tree and its log messages; captures PNG; `funciones` lists the GUI catalogue; `formulario` pairs each field of a dialog with its label (Windows OCR). |
| `gui_actuar` | `accion`: `click` (default), `clic_fisico`, `clic_texto`, `mensaje`, `cerrar`, `activar`, `teclas`, `escribir`, `seleccionar`, `pagina`, `esperar_cierre`, `lanzar`, `reiniciar`, `uia_invocar`, `esperar`, `paso`, `funcion`, `menu`, `campo`, `rellenar` | Drives PVsyst windows and controls, including typing into measured 96-DPI fields, combo and list selection and tab pages; `funcion` opens a catalogued PVsyst function (`gui_observar que:funciones`), `menu` posts a menu command (by `id`, or by `menu`+`item` position read from the open popup), `campo` sets a field by its label, `rellenar` fills a whole form from `{key: value}` (`entidad:"components.panels"`; `simular:true` only reports) and `gui_observar que:vista` reads it back by the same keys, without OCR, `clic_texto` clicks on-screen text (controls without a handle), `reiniciar` recovers from a PVsyst crash dialog. |

### Tool Synonyms and Aliases

<!-- alias:inicio -->
| Herramienta / Tool | Argumentos (alias → canónico) | Valores (alias → canónico) |
|---|---|---|
| `entorno` | `action` → `accion`, `destination_folder` → `carpeta_destino`, `key` → `clave`, `section` → `seccion`, `target_folder` → `carpeta_destino`, `value` → `valor` | `diagnostic` → `diagnostico`, `diagnostics` → `diagnostico`, `export_logs` → `exportar_logs`, `license_activate` → `licencia_activar`, `license_deactivate` → `licencia_desactivar`, `license_sync` → `licencia_sync`, `license` → `licencia`, `preferences_edit` → `preferencias_editar`, `preferences` → `preferencias` |
| `disenar_sistema` | `altitude` → `altitud`, `inverter` → `inversor`, `latitude` → `latitud`, `longitude` → `longitud`, `module` → `modulo`, `name` → `nombre`, `overwrite` → `sobrescribir`, `simulate` → `simular`, `system_type` → `tipo_sistema`, `template` → `plantilla`, `variant` → `variante` | — |
| `listar` | `file` → `archivo`, `folder` → `carpeta`, `recursive` → `recursivo`, `text` → `texto`, `type` → `tipo`, `what` → `que` | `components` → `componentes`, `files` → `archivos`, `modules` → `modulos`, `objects` → `objetos`, `projects` → `proyectos` |
| `leer` | `file` → `archivo`, `flat` → `plano`, `limit` → `limite`, `object` → `objeto`, `series` → `serie`, `type` → `tipo` | — |
| `editar` | `action` → `accion`, `edits` → `ediciones`, `fields` → `campos`, `file` → `archivo`, `from` → `desde`, `object` → `objeto`, `operation` → `operacion`, `parent` → `padre`, `project` → `proyecto`, `replace` → `reemplazar`, `type` → `tipo`, `variant` → `variante` | `delete_object` → `eliminar_objeto`, `fields` → `campos`, `insert_object` → `insertar_objeto`, `remove_object` → `eliminar_objeto` |
| `escribir_archivo` | `content` → `contenido`, `file` → `archivo` | — |
| `proyecto` | `action` → `accion`, `battery` → `bateria`, `comment` → `comentario`, `currency` → `moneda`, `element` → `elemento`, `inverter` → `inversor`, `module` → `modulo`, `new_name` → `nombre_nuevo`, `new_variant` → `variante_nueva`, `overwrite` → `sobrescribir`, `project` → `proyecto`, `pump` → `bomba`, `repair` → `reparar`, `source_variant` → `variante_origen`, `strategy` → `estrategia`, `template` → `plantilla`, `variant` → `variante` | `add_object` → `anadir`, `add` → `anadir`, `clean_temps` → `limpiar_temporales`, `clone_variant` → `crear_variante`, `convert_tracker` → `seguidor`, `create_variant` → `crear_variante`, `create` → `crear`, `delete` → `eliminar`, `economics` → `economia`, `economy` → `economia`, `injection_limit` → `limite_inyeccion`, `losses` → `perdidas`, `pumping` → `bombeo`, `remove` → `eliminar`, `rename` → `renombrar`, `size` → `dimensionar`, `sizing` → `dimensionar`, `storage` → `almacenamiento`, `tracker` → `seguidor` |
| `componente` | `action` → `accion`, `changes` → `cambios`, `component` → `componente`, `data` → `datos`, `destination` → `destino`, `file` → `componente`, `new_name` → `nombre_nuevo`, `source` → `origen`, `target` → `destino`, `template` → `plantilla`, `type` → `tipo` | `create_from_datasheet` → `crear_desde_ficha`, `create` → `crear`, `datasheet` → `crear_desde_ficha`, `delete` → `borrar`, `edit` → `editar`, `export` → `exportar`, `import` → `importar`, `remove` → `borrar` |
| `sitio` | `action` → `accion`, `altitude` → `altitud`, `date` → `fecha`, `file` → `archivo`, `latitude` → `latitud`, `longitude` → `longitud`, `points` → `puntos`, `project` → `proyecto`, `site_name` → `nombre_sitio`, `title` → `titulo`, `variant` → `variante` | `convert_meteo` → `convertir_meteo`, `create` → `crear`, `pvgis_horizon` → `horizonte_pvgis`, `pvgis_meteo` → `meteo_pvgis`, `solar_geometry` → `geometria_solar`, `write_horizon` → `escribir_horizonte`, `write_series` → `escribir_serie` |
| `simular` | `action` → `accion`, `job` → `trabajo`, `latitude` → `latitud`, `longitude` → `longitud`, `parameters` → `parametros`, `project` → `proyecto`, `rows` → `filas`, `target` → `destino`, `variant` → `variante` | `execute` → `ejecutar`, `preestimate` → `preestimar`, `prepare` → `preparar`, `run` → `ejecutar`, `status` → `estado`, `sweep` → `barrido` |
| `resultados` | `file` → `archivo`, `project` → `proyecto`, `variant` → `variante`, `variants` → `variantes`, `what` → `que` | `carbon` → `carbono`, `compare` → `comparar`, `deliverable` → `entregable`, `economics` → `economia`, `economy` → `economia`, `losses` → `perdidas`, `measured_vs_simulated` → `medido_vs_simulado`, `monthly` → `mensual` |
| `workspace` | `action` → `accion`, `archive` → `zip`, `destination` → `destino`, `overwrite` → `sobrescribir`, `path` → `ruta`, `project` → `proyecto`, `target` → `destino` | `create` → `crear`, `export_project` → `exportar_proyecto`, `export` → `exportar`, `history` → `historial`, `import` → `importar`, `restore` → `restaurar` |
| `pvsyst_cli` | `args` → `opciones`, `command` → `comando`, `options` → `opciones` | — |
| `gui_observar` | `entity` → `entidad`, `function` → `funcion`, `image` → `imagen`, `lines` → `lineas`, `path` → `ruta`, `process` → `proceso`, `what` → `que`, `window` → `ventana` | `capture` → `capturar`, `controls` → `controles`, `form` → `formulario`, `functions` → `funciones`, `messages` → `mensajes`, `read` → `leer`, `schema` → `esquema`, `tree` → `arbol`, `view` → `vista`, `windows` → `ventanas` |
| `gui_actuar` | `action` → `accion`, `class` → `clase`, `field` → `campo`, `function` → `funcion`, `index` → `indice`, `keys` → `teclas`, `label` → `etiqueta`, `message` → `mensaje`, `name` → `nombre`, `operation` → `operacion`, `process` → `proceso`, `project` → `proyecto`, `role` → `rol`, `text` → `texto`, `title` → `titulo`, `value` → `valor`, `window` → `ventana` | `close_dialog` → `cerrar`, `fill` → `rellenar`, `keys` → `teclas`, `press` → `click`, `select` → `seleccionar`, `type` → `escribir`, `wait` → `esperar`, `write` → `escribir` |
<!-- alias:fin -->

**Resources** (`resources/list` / `resources/read`): `pvsyst://recetas` (task → tool playbook), `pvsyst://cobertura` (coverage map), `pvsyst://formato` (file formats), `pvsyst://cli` (CLI reference), `pvsyst://gui` (GUI function catalogue and window map). Unknown URIs return JSON-RPC -32602 `Recurso desconocido`.

\* The action has an MCP-calculation branch. Its default is PVsyst; `via: "aproximacion"` (and, for the actions that only exist as an approximation, the action itself) requires `PVSYST_MCP_APROXIMACIONES=1`. Without the flag those gate-only actions are hidden from `tools/list`.

All tools read/write only files under the workspace; nothing is sent anywhere, except the PVGIS requests of `meteo_pvgis` and, with approximations enabled, `horizonte_pvgis` and the optional PVGIS fetch of `p50_p90` (PVGIS `MRcalc`, `printhorizon` and `tmy` verified live). Meteonorm data is not used: even the horizon block that `proyecto anadir` copies from a demo is emptied first — see [responsible use](#responsible-use-licensing-and-trademark).

**Path scope.** Every caller-supplied path (read, list, edit, import/export, capture output, CSV/report destinations, history restore) is confined to the workspace: `..` and paths outside it are rejected. Pass `permitir_fuera_del_workspace: true` to opt out explicitly. History restore always validates the whole manifest first and never writes outside the workspace. Edits refuse files whose PVObject structure is unbalanced, and selectors with malformed segments or indices (`pvSubArray[abc]`) are errors instead of falling back to `[0]`.

Snapshots of edited files live under `UserData/mcp_history/<id>/` (with `manifest.json`); simulation cache and trial-budget ledger live under `UserData/mcp_cache/`.


### Performance / latency

Tool execution is fast (a few milliseconds); the perceived slowness is **per-turn
model latency plus the size of tool output**. Two mitigations are built in:

- **`elapsed_ms`** — every tool response carries the server-side duration, so
  latency can be measured instead of guessed.
- **`detalle`** on every tool: `resumen` (default) trims arrays to 10 items and long
  strings; `normal` and `completo` return more. Use `completo` only when needed.
- **`siguiente`** on successful results suggests the next call in the flow.


Other levers: batch independent calls in one turn, avoid long `esperar_segundos`
waits (poll instead), and capture screenshots only when needed.

### Coverage

Because the primitives act on the whole *data + CLI + GUI* surface, the MCP
covers every PVsyst **state, computation and dialog** — projects, variants, all
component types, site/weather, shadings, models, batch runs, parameter overrides,
licensing, logs, and the mouse-driven GUI tools (3D editor, informative plots,
wizards), which are driven through UI Automation and Win32 (`gui_observar` /
`gui_actuar`). Full map: [`docs/cobertura.md`](docs/cobertura.md) and
the page-by-page inventory in [`docs/pvsyst_features.md`](docs/pvsyst_features.md);
the `pvsyst://cobertura` resource returns the machine-readable version.


### "X kWp at X coordinates" — a real flow

1. `sitio {accion:"crear", origen:"coordenadas", latitud, longitud, nombre}` → `.SIT` (reuses an existing site when coordinates match).
2. `proyecto {accion:"crear", plantilla, nombre_nuevo, modulo, inversor, n_mod_serie, n_strings}`.
3. `proyecto {accion:"dimensionar", proyecto, n_mod_serie, n_strings}` → writes `NModSerie`/`NStringCh`/`PNomSizing`.
4. `editar {accion:"campos", proyecto, ediciones:[…]}` → tilt/azimuth, losses, etc.
5. `simular {proyecto, site:"…SIT", synthetic_weather:true}` (trial-licence runs ask for confirmation when they use two or more executions). Check `avisos_pvsyst`: that is where PVsyst warns, e.g., about a Vmpp outside the inverter window.
6. `resultados {que:"perdidas"}`, then `que:"mensual"`, `que:"carbono"`, `que:"entregable"`.


---

## Requirements

* **PVsyst 8.1.7** installed (default `C:\Program Files\PVsyst8.1.7\`).
* **Windows** — the GUI bridge uses the Win32 API.
* **A PVsystCLI licence** for unlimited headless simulation. Without
  `C:\ProgramData\PVsystCLI\PVsystCLI.lic` the CLI runs in **TRIAL mode** (limited days and
  executions; `entorno accion:licencia` shows the counters and the server keeps a ledger,
  asks for confirmation before multi-execution runs and caches repeated simulations). To
  activate, run yourself `"C:\Program Files\PVsyst8.1.7\PVsystCLI.exe" lic-activate -k:<YOUR_KEY>`
  (`entorno accion:licencia_activar` prints the command; the server never executes it).
  Everything else (files, sites, sizing, GUI) works without a licence.
* **Rust** (any recent stable).
* A **linker**: MSVC (`link.exe` from VS Build Tools) — or, as used on the
  machine this was built on, MinGW-w64 + the GNU toolchain:

  ```powershell
  winget install BrechtSanders.WinLibs.POSIX.UCRT      # gcc/ld for the GNU target
  rustup toolchain install stable-x86_64-pc-windows-gnu
  rustup override set stable-x86_64-pc-windows-gnu     # per-project (this dir)
  # make sure <WinLibs>\mingw64\bin is on PATH so cargo finds gcc.exe
  ```

---

## Build from source

```powershell
cargo build --release
# → target\release\pvsyst-mcp.exe
```

Run the tests:

```powershell
cargo test
```

---

## Install into an MCP client

### Claude Desktop (one-click bundle)

Download the `.mcpb` file from the
[latest release](https://github.com/Angelthebestone/PVSYST-MCP/releases/latest) and
open it (or drag it onto *Settings → Extensions*). Claude Desktop asks for the PVsyst
workspace, `PVsystCLI.exe` and `PVsyst.exe`; leave the paths empty to let
the server auto-detect them. The bundle is built from [`mcpb/manifest.json`](mcpb/manifest.json).
The bundle does not update itself: install the new `.mcpb` from each release.

### npm / npx (updates itself)

```json
{
  "mcpServers": {
    "pvsyst": { "command": "npx", "args": ["-y", "pvsyst-mcp"] }
  }
}
```

The npm launcher keeps the server current on its own. Every start runs the newest
binary it has (the packaged one or one downloaded earlier to
`%LOCALAPPDATA%\pvsyst-mcp`) and, once the server is running, checks npm in a
detached background process. A newer release is downloaded for the **next** start; the
current session never switches binaries mid-way. Its log is
`%LOCALAPPDATA%\pvsyst-mcp\actualizacion.log`. Set `PVSYST_MCP_AUTO_ACTUALIZAR=0` to
turn it off. Only npm/npx installs update this way; the zip and the `.mcpb` do not.

### Command Code (Windows)

> **Important — the space-in-path gotcha.** Command Code does not quote the
> executable path when it spawns a stdio MCP server, so a `command` containing a
> space (e.g. `...\PVSYST MCP\...`) **fails to start silently** — the server is
> listed but never connects, and its tools never appear. Install the binary to a
> **space-free** location.

```powershell
# 1. build release and copy it to C:\pvsyst-mcp\pvsyst-mcp.exe
powershell -ExecutionPolicy Bypass -File .\scripts\install.ps1

# 2. register it (binary is cmd / cmdc / command-code; use "cmdc" on Windows)
cmdc mcp add --scope user --env PVSYST_WORKSPACE=C:\Users\Lenovo\PVsyst8.1_Data pvsyst -- "C:\pvsyst-mcp\pvsyst-mcp.exe"
cmdc mcp list
```

`mcp.example.json` shows a ready-to-copy configuration for that space-free binary. `.mcp.json` is ignored, so copy the example and adjust the paths:

```json
{
  "mcpServers": {
    "pvsyst": {
      "transport": "stdio",
      "enabled": true,
      "command": "C:\\pvsyst-mcp\\pvsyst-mcp.exe",
      "env": { "PVSYST_WORKSPACE": "C:\\Users\\Lenovo\\PVsyst8.1_Data" }
    }
  }
}
```

The tools then appear as `mcp__pvsyst__listar_proyectos`, etc. Re-run
`scripts\install.ps1` after every rebuild.

### Other MCP clients

Any client that speaks MCP over **stdio** can drive it (Claude Desktop, Cursor, VS Code,
and anything else that reads the usual `mcpServers` map). Point the client at the
space-free binary and give it the workspace folder:

```json
{
  "mcpServers": {
    "pvsyst": {
      "command": "C:\\pvsyst-mcp\\pvsyst-mcp.exe",
      "env": { "PVSYST_WORKSPACE": "C:\\Users\\Lenovo\\PVsyst8.1_Data" }
    }
  }
}
```

Keep it **local**: the server is designed for the workstation that holds your PVsyst
licence, not for a shared endpoint — see
[responsible use](#responsible-use-licensing-and-trademark).

**Client-compatibility notes.** The server echoes back the `protocolVersion` the
client requests when it supports it (`2024-11-05`, `2025-03-26`, `2025-06-18`;
Command Code asks for `2025-03-26`) and otherwise answers with its latest supported
version, as the MCP spec requires. It exposes five resources (`pvsyst://recetas`,
`pvsyst://cobertura`, `pvsyst://formato`, `pvsyst://cli`, `pvsyst://gui`) and seven prompts built
from those recipes. With `2025-06-18`, tool results also carry `structuredContent`;
long simulations report progress and accept `notifications/cancelled`. Tool
arguments are type-checked against each tool's schema before dispatch.
Verify a live connection with `/mcp` inside a session, or headlessly:
`cmdc -p "usa la tool pvsyst entorno" --tools-all -t --yolo`.


### Environment variables (all optional)

An empty value counts as unset, so the default or auto-detection applies.

| Variable | Meaning | Default |
|---|---|---|
| `PVSYST_WORKSPACE` | User workspace folder | `%USERPROFILE%\PVsyst8.1_Data`, then `Documents\…` |
| `PVSYST_CLI` | Path to `PVsystCLI.exe` | `C:\Program Files\PVsyst8.1.7\PVsystCLI.exe` |
| `PVSYST_GUI` | Path to `PVsyst.exe` | `C:\Program Files\PVsyst8.1.7\PVsyst.exe` |
| `PVSYST_MCP_APROXIMACIONES` | `1` enables the MCP's own calculations (labelled `origen: "aproximacion_mcp"`) | unset (PVsyst results only) |
| `PVSYST_MCP_ESCRITORIO` | `aislado` runs PVsyst and the GUI tools on a private hidden Win32 desktop (the agent never takes your focus, cursor or keyboard); `compartido` uses your desktop. Any other value is an error | `compartido` |
| `PVSYST_MCP_AUTO_ACTUALIZAR` | npm launcher only: `0` disables the auto-update | enabled |
| `PVSYST_MCP_ALMACEN` | npm launcher only: where updates are downloaded | `%LOCALAPPDATA%\pvsyst-mcp` |


> **Note on the workspace path.** PVsyst 8 stores user data in
> `%USERPROFILE%\PVsyst8.1_Data` (created on first run), not in
> `Documents\PVsyst8.1`. The MCP auto-detects the real workspace and also
> accepts an explicit `workspace` argument on every tool. That argument moves the
> confinement root itself (every path check is relative to it), so it is a
> deliberate capability of this local server: only connect MCP clients you trust.

---

## Usage examples

```
listar                 { "que": "modulos", "texto": "CX-M500" }
leer                   { "archivo": "CX-M500_2450Wp_1_00_086.PAN" }
→ { "format": "legacy-real48", "vendor": "Soitec", "pnom": 2450.0,
    "voc": 695.7, "isc": 3.861, ... }

disenar_sistema         { "latitud": 7.07, "longitud": -73.84, "kwp": 10,
                          "n_mod_serie": 10, "n_strings": 3 }
→ { "kpis": { "energia_producida_kwh": 15517, "specific_yield_kwh_per_kwp": 1567, "pr": 83.0 },
    "site": { "weather_source": "NASA SSE 6.0 grid cell 7/-74 ..." }, "pasos": [ ... ] }

proyecto                { "accion": "crear", "plantilla": "_DEMO_Commercial",
                          "nombre_nuevo": "MY_VARIANT",
                          "modulo": "CX-M500_2450Wp_1_00_086.PAN" }

simular                 { "proyecto": "MY_VARIANT", "variante": "VC0",
                          "output_csv": "C:\\...\\Results\\out.csv",
                          "report_pdf": "C:\\...\\Results\\out.pdf" }
→ { "success": true, "stdout_tail": "... Simulation done in 0min 3sec ...", "costo": { ... } }

resultados              { "que": "kpis" }

```

---

## File-format documentation

`docs/pvsyst_format.md` documents every file type observed from the outside,
including the recovered layout of the legacy binary `.PAN` (Delphi `Real48`,
50 × 6-byte slots) with the decoded field table. A standalone Python reference
decoder lives in `tools/pan_decode.py`.

`PVSYST_CLI_REFERENCE.md` is the exhaustive, verified reference for
`PVsystCLI.exe` (commands, exact option grammar, batch mode, success detection).

---

## Verification (evidence)

* Tests: `cargo test` runs the unit tests and the end-to-end smoke tests in `tests/mcp_smoke.rs`, which drive the compiled server over stdio exactly like a client. Real-PVsyst tests skip when PVsyst is absent.
* `leer` decodes the bundled legacy `CX-M500` module to **PNom = 2450 W** (exact), Voc = 695.7 V, Isc = 3.861 A.
* **Full coordinates → capacity → simulation → results run** (all via MCP tools, recorded on the 15-tool surface before consolidation): the site from coordinates (lat 46.2044, lon 6.1432) was created; `proyecto crear` built a ~507 kWp variant; `simular` ran it at the new site with synthetic weather → **`Simulation done in 0min 2sec`** + PDF generated; `resultados que:csv` parsed the 330 KB CSV (8 760 hourly rows, `date;GlobInc;GlobEff;EArray;E_Grid;PR`).
* `editar` (fields), `proyecto dimensionar` and `workspace restaurar` are covered by smoke tests; history snapshots are checked via `UserData/mcp_history/<id>/manifest.json`.
* PVsyst-side validation errors are surfaced (`success:false`), e.g. an over-sized array rejected by PVsyst with "array Voc … greater than the inverter absolute maximum input voltage".
* **GUI automation verified against the live app**: `gui_observar que:ventanas|controles` enumerated PVsyst's real VCL windows/controls; `gui_actuar accion:click` on the *Herramientas* button opened `TToolsForm`; `gui_observar que:capturar` produced a screenshot; `gui_actuar accion:cerrar` closed it. Covered by the gated test `gui_invokes_button_and_opens_window`.

Every table, figure and report quoted above was produced with **PVsyst 8.1.7
(PVsystCLI)**; see [citation](#how-to-cite-this-work).


## Layout

One folder per PVsyst business area; dependencies point inward to `format/`
(pure core, imports nothing from the crate). `server/` (inbound) and
`adapters/` (outbound) are the edges. `tools/check_deps.py` fails CI on any
cycle; the layering, bottom-up, is:

`error` → `format`, `state` → `config` → `args` → `history` → `workspace` →
`adapters` → feature modules (`components`, `objects`, `projects`, `sites`,
`simulation`, `admin`, `prefs`, `easy_mode`) → `server`

Every production source file in `src/` stays under 300 lines; large areas are directory
modules split by concept. The two older end-to-end test suites (`tests/mcp_smoke.rs` and
`tests/mcp_regresiones.rs`) predate that limit.

```
src/
  main.rs              stdio entry point
  config.rs            workspace / CLI / GUI path resolution
  state/               per-server registries (background jobs, UIA cache) and their records
  args.rs              tool-argument extraction (aliases, fail-fast errors)
  server/              inbound adapter: MCP over stdio
    mod.rs             JSON-RPC / MCP protocol (newline-delimited, version negotiation)
    tools/             one declaration per tool: schema + validation + handler (by family)
  format/              core: PVsyst file formats (no I/O)
    object.rs          generic PVObject text parser (BOM-safe) + structural diagnostic
    selector.rs        strict object selector parser (`pvVCalcul/pvSubArray[1]`)
    span.rs            byte-preserving object span scanner
    edit.rs / insert.rs / delete.rs   field edits, object insert/extract, object removal
  adapters/            outbound adapters to the PVsyst executables
    cli/               PVsystCLI runner: capped capture, typed classification, background jobs
    gui/               Win32 UI automation (windows, actions, waits, GDI capture with RAII, PNG)
    uia/               UI Automation tree and element operations
  history/             snapshots, validated restore, atomic writes (unique temp, per-path lock)
  workspace/           path scope (`resolver_ruta`), bounded reads, guarded listing, writes
  objects/             leer_objeto / listar_objetos / editar_objeto
  projects/            listing, lifecycle/, doctor/, anadir/, temporales
  components/          all component kinds; pan/ (Real48, strict legacy decoder), ficha/
  sites/               sites, readers, NASA grids, solar/, pvgis/ (staged, transactional)
  simulation/          preparation, execution, state, batch, cache/, results/, report_pdf/, engineering/
  admin/               workspace/project/component import-export, licence, logs
  prefs/               PVsyst.ini preferences and CLI log diagnostics
  easy_mode/           disenar_sistema: typed per-step pipeline, diagnostico, leer_kpis
tests/mcp_smoke.rs     end-to-end protocol tests
docs/pvsyst_format.md    file formats (observed from outside)
docs/cobertura.md        coverage map (area -> tool, GUI-only boundary)
docs/pvsyst_features.md  full 623-page feature inventory
tools/pan_decode.py      reference decoder
```

---

## FAQ

### Can I run PVsyst simulations without opening the GUI?

Yes. `simular` prepares the variant, shells out to the documented
`PVsystCLI run-simulation` command, parses the success markers and returns the KPIs —
no window is opened. The GUI tools (`gui_observar`, `gui_actuar`) exist for the dialogs
that have no file or CLI equivalent; they are available by default.

### Do I need a PVsyst licence?

For headless simulation, yes. `PVsystCLI` runs in TRIAL mode without a licence
(evaluation limits apply, see [responsible use](#responsible-use-licensing-and-trademark));
the server tracks the remaining budget in a ledger, asks for confirmation before a run
that would spend several executions, and caches repeated simulations. File-level work —
listing, reading, editing, sites, sizing, component creation — needs no licence at all.

### Which MCP clients work with it?

Anything that speaks MCP over stdio and reads the usual `mcpServers` map: Command Code,
Claude Desktop, Cursor, VS Code, and others. See
[install into an MCP client](#install-into-an-mcp-client). The server opens no network
port.

### Is this affiliated with PVsyst SA?

No. It is an unofficial interoperability project: no PVsyst code, no decompilation, no
redistribution of PVsyst software, and no licence management. `PVsyst.exe` is never
opened in a disassembler; `PVsystCLI.exe` is only run.

### Will it corrupt my projects?

It edits only what you ask, at byte level, and refuses files whose PVObject structure
is unbalanced. Every write is preceded by a snapshot under `UserData/mcp_history/<id>/`,
and `workspace accion:restaurar` validates the whole manifest before writing anything
back. Note that PVsyst itself documents its file format as internal and unsupported, so
snapshots are the safety net, not PVsyst support.

### Does it send my data anywhere?

No. The only outbound requests are the PVGIS calls you trigger: `meteo_pvgis` and, with
`PVSYST_MCP_APROXIMACIONES=1`, `horizonte_pvgis` and the optional fetch of
`resultados que:p50_p90`. Nothing is uploaded, and no telemetry is collected.

### Can it do batch runs and parameter sweeps?

Yes: `simular accion:ejecutar` with `filas` runs a PVsyst batch (`-bpf`, one row per
orientation or configuration, each counting as one execution), `simular accion:preparar` writes
batch and parameter files, and long runs can be started
as background jobs polled with `simular accion:estado`.

### Can it read the old binary `.PAN` modules?

Yes. The legacy Delphi `Real48` layout was recovered from the outside and is decoded by
`components/pan/`; `tools/pan_decode.py` is a standalone reference decoder you can run
without the server.

### Which PVsyst versions and file types are supported?

Defaults target PVsyst 8.1.7, with the executable and workspace paths configurable
through `PVSYST_CLI`, `PVSYST_GUI` and `PVSYST_WORKSPACE`. Supported artifacts include
`.PRJ` / `.VCi` projects and variants, `.PAN` (text and legacy binary) and `.OND`
components, `.SIT` sites, `.MET` weather files, plus raw PVObject text.

### Why not use a PVsyst API instead?

There is no public API to use. This project deliberately takes the two supported routes
— the documented `PVsystCLI` commands and the data files — instead of reimplementing a
simulation engine, which would give you different numbers than PVsyst.

---

## Responsible use, licensing and trademark

This is an unofficial tool that drives **your** installation of PVsyst, so the terms
that apply to the program are the ones you accepted when you installed it: PVsyst SA's
[*General Conditions of Use*](https://www.pvsyst.com/pdf/common/general-conditions-of-use-english.pdf)
and your subscription contract. How this project positions itself against them:

* **No decompilation, no modification of the programs.** The GCU forbids reproducing,
  modifying, translating, decompiling or recompiling the Software. This project never
  touches `PVsyst.exe`/`PVsystCLI.exe`; it reads and writes *your* data files and runs
  the documented CLI.
* **Keep it local.** The GCU forbids making the Software accessible to third parties and
  requires you to prevent unauthorised access to it, with one licence per workstation.
  Do not expose this server as a shared or remote endpoint: it is meant for the machine
  that holds the licence, and its `workspace` argument plus `permitir_fuera_del_workspace`
  give any connected client real filesystem reach.
* **GUI automation stays on the workstation.** PVsyst 8 must not be used on a server or
  operated through a Windows Remote Desktop client; `PVsystCLI` explicitly may. That maps
  to this project's surface: the CLI-based tools are fine on a server, the GUI tools
  `gui_observar` / `gui_actuar` are not.
* **Never touch the clock.** Licences depend on the machine's date and time never having
  been amended; nothing in this project reads or changes it.
* **No Meteonorm data leaves PVsyst.** The GCU prohibits exporting Meteonorm data to
  other software, so the MCP does not use it: sites come from coordinates (NASA SSE) or
  PVGIS TMY, and the Meteonorm-derived horizon block of a demo project is emptied before
  it is copied.
* **Trial and evaluation limits are respected, not bypassed.** The server only *counts*
  the remaining trial executions to warn you before spending them, and
  `entorno accion:licencia_activar` merely prints the activation command for you to run
  with your own key. The licence file, the registry and the system clock are never
  written to.
* **Editing files is your call.** PVsyst documents its file format as internal-use and
  unsupported; editing is legal — they are your project files — but PVsyst will not
  support a file this project (or you) broke. Validate with `proyecto accion:doctor` and
  keep the snapshots.

### How to cite this work

Clause 8.4 of the GCU requires quoting PVsyst whenever you publish images or tables
generated by it. Every result quoted in this repository — including the report fixture
in `tests/fixtures/results/` — was generated with:

> PVsyst 8.1.7 (PVsystCLI), PVsyst SA, Satigny, Switzerland — https://www.pvsyst.com

### This program's licence

`pvsyst-mcp` itself is licensed under the **Apache License 2.0** ([`LICENSE`](LICENSE)),
with the attribution notice in [`NOTICE`](NOTICE). If you fork it or redistribute a
modified version you must keep the attribution notices and mark the files you changed
(sections 4(b)–4(d)), and section 6 grants no rights to the project name. Contributions
are accepted under the same licence — see [CONTRIBUTING.md](CONTRIBUTING.md).

## Contributing

Issues and pull requests are welcome. Start with [CONTRIBUTING.md](CONTRIBUTING.md) for
the ground rules (no vendor data in the repository, no decompilation, behaviour-based
tests), the [Code of Conduct](CODE_OF_CONDUCT.md), and [SECURITY.md](SECURITY.md) to
report a vulnerability privately. Changes are tracked in [CHANGELOG.md](CHANGELOG.md).

## Development checks

```powershell
cargo fmt --check
cargo clippy --all-targets -- -D warnings   # lints (unwrap/panic/unsafe docs/complexity/...) live in Cargo.toml + clippy.toml
cargo test                                  # hermetic: synthetic workspace in tests/fixtures/workspace, no PVsyst needed
cargo test -- --ignored                     # also run the tests that need the PVsyst 8.1.7 demo data / a running GUI
cargo audit; cargo deny check               # advisories, licences, bans, sources (deny.toml)
```

The crate is a library (`pvsyst_mcp`: `Paths`, `Server`, `serve`) plus a thin
`pvsyst-mcp` binary. Errors are typed (`src/error.rs`); logs go to stderr and are
filtered with `PVSYST_MCP_LOG` (default `info`) because stdout carries the protocol.
