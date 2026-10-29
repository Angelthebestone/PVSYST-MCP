# PVsyst MCP — coverage map

How the MCP covers the PVsyst 8.1.7 feature surface across **all three layers**
PVsyst exposes: data files, the CLI, and the desktop GUI. The full, page-by-page
feature inventory (623 help pages) lives in
[`pvsyst_features.md`](./pvsyst_features.md). The same map is available at
runtime through the `pvsyst://cobertura` resource.


## Principle

PVsyst exposes three surfaces: **text files** (the `PVObject` format, since v6.40
/ UTF-8 since v6.80), a **documented CLI**, and a **Win32/VCL desktop app**.
Instead of one tool per GUI button, the MCP exposes **generic primitives over all
three**:

* edit **any field of any object in any PVsyst file** (`editar` `accion:campos`);
* read/parse **any file** (`leer`, `listar` `que:objetos`);
* run **any PVsystCLI command** (`pvsyst_cli`);
* drive **any window/control of the GUI** (`gui_observar` / `gui_actuar`).

Everything else is a convenience layer on top. So coverage is bounded by the
*data + CLI + GUI surface*, not by a fixed menu.

## Tools (15 by default)

**Environment & licence:** `entorno` (`accion`: `diagnostico`, `licencia*`, `exportar_logs`, `logs_cli`, `preferencias*`).

**End-to-end (start here):** `disenar_sistema` (coordinates + kWp → site, project, sizing, simulation, KPIs), `resultados` (`que:kpis`).

**Generic primitives** — reach anything: `leer` (type detected from `archivo`), `listar` (`que`: `proyectos`, `componentes`, `modulos`, `archivos`, `objetos`), `editar` (`accion`: `campos`, `insertar_objeto`, `eliminar_objeto`), `escribir_archivo`, `pvsyst_cli`.

**Projects & variants:** `proyecto` (`accion`: `crear`, `eliminar`, `renombrar`, `crear_variante`, `dimensionar`, `doctor`, `anadir`, `seguidor`, `limite_inyeccion`, `almacenamiento`, `perdidas`, `bombeo`, `economia`, `p50_p90`, `limpiar_temporales`).

**Components (all kinds):** `componente` (`accion`: `crear`, `borrar`, `importar`, `exportar`, `crear_desde_ficha`).

**Site & weather:** `sitio` (`accion`: `crear` with `origen` `coordenadas`|`csv`, `convertir_meteo`, `geometria_solar`, `horizonte_pvgis`, `meteo_pvgis`).

**Simulation, batch & results:** `simular` (`accion`: `ejecutar`, `preparar`, `estado`, `preestimar`, `barrido`), `resultados` (`que`: `kpis`, `csv`, `perdidas`, `mensual`, `comparar`, `medido_vs_simulado`, `p50_p90`, `carbono`, `economia`, `entregable`).

**Workspace / admin:** `workspace` (`accion`: `crear`, `exportar`, `importar`, `exportar_proyecto`, `historial`, `restaurar`).

**GUI automation:** `gui_observar` (`que`: `ventanas`, `controles`, `leer`, `capturar`, `uia`, `arbol`, `mensajes`, `funciones`, `formulario`, `vista`, `esquema`, `menus`), `gui_actuar` (`accion`: `click`, `clic_fisico`, `mensaje`, `cerrar`, `activar`, `teclas`, `fila`, `escena`, `escena_leer`, `escribir`, `seleccionar`, `pagina`, `esperar_cierre`, `lanzar`, `uia_invocar`, `esperar`, `paso`, `funcion`, `menu`, `campo`, `rellenar`, `clic_texto`, `reiniciar`). `PVSYST_MCP_ESCRITORIO=aislado` runs PVsyst and these tools on a private hidden desktop.

