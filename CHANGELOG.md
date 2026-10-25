# Changelog

All notable changes to this project are documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Fixed
- **`proyecto configurar` chains variant changes in one call** (`pasos:[{accion:…}]`: dimensionar,
  anadir, seguidor, limite_inyeccion, almacenamiento, perdidas, bombeo, economia). Every step is
  validated before the first runs; `proyecto`, `variante` and `workspace` fill what a step omits.
- **`disenar_sistema` sizes with the first available module** (workspace, then DataRO) when the
  template's own module is not in any database, and says so in `avisos`. A failed run restores the
  project files it had overwritten, not just deletes the new ones.
- **GUI `secuencia` schema declares its steps as objects** so validating clients accept them.
- **`listar objetos` keeps every object with `todos:true`;** by default the list is trimmed
  and `truncado`, `total` and the `detalle:completo` hint say so.
- **One name per output magnitude:** `pr_pct` and `produccion_especifica_kwh_kwp` replace
  the `pr`/`specific_yield_kwh_per_kwp` aliases in easy-mode KPIs; `simular` dry run returns
  `cli_args` only, without the duplicate `command_line`.
- **GUI `pagina` vetoed by the form names the reason.** The error carries the form's
  visible texts; an out-of-range `indice` names the page count.
- **GUI `cerrar` takes `cancelar:true`** to press Cancelar/No before the default close
  buttons. **`cerrar_todo`** closes every visible PVsyst window.
- **GUI `secuencia` runs a list of actions and stops at the first failure,** naming the
  failed step. Partial results are not returned.
- **GUI `funcion` returns a form summary** (`resumen`: buttons, tabs, fields);
  **`rellenar` returns the compact view** of the form after filling it.
- **GUI `teclas` rejects unknown key names** (for example `ZZZ+Q`) instead of typing them.
- **GUI `esperar` honours `esperar_segundos`** over `timeout_ms`.
- **GUI `escribir`/`seleccionar` refuse classes that cannot take the value,** naming the
  class and `campo`; `paso` rejects `operacion` it does not use.
- **GUI `lanzar` refuses `proyecto` while PVsyst is already running,** instead of
  reporting a second instance that never opens.
- **GUI tab selection checks the tab actually changed;** a vetoed tab is an error, not OK.
- **GUI `clic_texto` falls back to UIA text lookup** when OCR cannot draw the text.
- **GUI `controles` lists visible controls by default** (`todos:true` for all).
- **`help` with exit code 0 is success.** `pvsyst_cli help <cmd>` text documents
  markers such as "Required option"; those no longer classify the call as an error.
- **`resultados que:resumen`.** One call returns KPIs, monthly production and the
  three largest losses, with `avisos` when the loss diagram is missing.
- **Approximation-only `resultados` properties are hidden.** `medido_kwh`,
  `medido_csv` and `umbral_pct` leave `tools/list` unless approximations are enabled.
- **`simular` dry_run reports the trial cost.** `costo_estimado` gives the executions
  a run would charge and the remaining licence count; `max_ejecuciones` gives the cap
  for that call (1, or 2 for batch runs).
- **`proyecto crear` takes `proyecto` as the new project name.** `nombre_nuevo` still
  works; the error names both keys when neither is given.
- **New projects get the project name as their demo comment.** A `DEMO` comment is
  replaced by the project name unless `comentario` is passed, and `comentario` is
  written on both `pvProject` and `pvVCalcul`.
- **`proyecto dimensionar` validates the module and inverter before writing.** A
  module that does not exist (and has no inferable `Pnom`) or an unknown inverter is
  rejected, and `modulo`/`inversor` are refused on the GUI-only route.
- **`disenar_sistema` rejects a grid design without `n_mod_serie` and `n_strings`
  before any GUI run,** unless `via:"aproximacion"` is requested.
- **`disenar_sistema` checks the realised kWp against `tolerancia_kwp_pct`** (20 % by
  default). A failed run removes the project files it created; pre-existing files and
  `sobrescribir` overwrites are not rolled back.
- **The project site is the one requested.** `disenar_sistema` binds the new `.SIT` to
  the project after the project step and sets `MeteoFileName` only after the simulation.
- **`sitio crear` validates `region`, latitude, longitude and `nombre_sitio` before calling
  PVsystCLI.** An explicit `nombre_sitio` that already holds other coordinates is refused
  unless `forzar` is set; `timezone` is checked too.
- **`sitio convertir_meteo` validates `input_mef_timeshift` (-30 to 30) and the three input
  files before running,** including in `dry_run`, and names the parameter that fails.
