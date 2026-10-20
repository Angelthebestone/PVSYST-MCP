# PVsyst MCP: hybrid GUI and roadmap

> Status: **implemented** in the 15-tool surface (12 by default; expert tools with `PVSYST_MCP_EXPERTO=1`), except where marked otherwise in the README. The Meteonorm data source is excluded by design (PVsyst conditions of use §1.3). `configurar_analitica` is not implemented: CLI analytics are disabled in `PVsystCLI.ini`. The PVGIS endpoints (`MRcalc`, `printhorizon`, `tmy`) were verified with live calls. PVsystCLI cannot simulate measured-data variants (`CMx`): those stay GUI-only.
>
> Guiding idea: two resources are scarce, **trial executions** (231 → 223 after
> one test round) and **tokens**. Two are slow: PVsystCLI (~5–6 s per call) and
> the GUI (~16 s to start). The budget ledger, simulation cache and background jobs now manage them.

## 1. GUI: hybrid model (Win32 + UI Automation + vision)

Goal: drive every PVsyst screen with as few screenshots as possible.

### Today and its limits

`gui_*` uses only Win32 messages:

- **Listing:** `EnumWindows` for top-level windows and `EnumChildWindows` for
  their controls.
- **Actions:** `BM_CLICK`, `WM_*` messages, keystrokes and `clic_fisico`.
- **Reading:** `GetWindowText`.

This reaches standard VCL buttons (*Herramientas*, *Bases de datos*), but not:

- **items without their own hwnd:** grid, tree, list and tab items, and the home
  screen. On `THomeForm`, `gui_controles` returns 0 controls;
- **drawn content:** the 3D editor, loss diagram, I-V curves and plots;
- **HTML panes:** home tiles and help.

It is also stateless. Forms are created lazily, so the caller has to
re-enumerate after every step.

Other findings from testing:

- **Startup:** PVsyst takes ~16 s to start. `gui_lanzar` now polls for a
  visible `THomeForm` (budget 30 s, `ready` field in the result) instead of
  sleeping a fixed 8 s.
- **Hidden windows:** about half the top-level windows are hidden helpers.
  `gui_ventanas` now lists only visible windows by default, cutting its output
  from ~550 to ~90 tokens.
- **To retest:** `gui_leer` and `gui_capturar` were only tested before the
  startup fix, and showed:
  - an empty class and title in the capture result;
  - `gui_leer THomeForm` returned the text "Preferences";
  - two different hwnds for `THomeForm` in the same session.

  If the hwnd really changes, resolve `ventana` once per step and reuse it.
- **Screenshots:** a capture is a 1.6 MB BMP. If vision is added, save PNG and
  crop to the target rect. Capturing is only needed for content with neither
  text nor a handle.

### Proposal: `observe → act → verify` with three providers

| Layer | Strong at | Weak at |
|---|---|---|
| Win32 (today) | VCL buttons, close/activate, keys; cheap | items without an hwnd, drawn content, HTML |
| UI Automation | name/role/patterns (`Invoke`, `Value`, `Toggle`, `Selection`, `ExpandCollapse`) for menus, tabs, combos, grids, trees | VCL exposes little; graphics don't exist in the tree |
| Screenshot + vision | anything drawn; clicks by coordinates | needs an interactive desktop; costs tokens; fragile |

- **`act(target)` cascade:** UIA by name/role → Win32 by class/title →
  `clic_fisico` from a screenshot, with `gui_teclas` as the universal fallback.
- **`observe()`:** a single merged Win32 + UIA tree with
  `name/type/patterns/hwnd/rect` per node, flagging the containers that only
  vision can handle.
- **Waiting:** driven by UIA events, not polling.

New tools (`gui_uia_*`, `gui_esperar_ventana`, `gui_paso`) fit into the two GUI
tools of section 3 instead of adding more.

### Risks

- **PVsyst is Delphi/VCL:** UIA will expose little, so the backbone remains
  Win32 + keys.
- **Speed:** walking the UIA tree is slow; cache per hwnd and fetch only
  subtrees.
