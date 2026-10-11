# PVsyst MCP (Rust)

A [Model Context Protocol](https://modelcontextprotocol.io) server that lets an
agent **operate a PVsyst 8.1.7 installation by reading and writing ONLY its data
files** — never by touching `PVsyst.exe`, and without reimplementing a single
line of PVsyst's simulation logic.

> **Hard rules this project follows**
> * No PVsyst source code, no decompilation. `PVsyst.exe` is never opened in
>   Ghidra/IDA; `PVsystCLI.exe` is only ever *run* black-box.
> * No simulation engine is implemented here. When a simulation is needed the
>   MCP shells out to the **documented** `PVsystCLI run-simulation` command.
> * The MCP prepares, duplicates and inspects projects; PVsyst does the physics.

---

## What it exposes (15 tools; 12 by default)

Each tool takes a discriminator (`accion` or `que`); omitted values use the default shown.
The default surface has 12 tools. `pvsyst_cli`, `gui_observar` and `gui_actuar` appear only when `PVSYST_MCP_EXPERTO=1`.

| Tool | Discriminator (default) | Purpose |
|---|---|---|
| `entorno` | `accion`: `diagnostico` (default), `licencia`, `licencia_sync`, `licencia_activar`, `licencia_desactivar`, `exportar_logs`, `logs_cli`, `preferencias`, `preferencias_editar` | Environment, licence, logs, PVsyst preferences. `licencia_activar`/`licencia_desactivar` never execute; they return the command template. |
| `disenar_sistema` | — | Coordinates + kWp → site, project, sizing, simulation, KPIs in one call. |
| `listar` | `que`: `proyectos` (default), `componentes`, `modulos`, `archivos`, `objetos` | Lists projects, components, modules (with `pnom_min`/`pnom_max`/`texto` filters), files, objects. |
| `leer` | type detected from `archivo` | Reads a project, module (`.PAN`), site, meteo, component, PVObject file, or raw file bytes. |
| `editar` | `accion`: `campos` (default), `insertar_objeto`, `eliminar_objeto` | Edits fields of any object (byte-preserving, snapshot first) or inserts/removes objects. |
| `escribir_archivo` | — | Writes a workspace file (snapshot first; path-guarded). |
| `proyecto` | `accion`: `crear`, `eliminar`, `renombrar`, `crear_variante`, `dimensionar`, `doctor`, `anadir`, `limpiar_temporales` | Project and variant lifecycle, consistency checks and repairs, demo-object insertion, temp cleanup. |
| `componente` | `accion`: `crear`, `borrar`, `importar`, `exportar`, `crear_desde_ficha` | Component lifecycle; builds OND/PAN from datasheet values. |
| `sitio` | `accion`: `crear` (`origen`: `coordenadas` (default) or `csv`), `convertir_meteo`, `geometria_solar`, `horizonte_pvgis`, `meteo_pvgis` | Sites, weather conversion, solar geometry, PVGIS horizon and TMY hourly weather (endpoints verified live). |
| `simular` | `accion`: `ejecutar` (default), `preparar`, `estado`, `preestimar`, `barrido` | Runs simulations (CSV and PDF report default to `Results/<proyecto>_<variante>.csv` and `.pdf`; cache, trial ledger, background jobs), prepares batch files, local pre-estimate and sweep. |
| `resultados` | `que`: `kpis` (default), `csv`, `perdidas`, `mensual`, `comparar`, `p50_p90`, `carbono`, `economia`, `entregable` | Results from the CSV and the PDF report (losses, monthly table), P50/P90, carbon, economics, Markdown deliverable. |
| `workspace` | `accion`: `crear`, `exportar`, `importar`, `exportar_proyecto`, `historial`, `restaurar` | Workspace admin and history (snapshots) restore. |
| `pvsyst_cli` | — (expert) | Runs any PVsystCLI command with raw options. |
| `gui_observar` | `que`: `ventanas` (default), `controles`, `leer`, `capturar`, `uia`, `arbol` (expert) | Observes PVsyst windows, controls, UI Automation tree; captures PNG. |
| `gui_actuar` | `accion`: `click`, `clic_fisico`, `mensaje`, `cerrar`, `activar`, `teclas`, `lanzar`, `uia_invocar`, `esperar`, `paso` (expert) | Drives PVsyst windows and controls. |

**Resources** (`resources/list` / `resources/read`): `pvsyst://recetas` (task → tool playbook), `pvsyst://cobertura` (coverage map), `pvsyst://formato` (file formats), `pvsyst://cli` (CLI reference). Unknown URIs return JSON-RPC -32602 `Recurso desconocido`.

All tools read/write only files under the workspace; nothing is sent anywhere, except the PVGIS requests of `horizonte_pvgis` and `meteo_pvgis` and the optional PVGIS fetch of `p50_p90` (PVGIS `MRcalc`, `printhorizon` and `tmy` verified live). Meteonorm data is not used: even the horizon block that `proyecto anadir` copies from a demo is emptied first.

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
`gui_actuar`, expert mode). Full map: [`docs/cobertura.md`](docs/cobertura.md) and
the page-by-page inventory in [`docs/pvsyst_features.md`](docs/pvsyst_features.md);
the `pvsyst://cobertura` resource returns the machine-readable version.