- **`componente crear` and `importar` refuse to overwrite without `sobrescribir: true`.**
  `importar` rejects `destino` and points to `nombre_nuevo`.
- **`listar proyectos` filters with `texto`** (case-insensitive substring of the file name).
- **`proyecto p50_p90` rejects a `workspace` override on the PVsyst route** like the other
  GUI-only actions.
- **`proyecto anadir elemento:envejecimiento` works on CRLF demos.** Named closers
  such as `End of TAgeing` are matched without their trailing `\r`, so the
  `pvAgeing` block is no longer left open and the insert no longer fails with
  `Estructura PVObject desbalanceada`.
- **Missing arguments are reported all at once with an example.** A call that
  lacks several required keys now names every one and shows a call that supplies
  them (`Faltan: plantilla, nombre_nuevo. Ejemplo: …`). `editar rvt` without
  `operacion` is an error instead of silently running `listar`.
- **Schema lists the required keys of each action.** The discriminator
  description of `editar`, `proyecto`, … now says which keys each action needs.
- **Answers say what was ignored and how to get more.** Undeclared arguments are
  returned in `ignorados`; trimmed answers carry `truncado: true` and
  `hint: detalle:completo`; `siguiente` after `resultados kpis` and
  `proyecto crear` names a call that exists.
- **MCP instructions and prompts match the real behaviour.** The approximation
  gate is mentioned only when enabled; the instructions list the actions that
  spend TRIAL executions and the 13-item default truncation. `disenar_desde_coordenadas`
  uses component names the bundle ships, and `convertir_a_seguidor` creates a
  variant first.
- **`listar componentes` accepts names as `tipo`.** `inverters`, `modulos`,
  `batteries`… resolve to the PVsyst codes (`OND`, `PAN`, `BTR`).
- **`leer` accepts `Projects/X.VC0` and `Projects\X.VC0`.**
- **`editar` refuses field names with `=` or empty names.** A key such as `A=B`
  was written as `A=B=1` and read back as `A: "B=1"`. The edit now fails with
  an error naming the key, before the file is touched.
- **Tool schemas load in Gemini and OpenAI strict mode.** Array properties
  (`variantes`, `filas`, `paginas_reporte`, `extra_options`, `variables`,
  `ediciones`, `puntos`, `opciones`, `valores`) had no `items`, and Gemini
  rejected the whole tool list (`400 INVALID_ARGUMENT: properties[…].items:
  missing field`). Every array now declares its element type.
- **`source` in `sitio` reaches PVsystCLI again.** The new alias table mapped it to
  the `origen` selector, so `sitio crear {source: …}` failed with "Origen … no
  válido". No alias may now shadow a declared property.
- **`simular` no longer passes a CLI flag twice.** With both `csv` and `output_csv`
  the command got two `-ocf` options and the cache tracked the other file; the
  short names (`csv`, `pdf`, `bpf`, `sitio`, `cof`…) are now aliases of the
  canonical argument, and `simular {csv: …}` reports it as `output_csv`.
- **`proyecto eliminar` with `variante` deletes only that variant.** It used to
  move the whole project (PRJ and every VC/CM file) to the recycle bin. Now only
  the named variant and its `Results/<proyecto>_<VARIANTE>.csv|pdf` are moved; an
  unknown variant is an error that names it.
- **`sitio escribir_serie` rejects unknown variables.** A name that is neither a
  column of the file nor in the PVsyst catalog (e.g. `ZZ_NoExiste`) was appended
  as a new column with unit `u`; it is now an error that names the variable, and
  the file is left unchanged.
- **The remaining loose aliases of `sitio` and `disenar_sistema`** (`fuente`,
  `zona_horaria`, `pais`, `inclinacion`, `azimuth`, `potencia_kwp`…) live in the
  same synonym tables, so they are validated and announced like the rest.
- **Hidden approximation actions stay hidden.** Without `PVSYST_MCP_APROXIMACIONES`
  the `accion`/`que` descriptions and `pvsyst://cobertura` still listed
  `preestimar`, `barrido`, `medido_vs_simulado`, `geometria_solar` and their aliases.
- **GUI capture of a zero-sized window or control returns an error instead of
  panicking.** `recortar_rgb` called `clamp(0, ancho - 1)` with `ancho == 0`, and
  an empty crop then reached `png_rgb`, whose `chunks(0)` panicked.