- **No interactive desktop:** `PrintWindow` returns black and coordinate clicks
  fail. Detect this and degrade to UIA + Win32.

### Phases

1. UIA search/invoke + `incluir_uia` in the listing + event-based waiting.
2. Unified `observe()` and the `gui_paso` cascade.
3. Vision fallback with desktop detection.

## 2. PVsyst coverage matrix

The generic primitives (`leer_objeto`, `editar_objeto`, `pvsyst_cli`, `gui_*`)
can reach any field in theory, which is why `cobertura` claims full coverage. In
practice there are three limits:

1. **The CLI only** simulates, creates sites, converts weather, manages the
   licence and exports logs.
2. **`editar_objeto` only edits objects that already exist.** It cannot add a
   storage system, horizon, load or second sub-array missing from the template.
3. **GUI automation stops at standard buttons** (section 1).

The **"Demo object"** column comes from the bundled demos in
`DataRO\PVsyst8.1_Data\Projects`: `_DEMO_Commercial`, `_DEMO_DCGrid`,
`_DEMO_N13_Motorway_Data`, `_DEMO_Pumping`, `_DEMO_Residential`,
`_DEMO_StandAlone`, `_DEMO_Tracking` and `_DEMO_Utility`. The area list is from
general PVsyst knowledge and has not been checked against the 8.1 manual.