**Output contract:** every answer of the 15 tools, errors included, carries `vista` (`estado`, `resumen`, `origen`, `campos`, `avisos`, `siguiente`, `recortado`, `error`). `siguiente` holds calls the target tool accepts as written. Per-action summaries and next steps: `src/vista_agente/contrato/catalogo/`; the tool × action matrix is in `docs/vista_agente_salidas.md`.

**Resources:** `pvsyst://recetas`, `pvsyst://cobertura`, `pvsyst://formato`, `pvsyst://cli`, `pvsyst://gui`.

**GUI map:** `gui_observar que:funciones` lists the catalogued GUI functions;
`gui_actuar accion:funcion funcion:<clave>` opens one (opening its host window
first when needed) and `accion:menu` posts any other menu command id.
`pvsyst://gui` carries the same catalogue plus the windows, tabs and controls
observed on PVsyst 8.1.7 (`docs/gui_mapa.json`).


## Area-by-area

| PVsyst area | Artifact | Coverage |
| Workspace, any folder/file | `*_Data/**` | `listar` `que:archivos`, `leer`, `escribir_archivo` |
| Project definition | `.PRJ` | `proyecto` (`crear`, `eliminar`, `renombrar`), `editar` |
| Simulation variant | `.VCi` (`.VC0..VCZ`) | `proyecto` (`crear_variante`, `dimensionar`), `editar` |
| Comparison variant | `.CMi` | `proyecto crear_variante`, `editar`. **PVsystCLI cannot simulate `CMx` variants** (verified with `_DEMO_N13_Motorway_Data.CM0`: "Simulation cannot be run: the orientation is not defined"): run them from the GUI (`gui_actuar`). |
| Preliminary design | `.PSG`, `.PSS`, `.PSP` | `leer`, `editar`, `escribir_archivo` |
| Orientation | `pvOrient` (`FieldType/FieldTilt/FieldAzim`) | `editar` (`campos`), `proyecto doctor` |
| Array sizing | `pvSubArray` (`PVModule/GInverter/NModSerie/NStringCh/NInverter`) | `proyecto dimensionar`, `editar` |
| System type / circuit | `pvSystem`, `pvCircuit` | `editar` |
| Losses, IAM, thermal, quality | `pvMainArray`, `pvSubArray`, `pvIAM`, `pvShading` | `editar` |
| Storage, load, horizon, ageing, bifacial, carbon | `pvGridStorage`, `pvLoad`, `pvHorizon`, `pvAgeing`, `pvBiFacial`, `pvCarbonBalance` | `proyecto anadir`, `editar` `insertar_objeto` |
| PV modules | `.PAN` | `leer`, `listar` `que:modulos`, `componente crear_desde_ficha` |
| Inverters | `.OND` | `componente` (`crear`, `borrar`, `crear_desde_ficha`), `listar` `que:componentes` |
| Batteries `.BTR`, Pumps `.PMP`, Regulators `.RLT`, Pump ctrl `.RLP`, Gensets `.GEN`, Sellers `.REV`, Clients `.CLI` | `ComposPV/*` | `componente`, `listar`, `leer` |
| Price list | `.PRI` | `leer`, `escribir_archivo` |
| Geographic site | `.SIT` | `sitio` (`crear`), `leer`, `editar` |
| Weather data | `.MET`, `.MEF`, `.TFT`, `.DAT` | `sitio` (`convertir_meteo`, `meteo_pvgis`, `escribir_serie`), `leer` (`serie:true` decodes the hourly body of `.MET`), `escribir_archivo` |
| Measured data | `.DAF`, `.DAM` | `leer` (`serie:true` for `.DAM`), `sitio escribir_serie`, `escribir_archivo` |
| Horizon / shading scene | `.HOR`, `.SHD`, `.BLD`, `.SHO` | `.HOR`: `leer` and `sitio escribir_horizonte` (structured); `.SHD`/`.BLD`/`.SHO` are decoded read-only by `leer` (objects, positions, elementary shapes; `escribir_archivo` refuses them: mesh format not proven). The 27 elementary shapes are built in PVsyst's own 3D editor by `gui_actuar accion:escena` (PVsyst saves the file, the MCP compares it field by field) and the open scene is read back with `accion:escena_leer`; `sitio horizonte_pvgis` |
| Models / templates | `.LOD`, `.SPL`, `.SFI`, `.RVT` | `leer`, `escribir_archivo`; `.RVT` variable list: `editar accion:rvt` |
| Batch/parametric runs | `.CSV` + `.RVT` | `simular` (`preparar`, `ejecutar` with `filas`) |
| Param override | `Param_Modif.dat` | `simular` (`preparar` with `parametros`) |
| Simulation | engine | `simular`, `pvsyst_cli run-simulation` |
| Results CSV / report PDF | `.CSV`, `.PDF` | `resultados` (`csv`, `perdidas`, `mensual`, `entregable`), `resultados p50_p90` (reads PVsyst's report) |
| P50/P90 set-up | PVsyst window | `proyecto p50_p90` (PVsyst's energy-management window) |
| Licensing / logs / version / help | CLI | `entorno` (`licencia*`, `exportar_logs`, `logs_cli`), `pvsyst_cli` |
| GUI-only authoring: 3D shading editor, informative I/V & table tools, import wizards, dialogs | live Win32/VCL windows and UI Automation | `gui_observar`, `gui_actuar` |


## GUI automation (what was the boundary)

PVsyst is a Delphi/VCL application, so its dialogs and editors are **real Win32
windows**. The `gui_observar` and `gui_actuar` tools enumerate them (`EnumWindows`/`EnumChildWindows`), read their text, expose the UI Automation tree (`que:uia`, `que:arbol`, `que:mensajes`), and drive them with standard window messages (buttons via
`BM_CLICK`, dialogs via `WM_CLOSE`), real `keybd_event` accelerators, coordinate
fields (`escribir`), combo boxes (`seleccionar`) and tab pages (`pagina`). So the
features that have no file or CLI backing are reachable too:

* **Verified**: invoking the *Herramientas* (Tools) button opened the live
  `TToolsForm` window; `gui_observar que:capturar` produced a screenshot of the home
  form; `gui_actuar accion:cerrar` closed the dialog and the home form re-enabled.
  `gui_observar que:arbol` merged the real VCL controls with the UIA tree, and
  `que:mensajes` read PVsyst's own log.
* The pre-created VCL forms (`TPlotForm` for graphs, `TTablForm` /
  `TTablWriteForm` for tables and "save table values as file", wizards, etc.)
  are discoverable and drivable the same way.

This is **black-box**: no PVsyst code is read, disassembled or patched — only
the public Windows UI API is used.

### Residual limit (honest, and small)

Mouse-precision *drawing* inside the 3D scene canvas (placing individual objects
by dragging) is possible only via synthetic pointer input and is inherently
fragile; the tools provide `gui_actuar accion:mensaje`, `accion:teclas` and
`accion:clic_fisico` for those cases. The scene editor creates elementary shapes with `accion:escena` (up to 20 objects, measured on PVsyst 8.1.8 in PRUEBA; buildings, ground objects, PV tables and meshes are not measured). The scene editor's menus are reachable by position (`gui_actuar accion:menu menu item`, `gui_observar que:menus`); its canvas, the charts (`TPlotForm`) and the interactive map are drawn content: capture only. `.SHD/.BLD/.SHO` are read-only for the MCP (`leer`; PVsyst writes them); `.HOR` has a structured codec. See `docs/gui_mapa.json` (`controles_sin_hwnd`, `limites`) for the per-control decision and `Currency.dat`/`.MEF` are read-only/validated as described in `docs/pvsyst_format.md`.

## Verified end-to-end

`disenar_sistema` (coords 46.2044, 6.1432, ~507 kWp) → `resultados que:perdidas` on the resulting variant. The step-by-step form (`sitio crear` → `proyecto crear` → `proyecto dimensionar` → `simular` → `resultados`) is in the README "Verification".