- **GUI messages can no longer hang the server, and a timeout is reported.**
  `SendMessageW` waited without limit when PVsyst stopped pumping messages; every
  synchronous message now uses `SendMessageTimeoutW` (3 s, `SMTO_ABORTIFHUNG`)
  and checks the handle with `IsWindow`. A timeout is an error for actions and
  typing, and an unknown (`null`) checked state, never a silent success or `0`.
- **Creating a site in another workspace no longer writes into the default one.**
  `sitio crear` by coordinates and `disenar_sistema` dropped the `workspace`
  argument on the inner call, so the ledger cost and `UserData/mcp_cache/` went to
  `PVsyst8.1_Data` (the `MCP_*_Site_Monthly.csv` files left behind by tests).
- **Tests no longer reach the installed PVsyst.** The integration-test client left
  `PVSYST_GUI`/`PVSYST_CLI` unset, so the server under test resolved the real
  executables in `C:\Program Files\PVsyst8.1.7`. They now point to missing paths;
  live `#[ignore]` tests opt in with `Client::start_instalacion_real`.
- **English argument aliases work.** `leer {"file": …}`, `new_name`, `template`
  and the other English names in the code were unreachable: `validacion.rs`
  rejected the call ("Missing required argument 'archivo'") before the handler
  saw the alias. Synonyms now live in one table per tool and are normalised to
  the Spanish name before validation; alias and canonical name with different
  values is an error.
- **`mcpb/manifest.json` matched the code again.** The `proyecto` description had
  drifted; a test now fails when manifest and tool declarations diverge.
- **Aliases work for clients that validate the schema.** `tools/list` listed only
  the canonical values in each `accion`/`que` enum and the canonical names in
  `required`, so a validating client rejected `que:"modules"`, `{"file": …}` or
  `{"action": …}` before the server saw them. The enums now include the aliases
  and an argument that has an alias is not in `required` (its description says
  "Requerido."; the server still enforces it after alias normalization).
- **No hidden action or in-house route is named without
  `PVSYST_MCP_APROXIMACIONES`.** The tool descriptions of `simular`, `sitio` and
  `resultados` and the `via` description still spelled them out.
- **`listar.texto` is no longer described as GUI input.** The GUI argument
  annotations were keyed by property name only; they now apply to `gui_*` tools.
- **Integer arguments honour their declared bounds.** `workspace historial n:-1`
  was accepted and ignored; `n`, `lineas` and `n_baterias_*` are checked against
  the same table that writes `minimum`/`maximum` into the schema.
- **`tools/call` without a string `name` says so** instead of "Unknown tool ''".
- **A non-numeric coordinate is reported as such.** `sitio crear latitud:"abc"`
  answered "Missing required argument 'latitud'"; the three duplicate number
  readers of `sites` collapse into `args::req_numero`.
- **`escribir_archivo` can write an empty file** (`contenido:""` counted as missing).
- **`leer` refuses empty and truncated PVObject files.** An empty `.PRJ` or a
  `.PRJ`/`.SIT`/`.MET` cut off mid-structure came back as a success with empty
  sections; one check in `format::object::parse_completo` now serves every reader.
  Hints no longer name tools that do not exist (`leer_archivo`, `leer_objeto`,
  `listar_objetos`, `listar_proyectos`, `leer_modulo`).
- **`simular` reports the CSV and PDF it passes to PVsystCLI**, in `dry_run` and
  in the real run, taken from the same values that build `-ocf`/`-rpf`.
- **GUI: a click on a window a modal disables is an error.** `clic_texto`,
  `click`, `clic_fisico`, `cerrar`, `escribir`, `seleccionar` and `pagina` returned
  success while Windows dropped the input; they now fail with the same message
  `funcion` already gave, naming the dialog that blocks it. `cerrar` also fails
  when the window is still open after the wait, and a window that is already gone
  reads "no encontrada (¿ya cerrada?)".
- **GUI: a refused tab change is visible.** `pagina` reads the selected tab back
  (`TCM_GETCURSEL`) and fails when PVsyst's field validation vetoed the change;
  `clic_texto` on a tab strip adds an `advertencias` entry when the tab did not
  change. `formulario` marked every tab `seleccionada:false` (it reported the UI
  Automation capability, not the selection).
- **GUI: `capturar` refuses a hidden control** instead of returning a uniform grey
  PNG; `vista` honours `compacto`.

### Changed

- **Tool schemas announce the English aliases.** Every property and every
  `accion`/`que` value lists its synonyms (`archivo (alias: file)`), generated
  from the same table as `pvsyst://cobertura`, the manifest and the alias block of
  README/README.es.
- **Tests can no longer touch the installed PVsyst or the user's workspace**, and
  `scripts/verificar.ps1` runs fmt, clippy and each test binary with a timeout,
  reporting any `PVsyst.exe` left behind.