| PVsyst area | MCP today | Demo object | Path to cover it |
|---|---|---|---|
| Grid-connected, fixed plane | ✅ end to end | `pvOrient` (`FieldType=FixedPlane`) | — |
| Trackers / domes | ⚠️ only by editing a template that has it | `_DEMO_Tracking`; `pvDomeOrient` | files: copy the orientation object |
| Bifacial | ⚠️ manual field edits | `pvBiFacial`, `pvBiFacialSheds`, `pvBiFacialTrackers` | files |
| Stand-alone systems | ❌ | `_DEMO_StandAlone`; `pvLoad`; `Models\*.LOD` | files + templates |
| Pumping systems | ❌ | `_DEMO_Pumping`; `pvHydrCircuit` | files + templates |
| DC grid | ❌ | `_DEMO_DCGrid` | files + templates |
| Storage in grid-connected systems | ❌ | `pvGridStorage` | files: `añadir_baterias` |
| Horizon (far shading) | ⚠️ manual `.HOR` editing | `pvHorizon`; `Shadings\DEMO_Geneve.HOR` | own calculation (DEM / PVGIS) + files |
| Near shading 3D (scene, objects) | ❌ GUI only | `pvShading`, `pvShdFTable`, `pvShdArrayTable`, `pvShdFieldZone`; `Shadings\*.SHD/.BLD/.SHO` | hybrid GUI; read existing tables from files |
| Module layout | ❌ | `pvModuleLayout` | hybrid GUI |
| Detailed losses (wiring, mismatch, soiling, unavailability, grid limit) | ⚠️ editable fields, no guided flow | fields in `pvSubArray` / `pvVCalcul` | glossary + guided tool |
| Ageing | ❌ | `pvAgeing` | files |
| Carbon balance | ❌ | `pvCarbonBalance` | files / own calculation |
| Economic evaluation | ❌ | — (not in the demos) | own calculation (LCOE, payback) |
| P50/P90 | ❌ | — | own calculation (multi-year weather) |
| Optimisation / parametric | ⚠️ batch built by hand | batch CSV (`-bpf`) | local pre-estimate + batch |
| Weather import (PVGIS, Meteonorm, measured) | ⚠️ NASA monthly synthetic, or `convert-meteo` with a `.MEF` | `Meteo\*.MEF` | own download + `convert-meteo` |
| Component database | ⚠️ 2 modules; no online search or datasheet import | `ComposPV\*` | create `.PAN`/`.OND` from datasheets |
| Measured data comparison | ❌ | `_DEMO_N13_Motorway_Data` (`.CM0`, `pvVCalculMes`); `DataMeas\` | files + CLI (if it supports `.CMi`) |
| Results: loss diagram, report, plots | ⚠️ CSV totals + KPIs; PDF generated but not read | `pvSimulPlot`; `-isf` variable selection | loss variables via `-isf` |
| *Tools* menu (solar geometry, transposition, array electrical behaviour…) | ❌ | — | own calculation or hybrid GUI |

### How to cover the gaps

- **Files (most robust).** Learn what PVsyst writes by diffing variants:
  - compare a demo variant that has the feature with the same project without
    it (e.g. `_DEMO_Commercial.VC8` has `pvAgeing` and `pvBiFacial`);
  - or save a feature from the GUI and diff before and after.

  The MCP then **inserts whole objects** copied from a demo, with a new
  `insertar_objeto` primitive and tools built on it (`añadir_baterias`,
  `añadir_horizonte`, `añadir_carga`). This unlocks most of the table.
- **Hybrid GUI (section 1)** for what only exists on screen: the 3D scene,
  module layout and the graphical tools.
- **Own calculation** for what doesn't need PVsyst: horizon from a DEM, P50/P90
  from multi-year weather, economics, carbon balance.

## 3. Tool consolidation: 53 → 15

| New tool | Absorbs |
|---|---|
| `entorno` | `preflight`, `info_entorno`, `licencia`, `exportar_logs` |
| `disenar_sistema` | unchanged |
| `listar` (`que: proyectos\|componentes\|archivos\|objetos`) | `listar_proyectos`, `listar_componentes`, `listar_modulos`, `listar_archivos`, `listar_objetos` |
| `leer` (type detected from the extension) | `leer_proyecto`, `leer_objeto`, `leer_componente`, `leer_modulo`, `leer_sitio`, `leer_meteo`, `leer_archivo` |
| `editar` | `editar_objeto`, `editar_variante` |
| `escribir_archivo` | unchanged |
| `proyecto` (`accion`) | `crear_proyecto`, `eliminar_proyecto`, `renombrar_proyecto`, `crear_variante`, `dimensionar_variante` |
| `componente` (`accion`) | `crear_componente`, `borrar_componente`, `importar_componente`, `exportar_componente` |
| `sitio` (`origen: coordenadas\|csv`, plus weather conversion) | `crear_sitio_desde_coordenadas`, `crear_sitio`, `convertir_meteo` |
| `simular` (batch rows and hidden parameters as arguments) | `lanzar_simulacion`, `crear_batch`, `modificar_parametros` |
| `resultados` (`detalle`) | `leer_resultados_csv`, `leer_kpis` |
| `workspace` (`accion`) | `crear_workspace`, `exportar_workspace`, `importar_workspace`, `exportar_proyecto` |
| `pvsyst_cli` | expert mode only |
| `gui_observar` | `gui_ventanas`, `gui_controles`, `gui_leer`, `gui_capturar` |
| `gui_actuar` | `gui_invocar`, `gui_activar`, `gui_cerrar`, `gui_teclas`, `gui_lanzar` |

Related changes:

- **Docs become MCP resources.** `recetas`, `cobertura` and the format docs cost
  nothing until read (today ~2k tokens of the catalogue).
- **Expert mode.** The GUI tools and `pvsyst_cli` only appear when an env flag
  is set, so the model normally sees **12 tools**.
- **Licence activation out of the model's reach.** Today the licence key passes
  through the chat. Return the command for the user to run instead.

Caveats:

- **Savings are not proportional.** An `accion` tool carries the parameters of
  every tool it absorbs. Estimate: catalogue from ~20 KB to ~10–11 KB (~50 %).
  Measure with `.scratch/probe.py`.
- **More parameter mistakes.** JSON Schema doesn't express rules such as
  "`clave` is required only if `accion=activate`" well. Validate in the server
  with clear errors and keep parameter names consistent.
- **Nothing breaks.** The old names stay as hidden aliases in the dispatcher.
  Clients and smoke tests keep working.
- **Moderate effort.** The domain code doesn't change. The work is:
  - `server/tools.rs`: schemas and routing by `accion`;
  - `server/mod.rs`: `resources/list` and `resources/read`;
  - `README.md` and `recetas`: update the docs.

Phases:

1. **Risk-free merges:** `listar`, `leer`, `entorno`, the two GUI tools and the
   resources, giving ~30 tools.
2. **Action merges:** `proyecto`, `componente`, `workspace`, `sitio` and
   `simular`, to reach 15.

After each phase, measure the catalogue tokens and check
`disenar_sistema` → `resultados`.

## 4. Scarce resources: trial and time

- **Simulation cache.** Key = hash of (variant + site + weather + options). An
  identical re-run uses neither an execution nor ~30 s.
- **`.SIT` written directly, or reused for the same coordinates**, instead of
  calling `create-site`. Saves ~6 s and one execution, so `disenar_sistema`
  costs 1 execution instead of 2.
- **Budget ledger.** Every tool declares its cost ("uses 1 of 223").
  Expensive operations return a plan first and run on confirmation.
- **Local pre-estimate.** The NASA grids are already parsed. A simple
  PVWatts-like model gives instant tilt/azimuth sweeps; PVsyst then simulates
  only the 2–3 best candidates, in one batch.
- **Background jobs.** Batches return an id to poll, with progress
  notifications, instead of blocking.

## 5. Engineering value

- **Project "doctor" before simulating, with automatic repair.** It checks:
  - duplicate keys;
  - missing PAN/OND files;
  - a variant site that differs from the simulated one;
  - stale comments;
  - electrical limits: Voc at minimum temperature × series vs the inverter's
    maximum voltage, the MPPT window and the DC/AC ratio.

  First targets: `Barrancabermeja_10kW`, `E2E_Demo` and `MCP_E2E`, which have
  contradictory `FieldTilt`/`FieldAzim` duplicates from the `pvOrient` bug fixed
  on 2026-10-11.
- **History and undo.** One snapshot folder replaces the loose `.bak` files,
  which renaming and deleting currently leave behind. Multi-field edits are
  atomic.
- **Variant comparison:** changed parameters plus the change in energy, yield,
  PR and losses.
- **Loss tree as the main output.** Request the loss variables via `-isf` and
  return "−3.1 % temperature, −2 % wiring…" in ~200 tokens instead of 8760
  hourly rows.
- **Better inputs:**
  - real hourly weather (PVGIS / NASA POWER TMY) instead of values synthesised
    from monthly means;
  - `.PAN`/`.OND` built from datasheets. The database has only 2 modules today.
- **`disenar_sistema` fixes:**
  - write the site into the project; today a re-run without `site`, or the GUI,
    uses Geneva;
  - search over (series, strings) to get closer to the target; today 10 kWp gave
    9 kWp because the series count is fixed.
- **Prefixed test projects** (`mcp_tmp_*`) with automatic cleanup, since the
  CLI only accepts the real workspace.
- **Client deliverable:** KPIs, losses, the PVsyst PDF, LCOE and payback.

## 6. Responses and quality

- **Response size:** `detalle: resumen|normal|completo` on every tool, `resumen`
  by default.
- **Next step:** every response suggests the next action, so the model doesn't
  need `recetas`.
- **Actionable errors:** "module X not found; candidates: A, B; or pass
  `pnom_modulo`" instead of "os error 2".
- **Fake PVsystCLI with recorded outputs**, so CI covers the CLI paths.
- **Tests on real PVsyst files**, plus a parse → edit → re-parse round trip
  with untouched bytes identical.

## 7. Priority

| # | Improvement | Impact | Effort |
|---|---|---|---|
| 1 | Doctor + repair of the 3 projects | Avoids wrong results and wasted executions | Medium |
| 2 | Simulation cache + `.SIT` without the CLI | Half the executions, ~6 s less per design | Low |
| 3 | Losses and variant comparison | The engineering value that is missing today | Medium |
| 4 | 53 → 15 tools + resources | ~3k fewer tokens per session | Medium |
| 5 | Local pre-estimate + sweep | Optimise without using the trial | Medium |
| 6 | History and undo | Safe editing | Low |
| 7 | `insertar_objeto` + object templates from the demos (section 2) | Storage, horizon, loads, bifacial, ageing, stand-alone, pumping | Medium |
| 8 | Hybrid GUI (section 1) | Reach the screens Win32 can't | High |