### "X kWp at X coordinates" — a real flow

1. `sitio {accion:"crear", origen:"coordenadas", latitud, longitud, nombre}` → `.SIT` (reuses an existing site when coordinates match).
2. `proyecto {accion:"crear", plantilla, nombre_nuevo, modulo, inversor, n_mod_serie, n_strings}`.
3. `proyecto {accion:"dimensionar", proyecto, kwp}` → computes and writes `NModSerie`/`NStringCh`/`PNomSizing`.
4. `editar {accion:"campos", proyecto, ediciones:[…]}` → tilt/azimuth, losses, etc.
5. `simular {proyecto, site:"…SIT", synthetic_weather:true}` (trial-licence runs ask for confirmation when they use two or more executions).
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

## Build

```powershell
cargo build --release
# → target\release\pvsyst-mcp.exe
```

Run the tests:

```powershell
cargo test
```

---

## Install into Command Code (Windows)

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

`.mcp.json` (checked in) points at that space-free path:

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

**Client-compatibility notes.** The server echoes back the `protocolVersion` the
client requests when it supports it (`2024-11-05`, `2025-03-26`; Command Code asks
for `2025-03-26`) and otherwise answers with its latest supported version, as the MCP
spec requires. It exposes four resources (`pvsyst://recetas`, `pvsyst://cobertura`,
`pvsyst://formato`, `pvsyst://cli`) plus an empty `prompts/list`, so it connects cleanly.
Tool arguments are type-checked against each tool's schema before dispatch.
Verify a live connection with `/mcp` inside a session, or headlessly:
`cmdc -p "usa la tool pvsyst entorno" --tools-all -t --yolo`.


### Environment variables (all optional)

| Variable | Meaning | Default |
|---|---|---|
| `PVSYST_WORKSPACE` | User workspace folder | `%USERPROFILE%\PVsyst8.1_Data`, then `Documents\…` |
| `PVSYST_CLI` | Path to `PVsystCLI.exe` | `C:\Program Files\PVsyst8.1.7\PVsystCLI.exe` |
| `PVSYST_GUI` | Path to `PVsyst.exe` | `C:\Program Files\PVsyst8.1.7\PVsyst.exe` |
| `PVSYST_MCP_EXPERTO` | `1` exposes `pvsyst_cli`, `gui_observar` and `gui_actuar` | unset (12 tools) |


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

disenar_sistema         { "latitud": 7.07, "longitud": -73.84, "kwp": 10 }
→ { "kpis": { "e_grid_kwh": 15517, "specific_yield_kwh_per_kwp": 1567, "pr": 0.83 },
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
* **GUI automation verified against the live app** (expert mode): `gui_observar que:ventanas|controles` enumerated PVsyst's real VCL windows/controls; `gui_actuar accion:click` on the *Herramientas* button opened `TToolsForm`; `gui_observar que:capturar` produced a screenshot; `gui_actuar accion:cerrar` closed it. Covered by the gated test `gui_invokes_button_and_opens_window`.


## Layout

One folder per PVsyst business area; dependencies point inward to `format/`
(pure core, imports nothing from the crate). `server/` (inbound) and
`adapters/` (outbound) are the edges. The module graph is acyclic, enforced
in CI by `tools/check_deps.py`:

`error → format, history, state → config → args → adapters / workspace → sites,
components, projects, simulation → objects, admin, easy_mode → server`

Every file stays under 300 lines; large areas are directory modules split by concept.

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