- **Agent view of forms.** `gui_observar que:esquema` lists the forms PVsyst
  exposes by stable key (today `components.panels`, the PV module window's
  "Datos básicos" page: model, manufacturer, nominal power, technology,
  reference conditions, Isc, Voc, Impp, Vmpp, muIsc, cells in series), each
  with unit, range and the key PVsyst writes in the file. `que:vista` returns
  the live values, enabled state, validity and, for lists, the options the list
  offers, read from the controls (30 ms, no OCR, any DPI); `ocr:true` adds the
  text PVsyst draws (validation messages, model summary). `gui_actuar
  accion:rellenar entidad valores` checks every value first (keys, type, range,
  list options) and types nothing if any is wrong; `simular:true` only reports.
  It returns, per field, what was asked, what was there, what PVsyst holds
  afterwards and whether they match, plus the fields PVsyst recomputed on its
  own. `gui_observar que:formulario` accepts `ocr:false` and no longer drops
  fields at the default `detalle`.
- **`gui_observar que:capturar` no longer returns a black margin.** PVsyst is
  DPI-unaware: it draws its 96-DPI layout in the top-left of a bitmap as large
  as the physical window. The capture is now cut to the drawn area (800x640
  instead of 1200x960 at 150 %), and `recorte:true` scales the control's
  rectangle the same way.

- **OCR of the GUI is much faster.** Enlarging and sharpening the capture
  computed neighbour indices per pixel; they are now tabulated once per axis.
  In a release build `gui_observar que:formulario` and `gui_actuar
  accion:clic_texto` take about 0.4 s on THomeForm (1.4 s with the previous
  release code, 6-10 s in a debug build). Use the release binary.

- **All 15 PVsyst-native tools are available by default.** `pvsyst_cli`,
  `gui_observar` and `gui_actuar` no longer need `PVSYST_MCP_EXPERTO`; the
  variable and the expert access level are gone.
- **Only the MCP's own calculations are opt-in.** They sit behind
  `PVSYST_MCP_APROXIMACIONES=1`; without it they are hidden from `tools/list`,
  from `prompts/list`, from `pvsyst://recetas` and from `pvsyst://cobertura`,
  and calling them returns the PVsyst route that replaces them. With it, their
  results carry `origen: "aproximacion_mcp"` and `aviso_origen`. Gated:
  `simular preestimar`/`barrido`, `sitio geometria_solar`,
  `resultados medido_vs_simulado` and the automatic sizing of
  `disenar_sistema`. Actions whose default is already PVsyst stay visible:
  `componente crear_desde_ficha`, `sitio horizonte_pvgis`,
  `resultados p50_p90` and `proyecto dimensionar` by `kwp` use PVsyst's own
  windows and only run the MCP calculation when called with
  `via: "aproximacion"` and the flag is on.
- `resultados que:carbono` and `que:economia` read PVsyst's report (Carbon
  Balance page; investment, yearly costs, payback and LCOE from the
  economic-evaluation block) instead of computing their own models; the
  `capex_por_kwp`, `tarifa_por_kwh`, `opex_anual_por_kwp`, `tasa_descuento`,
  `anios`, `moneda`, `vida_anios`, `degradacion` and `lce_kgco2_por_kwp`
  arguments are gone. Report numbers with `.` as thousands and decimal
  separator (`23.410.00`) are parsed correctly.
- `proyecto dimensionar` writes `n_mod_serie`/`n_strings` as given; `kwp` is
  optional. `disenar_sistema` uses them directly when both are passed.
- `disenar_sistema` and `resultados que:kpis` take energy, specific production
  and PR (in %) from PVsyst's PDF report, not from CSV column sums; the KPI
  `e_grid_kwh` is now `energia_producida_kwh` (PVsyst's "Produced Energy") and
  `poa_kwh_m2` is gone. Summary boxes printed as a row of labels followed by a
  row of values (battery and self-consumption reports) are read correctly.
- `resultados que:csv` integrates only power columns; it no longer reports an
  hourly average of ratios (`PR`) or sums of angles.
- `resultados que:entregable` contains only PVsyst data; the P50/P90 section
  appears only with approximations enabled, marked as such.

### Added

- **GUI automation of controls without a window handle.**
  `gui_actuar accion:clic_texto ventana texto [indice]` clicks the text a
  person reads (Windows OCR on an enlarged, sharpened capture): speed buttons
  of the toolbars, rows of `TVirtualStringTree` lists (project load) and
  `TStringGrid` grids. Several matches without `indice` are refused, not
  guessed. `accion:click` and `clic_fisico` take `dx`/`dy` (px at 96 DPI from
  the control's corner).
- **Menus by position.** The command ids of a secondary form's menu change
  with the instance (the project window's moved by 3 between two runs), so
  `gui_actuar accion:menu ventana menu item` reads the id from the open popup
  (`MN_GETHMENU`); `gui_observar que:menus` lists every menu of a window with
  ids, separators and submenus. Needs a real desktop (VCL ignores posted keys).
  The project window and the 3D scene editor are catalogued this way.
- `gui_actuar accion:reiniciar` recovers from PVsyst's madExcept dialog or the
  "unstable after a crash" warning (restart button, or No + launch); it refuses
  to act on any other screen.
- Menu commands are refused while the host form is disabled (a modal dialog is
  open): PVsyst raised an internal exception and had to be restarted.
- `gui_observar que:formulario` returns `textos` (captions outside controls,
  e.g. a message box's question) and `pestanas_error`; labels are read from
  segmented OCR words, so a caption no longer glues to its neighbour field.
- Catalogue: second-level dialogs of every project-window form (`dialogos_*`),
  entries carry `pagina` (tab of the host) and `nota` (what could not be
  observed). `docs/gui_mapa.json` now maps every window with its tabs, fields
  and buttons, the menus of the home, project and 3D-editor windows, and a
  decision per control type without handle.
- `leer` decodes `Currency.dat`, `.MEF` (modern and legacy) and the 3D scenes
  `.SHD/.BLD/.SHO` (read-only); `escribir_archivo` refuses scenes, `Currency.dat`
  and legacy `.MEF`, and accepts a modern `.MEF` only if it re-serialises
  identically.

- **GUI function catalogue.** `gui_observar que:funciones` lists 203 PVsyst
  functions (the start page's menus and buttons, Herramientas, Bases de datos,
  Datos medidos, Ayuda, Licencia, Idioma and the project window's buttons and
  menus) with the window each opens and its `efecto` (`abre`, `modifica`,
  `cierra`). `gui_actuar accion:funcion funcion:<clave>` runs one, first
  opening the host window it needs (`herramientas.factor_transposicion` opens
  Herramientas first); `accion:menu ventana id` posts any other menu command
  id (the main form's ids are stable; the project window's are not, see below).
  New resource `pvsyst://gui` (five resources now) returns the catalogue and
  `docs/gui_mapa.json`: 82 windows with titles, tabs, fields and buttons observed on
  PVsyst 8.1.7 (Spanish interface).
- **Forms by label.** `gui_observar que:formulario` pairs each field of a VCL
  dialog with the caption a person reads next to it (Windows built-in OCR; the
  captions are windowless `TLabel`s that Win32 and UI Automation cannot see),
  with checked state, values, buttons and tabs; ambiguous pairings are
  reported, never guessed. `gui_actuar accion:campo etiqueta valor` sets a
  field by that label (text, combo or list item, check or radio) and returns
  the value before and after; a label that matches several fields is an error
  listing them. `seleccionar` now handles list boxes, including multi-select.
  Needs a Windows OCR language pack (Spanish or English).
- **Isolated desktop.** `PVSYST_MCP_ESCRITORIO=aislado` creates a private Win32
  desktop, runs PVsyst there (`gui_actuar accion:lanzar`) and posts every click
  and key as window messages, so the agent never takes the user's focus,
  cursor or keyboard. Verified with PVsyst: window enumeration, UI Automation,
  captures, catalogue functions, label-addressed fields and posted clicks.
  Limits: PVsyst shows "already running" when the user has it open (answer it
  yourself); posted keys are ignored by VCL controls and Ctrl/Shift
  combinations are refused. `entorno` reports `escritorio`.
- `gui_actuar accion:cerrar` answers the madExcept crash dialog with
  *Continuar* only and never answers a Sí/No question for you.
- **`.HOR` horizon files.** `leer` decodes them and `sitio accion:escribir_horizonte`
  writes them (Delphi Real48 records, cubic profile). Byte-exact round trip on
  PVsyst's own `DEMO_Geneve.HOR`; a generated file loaded in PVsyst's horizon
  window shows the five points and the profile.
- **Hourly series of `.MET` and `.DAM`.** `leer serie:true offset limite`
  decodes the packed days (base-62 tokens, per-variable scale) and
  `sitio accion:escribir_serie` rewrites chosen variables keeping the header
  bytes. Re-encoding real files is byte-identical; a file with `T_Amb` +20 °C
  opened in PVsyst's weather tables with monthly means shifted by 20 °C.
- **Legacy `.PAN` editing.** `componente accion:editar`, `editar` and
  `componente crear` patch only the Real48 slots of the edited fields in
  pre-6.40 binary modules (new exact `encode_real48`).
- **More templates and `.RVT` editing.** `proyecto anadir` gains `domo`,
  `zona_campo`, `bifacial_seguidor` and `circuito_hidraulico`, taken from
  variants that exist in PVsyst's demos; `editar accion:rvt` lists, adds,
  removes and reorders result variables of `.RVT` templates against the names
  PVsyst itself writes.
- **GUI field automation.** `gui_actuar` now has `escribir` (type into a measured
  96-DPI field or directly into a control), `seleccionar` (combo item by prefix),
  `pagina` (tab page) and `esperar_cierre`. `gui_observar que:mensajes` returns
  PVsyst's message-box and warning log. Keys are sent with `keybd_event` instead
  of posted `WM_KEY*`, which VCL forms ignored. Log discovery no longer hardcodes
  PVsyst 8.1, message-box acceptance covers OK/Sí/Aceptar/Continuar/Cerrar,
  variant saving compares file bytes instead of `mtime`, and every GUI action
  invalidates the UIA tree cache.
- `avisos_pvsyst` on every finished simulation (`simular ejecutar`, `estado`,
  `disenar_sistema`): the summary PVsyst wrote to its CLI log, such as
  "The array Vmpp at 60 °C is lower than the inverter minimum operating
  voltage" and "1879 occurrences of MPP voltage lower than min. voltage input
  of the inverter", without the per-hour detail lines or licence noise.

### Fixed

- **GUI coordinates on scaled screens.** The process now declares itself
  Per-Monitor-DPI-aware (v2) at start-up and scales 96-DPI positions by the
  DPI of the monitor, not of the PVsyst window (which reports 96 when PVsyst is
  DPI-unaware). Before, on a 150 % screen the MCP saw PVsyst's window at
  801x701 instead of 1202x1052, so coordinate clicks missed. Verified at 150 %
  only; 100 % and 125 % are covered by unit tests of the scaling.

## [0.3.0] - 2026-10-20

### Added

- **Trackers**: `proyecto accion:seguidor` converts a single-orientation variant
  to one-axis tracking from the `_DEMO_Tracking` template, preserves `NoOrient`,
  and refuses 3D shading scenes, bifacial sheds and multi-orientation variants.
  `proyecto doctor` now checks tracker geometry.
- **Injection limit**: `proyecto accion:limite_inyeccion` writes `PLimGrid`,
  `LimGrSeparLoss` and optional `TanPhi`.
- **Battery storage**: `proyecto accion:almacenamiento` configures
  self-consumption or peak shaving in `pvGridStorage`, creates the storage
  objects when missing, and reports every inserted object.
- **Variant losses**: `proyecto accion:perdidas` writes soiling, thermal, LID and
  mismatch losses on `pvMainArray`/`pvSubArray`; the new PVsyst soiling model is
  rejected with an explicit message.
- **Pumping**: `proyecto accion:bombeo` edits the pump file, static depth and the
  `WaterUse` section.
- **PVsyst economics**: `proyecto accion:economia` updates every `DiscountRate`
  plus currency, project life, inflation and feed-in tariff in `EconomicEval`.
- **Measured vs simulated**: `resultados que:medido_vs_simulado` compares monthly
  `E_Grid` from the PDF with a measured kWh array or CSV and flags months over a
  threshold.
- **Stand-alone easy mode**: `disenar_sistema` accepts `tipo_sistema:aislado`,
  uses `_DEMO_StandAlone.VC3`, and applies optional `n_baterias_serie` /
  `n_baterias_paralelo` after sizing.
- **Format primitives**: multi-object edits, object replacement and section
  editing with duplicate-key, insertion, CRLF and missing-section coverage.
- **Recipes and coverage**: three new prompts (`convertir_a_seguidor`,
  `autoconsumo_con_baterias`, `comparar_medido`), ten prompts total, and the
  coverage resource names every new action.
- **Schema constraints**: enums for `estrategia`/`tipo_sistema`, a 12-item
  `medido_kwh`, and battery-count minimums.

### Changed

- `format::seccion` treats deleting an absent field as `NotFound`, matching
  `editar`.
- Numeric argument readers are shared from `args`, and the new recipes share
  their project/variant arguments.
- The catalogue applier is a value-returning function
  (`aplicar_elemento`/`ElementoAplicado`) reused by storage.

### Fixed

- Injection-limit and storage actions reject peak shaving without `PLimGrid`
  before writing.
- The pumping action no longer forces `FlowModul=Yearly` when only the static
  depth changes.
- Stand-alone sizing tolerates variant-derived generic modules that have no file,
  while still rejecting explicit unreadable modules.
- New direct fields are inserted at the object's direct level even when only
  nested fields exist.
- `mcpb/manifest.json` descriptions match the tool schemas.

### Verification

- `cargo fmt --check`, `cargo clippy --all-targets -- -D warnings`, `cargo test`,
  `cargo +1.88-x86_64-pc-windows-gnu check --all-targets`, `cargo audit`,
  `cargo deny check`, `npm test` and `npm audit --audit-level=high` pass.
- The 14 demo-backed tests (trackers, injection, storage, losses, pumping,
  economics and stand-alone) pass against PVsyst 8.1.7 demo data without
  consuming a trial execution; GUI/network/real-simulation suites remain
  `#[ignore]`.

## [0.2.0] - 2026-10-20

### Added

- **MCP tool metadata**: every tool now declares a title plus `readOnlyHint`,
  `destructiveHint`, `idempotentHint` and `openWorldHint`, and its description is
  bilingual (Spanish / English). The `.mcpb` manifest carries the same texts.
- **MCP prompts**: `prompts/list` and `prompts/get` expose the seven recipes
  (site design, step-by-step study, results, orientation sweep, GUI, file
  explorer and environment diagnosis); `pvsyst://recetas` keeps the resource form.
- **Protocol 2025-06-18**: object results also carry `structuredContent` (errors
  add `{error, tipo}`), long simulations emit `notifications/progress` and accept
  `notifications/cancelled`, and the negotiated version is fixed by the first
  `initialize`. `2024-11-05` and `2025-03-26` remain supported.
- **Responsive loop**: one worker thread executes tools while the main loop keeps
  answering pings and cancellations; cancelled calls never answer.
- **Synthetic legacy `.PAN` fixture** with an exact `Real48` encoder in Python and
  a non-ignored end-to-end decode test.
- **Property tests** for PVObject insert/extract/remove and for byte-identical
  history restore.
- **Shared calendar module** (proleptic Gregorian, Hinnant algorithm) replacing the
  three duplicated conversions.
- **Safer FFI**: 76 `unsafe` blocks reduced to 40, all with `SAFETY`; repeated
  Win32 calls live in `adapters/gui/ffi/` and UIA COM calls in `adapters/uia/com.rs`.

### Changed

- Tool errors use `MissingArg`, `NotFound`, `Io` or `Cli` consistently instead of
  `Invalid` for absent arguments and resources.
- Boolean parameters were replaced by domain enums in the affected APIs.
- `leer_bytes` has one definition in `workspace/lectura.rs`; the documentation of
  the touched modules was audited.
- The crate is explicitly Windows-only and declares `rust-version = "1.88"`
  (`proptest` 1.12 and `slice::as_chunks` need it; 1.85 does not compile).
- `tracing-subscriber` uses only the `env-filter`, `fmt` and `std` features; the
  log output is unchanged.

### Fixed

- ZIP listings and imports accept accented entry names in the console code page.
- Progress polling no longer copies the whole captured stdout when no new bytes
  arrived.
- A cancellation that races with response writing cannot delete a newer
  registration for a reused request id, and reader threads are joined on the
  cancel path.

### Verification

- `cargo fmt --check`, `cargo clippy --all-targets -- -D warnings`, `cargo test`
  and `cargo +1.88-x86_64-pc-windows-gnu check --all-targets` pass.
- The `#[ignore]` suites that need a real PVsyst installation, the GUI or network
  were not executed in this release environment; GUI wrapper behavior was verified
  by the existing unit/integration coverage, not by a live PVsyst session.

## [0.1.1] - 2026-10-20

### Added

- **npm auto-update**: the `npx pvsyst-mcp` launcher runs the newest binary it has
  (packaged or previously downloaded to `%LOCALAPPDATA%\pvsyst-mcp`) and, after the
  server starts, a detached process downloads a newer release for the next start.
  The running session never switches binaries. `PVSYST_MCP_AUTO_ACTUALIZAR=0`
  disables it; `PVSYST_MCP_ALMACEN` moves the store. Zip and `.mcpb` installs do
  not update themselves.

### Fixed

- Empty environment variables (`PVSYST_WORKSPACE`, `PVSYST_CLI`, `PVSYST_GUI`,
  `PVSYST_DATARO`, `LOCALAPPDATA`, …) count as unset, so the `.mcpb` form can be
  left blank and the workspace is auto-detected again; the bundle's path fields
  are now optional.
- A workspace import whose rollback cannot put a backup back now says so and
  names the backup path instead of losing it silently.
- Zip imports stream the `tar` listing with byte and entry caps and reject
  archives above 4 GiB, so a hostile archive cannot exhaust memory.
- `proyecto doctor` and `disenar_sistema` read Vmpp and μVoc from both `.PAN`
  layouts; the cold-Voc check was silently skipped before, and μVoc is now
  applied relative to Voc.
- `resultados` accepts a numeric `kwp` (it was ignored unless sent as text).
- `resultados que:entregable` writes through history, so regenerating it can be
  undone, and returns `historial_id`.

### Distribution

- Built locally with the `x86_64-pc-windows-gnu` toolchain because the release
  workflow could not run; the binary imports no MinGW runtime DLLs.

## [0.1.0] - 2026-10-19

First public release: an MCP server that operates a PVsyst 8.1.7 installation
through its data files and the documented `PVsystCLI` commands only.

### Added

- **Licence**: Apache-2.0 with a `NOTICE` file, so any fork or redistribution must
  mark the files it changed and keep the attribution notices (sections 4(b)–4(d) of
  the licence). The name is not licensed (section 6): a fork cannot present itself
  as this project.
- **MCP server over stdio** (`pvsyst-mcp`, lib + thin binary) negotiating protocol
  versions `2025-03-26` and `2024-11-05`, with 15 tools (12 on the default surface)
  and four resources: `pvsyst://recetas`, `pvsyst://cobertura`, `pvsyst://formato`
  and `pvsyst://cli`.
- **Byte-preserving file core** (`format/`): PVObject text parser (BOM-safe),
  strict object selector, object span scanner, field edits, object insert/extract
  and removal — with structural validation that refuses unbalanced files.
- **Snapshots and validated restore** (`history/`): every write is snapshotted to
  `UserData/mcp_history/<id>/manifest.json` before it happens; restore validates
  the whole manifest first and never writes outside the workspace.
- **Project and variant lifecycle**: create, delete, rename, variants, sizing
  (`NModSerie` / `NStringCh` / `PNomSizing`), consistency doctor, demo-object
  insertion and temporary-file cleanup.
- **Components**: listing and filtering, creation, import/export, and
  `crear_desde_ficha` (builds `.OND` / `.PAN` files from datasheet values),
  including the recovered legacy binary `.PAN` layout (Delphi `Real48`) with a
  standalone reference decoder in `tools/pan_decode.py`.
- **Sites and weather**: site creation from coordinates or CSV, meteo conversion,
  solar geometry, PVGIS horizon and TMY hourly weather (endpoints verified live),
  NASA SSE grids, and reuse of existing sites when coordinates match.
- **Simulation**: preparation, execution, background jobs, batch parameter files,
  parameter sweeps, simulation cache, local pre-estimate, and trial-budget ledger
  that asks for confirmation before spending several trial executions.
- **Results**: KPIs from the output CSV, monthly tables, loss diagrams, P50/P90,
  carbon and economics, and a Markdown deliverable generated from the PDF report.
- **Windows GUI bridge** (expert mode): window and control enumeration, UI
  Automation tree inspection, GDI screenshots, clicks, keystrokes, waits.
- **Typed errors** (`src/error.rs`) and a workspace-confined path layer that
  rejects `..` and out-of-workspace paths unless explicitly opted out.
- **Toolchain**: permanent lints (`unwrap`/`expect`/`panic`/`todo`,
  `undocumented_unsafe_blocks`, `print_stdout`/`print_stderr`, complexity and file
  size limits), hermetic test fixtures, property tests, CI running `fmt`,
  `clippy -D warnings`, `test`, `cargo llvm-cov` and the Python tooling checks, plus
  `cargo audit` and `cargo deny check` in the documented development checks.
- **Distribution**: GitHub Releases ship the release binary as a zip with
  `LICENSE`, `NOTICE` and the READMEs, plus a one-click `.mcpb` bundle for Claude
  Desktop built from `mcpb/manifest.json`. The v0.1.0 binary was built locally with the
  `x86_64-pc-windows-gnu` toolchain; later `v*` tags are built by
  `.github/workflows/release.yml`. `pvsyst-mcp@0.1.0` on npm (Windows x64, `npx -y pvsyst-mcp`) is an
  earlier build published on 2026-10-15 from the pre-release history; its README
  links to a `master` branch that no longer exists.
