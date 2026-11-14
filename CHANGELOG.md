# Changelog

All notable changes to this project are documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Changed
- **A successful PVsystCLI run is no longer blamed for the crash of the command before it.** The crash detector read every Admin log modified in the 2 s before the run, so a `lic-info` that had just crashed made the next `convert-meteo` answer `success:true` with a `crash` field. It now looks only at logs created since the run started. Checked live while verifying BUG-07: with a deliberately invalid date, `convertir_meteo` reports `Invalid date Line` and `costo.ejecuciones:1`, and the documented Hawaii example converts and also costs 1.
- **`sitio crear`, `sitio convertir_meteo`, `sitio meteo_pvgis` and `disenar_sistema` refuse a workspace PVsyst does not have configured (BUG-05, 07, 08)**: PVsystCLI `create-site` and `convert-meteo` crash with `Instruction privilégiée` on any workspace other than the `DataPath` of `PVsyst.ini`, the same cause as `simular ejecutar` (BUG-06). The check that `simular` already had is shared and runs before the NASA lookup, the PVGIS download and any file is written, dry runs included. Checked live: the same `sitio crear` crashed on a temporary workspace and created the site on the configured one.
- **GUI inventories are not summarised to 13 items.** `que:inventario` and `que:elementos` cap
  themselves at 500 rows (`total` and `truncado` say what was left out) and their lists `controles`,
  `elementos` and `otros` are never cut by `detalle`. `que:controles` with `todos:true` keeps up to 500
  rows when the call names no `detalle`; an explicit `resumen` or `normal` is honoured there.
- `que:inventario` and `que:elementos` say when the UI Automation tree was cut (`uia_truncado`, and
  `truncado` in `elementos`), and an element at the depth limit is no longer reported as `solo_vision`
  just because its children were not walked.

### Added
- **`componente crear_desde_ficha` of a `.PAN` fills the module size (BUG-14)**: `components.panels`
  gains `longitud_mm`, `anchura_mm`, `espesor_mm` and `peso_kg` (page «Dimensiones y tecnología», anchors
  read live on PVsyst 8.1.8 with `gui_observar que:inventario`), saved by PVsyst as `Height`, `Width`,
  `Depth` (in metres) and `Weight`. The datasheet takes `Height` and `Width` in mm, as the window does,
  and refuses the call before opening PVsyst when they are missing (PVsyst does not save a module whose
  size is not defined: it vetoes the tab change); `Depth` and `Weight` are optional. Checked live on
  8.1.8 through `gui_actuar rellenar`, all four fields read back `ok`. The Comercial page has no
  required field. Fixture: `tests/fixtures/vista_agente/Modulo_con_dimensiones_8_1_8.PAN`.
- **`pvsyst://gui` describes its own map** in `metadatos`: `measured_on` (date of the last walk
  recorded in `docs/gui_mapa.json`), `pvsyst_version` it was measured on, `idioma_medido`,
  `windows_total`, `cobertura` (windows with fields mapped, controls without a handle, limits) and
  `pvsyst_version_en_ejecucion`, read from the install folder of the GUI executable; when the two
  differ, `aviso` says the tabs, anchors and menu positions may have changed and that the live
  inventory is what holds.
- **`gui_observar que:elementos`** lists the rows, cells and items a control draws itself
  (`TVirtualStringTree`, grids, lists), which have no window and are not in the inventory. They come
  from the UI Automation tree under the control named by `ventana` (a control's handle, or a form for
  all of it): `indice`, `rol`, `nombre`, `rect`, `patrones` (now also `contenedor` and `virtualizado`,
  the item-container and virtualised-item patterns) and `hwnd` when the element is a window. A control
  with none is `solo_vision:true`, with the actions that still reach it (`fila`, `menus`/`menu`,
  `clic_texto`). Only the elements the control has realised are listed: a virtualised row scrolled out
  of view is not realised through its container. MSAA (`IAccessible`) is not read.
- **`gui_observar que:ventanas` says whether a click would land**: every window carries `foreground`
  (it is the foreground window), `modal` (it is the dialog that blocks the others; one embedded in the
  form it disables is listed too) and `iconic` (minimised, the name Win32 gives `minimized`), next to
  `visible` and `enabled`.
- **`gui_observar que:formulario` with `todos:true`** also lists the controls that are not shown (other
  tab pages, hidden panels), and every answer gains `otros`: each control that is neither a button nor a
  field (`TPageControl`, `TTabSheet`, `TGridPanel`, `TVirtualStringTree`, custom classes) with its
  `clase`, `hwnd`, `rect`, `texto`, `visible` and `enabled`. Buttons and fields now carry `visible`,
  `parent`, `pestana` and `indice_pestana`, and fields carry `etiqueta_fuente` (`ocr` or
  `window_text`). A control that is not shown is read without OCR, since the text drawn where it sits
  belongs to the page that is shown; when OCR finds no label, the text of a window beside the field
  names it.
- **`gui_observar que:inventario`**: every control of a form as a machine-readable list, hidden
  children and tab pages that are not shown included (`todos` defaults to `true` here), without
  switching tab. Each row carries `clase`, `hwnd`, `rect`, `texto`, `visible`, `enabled`, `parent`,
  `pestana` and `indice_pestana`, `rol`, `indice_clase`, `etiqueta` with `etiqueta_fuente`, `marcado`
  for check boxes and options and `campo {clase, x, y}` (the 96-DPI anchor `escribir` and `campo`
  take). Classes nobody classified (`TGridPanel`, `TVirtualStringTree`, custom buttons) are listed
  with `rol:"other"`. `pestanas` lists the pages with the controls each holds, and flags
  `creado_al_activar` on a hidden page without windows: VCL may build its controls only when it is
  first shown, so such a page can look empty until then. At most 500 rows per answer, with `total`
  and `truncado`. UI Automation is read too (`incluir_uia:false` skips it; `profundidad` and
  `max_nodos` bound its tree) and lands in the same list: a row says which API saw it (`fuente`:
  `win32`, `win32+uia` or `uia`), a window gains the `patrones` it supports and, for a field, the name
  UI Automation gives it (`etiqueta_fuente:"uia_name"`); the cells, labels and items a control draws
  itself, which have no window, are listed with `hwnd:null`, `visible:null` and `solo_vision:true`
  when only vision can read inside them. The tab order comes from UI Automation. MSAA (`IAccessible`)
  is not read.
- **Configurable client-facing language** via `PVSYST_MCP_IDIOMA` (`es` or `en`, default `en`; also a
  user-configurable option of the mcpb bundle). It selects the language of every client-visible
  string: tool descriptions and titles, schema annotations, recipes (resource and prompts),
  `pvsyst://` resource names, the server `instructions` of `initialize` and the runtime messages
  (missing arguments, wrong types, out-of-range values, alias conflicts, unknown tool or resource,
  licence instructions, workspace rejections and approximation warnings). Protocol identifiers
  (`accion`/`que` values, argument names, `detalle` values) stay as they are.
- **`vista` block on every tool answer** (all 15 tools, every `accion`/`que`, errors included):
  `estado` (`ok|parcial|error`), `resumen`, `campos` with stable keys, unit and origin (`pvsyst`, `mcp`,
  `aproximacion_mcp`), `avisos`, `siguiente` as `{tool, argumentos}` that the target tool accepts as
  written, and `recortado` (what `detalle` cut and how to ask for the rest). A failed call carries
  `error {codigo, motivo, causa_probable, que_hacer, claves_validas}`. Existing keys are unchanged;
  `structuredContent` of an error gains `vista`. A flow that fails halfway (`ok:false`, `errores`) is
  `parcial` or `error` and suggests a diagnosis instead of the results of a simulation that never ran.
  The per-action summaries and next steps live in `src/vista_agente/contrato/catalogo/`; a test fails
  when a tool declares an action without an entry. The top-level `siguiente` hint now reads from that
  catalogue.
- A trial-mode confirmation (`requiere_confirmacion`) is `parcial` and its only `siguiente` is the same
  call with `confirmar:true`; a dry run (`dry_run`, `simular`) says nothing was applied and offers the
  real call. Clients without `structuredContent` get the failure's `vista` as a second text item.
  Contract, size measurements and the tool × action matrix: `docs/vista_agente_salidas.md`.

### Fixed
- More findings of the same QA round:
  - **BUG-19**: `gui_actuar accion:campo` takes `indice` (from 0, top to bottom) and `clase` to pick one
    of several fields that share a label (the three «Núm. de inversores»); the error for an ambiguous
    label now says so.
  - **BUG-15**: `uia_invocar` refuses a form that a modal dialog disables and names the dialog, instead of
    waiting 10 s and answering about another window (`paso` is unchanged: it can target the modal).
  - **BUG-17**: `entorno licencia` asks `lic-info` once more when PVsystCLI answers it with nothing.
  - **BUG-20 and the GUI rows of BUG-12**: the schema of `gui_actuar` lists what `menu` needs (`id`, or
    `menu` and `item`) and that `uia_invocar` needs `ventana` and `operacion`.
- Four findings of the exhaustive QA of 2026-11-11 (`docs/INFORME_PRUEBAS_EXHAUSTIVAS_2026-11-11.md`):
  - **BUG-22**: `proyecto p50_p90` lists its ten stored fields in `vista.campos`; a nested object of up to
    ten scalars is a group of values (the twelve months of a year stay a table).
  - **BUG-21**: `gui_actuar accion:fila` needs no `clase` when the form has exactly one control whose row
    geometry was measured; with several it asks for `clase` or the control's `hwnd`.
  - **BUG-18**: `gui_observar que:leer` honours `ocr:true` and adds `ocr`, the lines drawn in the control
    (it was silently ignored); without it nothing is captured.
  - **BUG-16**: the existing-workspace, unknown-component-kind, window-not-found and object-not-found
    rejections, and the «no visible window to inspect» one, follow `PVSYST_MCP_IDIOMA`.
- QA of `pvsyst_cli` and the CLI adapter (`docs/INFORME_REVISION_PVSYST_CLI_2026-11-07.md`,
  `docs/INFORME_PRUEBAS_MCP_2026-11-07.md`), each finding reproduced first:
  - **No false confirmation**: an explicit CLI error (`Error: Invalid command syntax`, `Unknown command`,
    `Unknown command line option`) is an `error` even for `help`/`version`/`lic-info`, whose exit code is always 0;
    command names are case-insensitive (`VERSION`, `Help`); `export-logs` to a missing folder and
    `convert-meteo` with an unreadable date are errors, not `ambiguo`.
  - **A crash is an error**: a run whose worker thread dies exits 0 and prints only the license banner; the
    exception PVsystCLI logs (`EPrivilege`) is read and reported as `crash:<mensaje>` instead of `ambiguo`.
  - **The passthrough stays out of the licence and off the key**: `lic-activate`, `lic-deactivate` and
    `lic-sync` are refused (the `entorno` action that returns the command is named instead), and the
    displayed command line redacts `-k`/`--key`/`-c`/`--customer-id`, quotes the executable and escapes
    quotes, tabs and trailing backslashes.
  - **Rehearsals that cannot run for real**: `dry_run:null` is refused instead of read as `false`; the
    command name is trimmed and refused when blank or carrying spaces; `opciones` refuses blank or repeated
    items and non-scalar values, and its object form accepts `-w`-style keys, valueless `true` flags and
    omits `false`/`null`.
  - **Output no longer mutilated**: `help` keeps the lines that mention `lic-activate`, a `%` line of a
    command that is not a simulation is kept, the answer says when the 60-line tail was cut
    (`stdout_tail_truncado`) and `detalle:completo` returns the whole stdout.
  - **Timeouts, cancellation and paths**: `run-simulation` defaults to 900 s (120 s for the rest) and a
    `timeout_s` outside 1–3600 is an error instead of a silent clamp; `pvsyst_cli` honours
    `notifications/cancelled` and frees the single worker; a relative output path resolves against the
    workspace; `PVSYST_CLI` must be a file and a launch failure names it.
  - **Trial budget**: `pvsyst_cli run-simulation` reports `costo_estimado` in a rehearsal and records
    `costo` in the ledger, and the ledger no longer trusts a counter that goes back up (it was measured
    jumping between 133/134 and 249/250 without a licence change).
  - `log_level` is passed as `-ll:<n>`; `via` is listed in `ignorados`; the tool declares
  `openWorldHint:true` (`version` checks for updates); the wait loop no longer copies the whole stdout when
  nobody listens for progress, finished background jobs are bounded, and a non-UTF-8 console byte is decoded
  as its code-page character instead of U+FFFD.
- Review of the PR (Haiku review, `docs/REVISION_HAIKU_HALLAZGOS.md`), each finding reproduced first:
  - **Confinement**: project names that are not plain relative paths (`\Users\x`, `C:x`), RVT templates, `simular`
    `output_csv`/`report_pdf`, `sitio crear` `weather_data_file`/`cmd_options_file`/`output_sit_file`, horizon and series lookups and
    DataRO fallbacks stay inside the workspace unless `permitir_fuera_del_workspace`; `sitio` of `proyecto crear`, history ids,
    insertion roles and written file names (`CONIN$`, `CONOUT$`) are validated; variants are `VCn`/`CMn` everywhere.
  - **Refused input instead of silent changes**: `extra_options` that redefine a flag the MCP builds (`-w`, `-p`, `-v`, `-ocf`, `-s`,
    …); batch fields with `;` or line breaks; non-text `opciones` of `pvsyst_cli`, `variables` of RVT and series, `variantes` of
    `resultados comparar`; a non-numeric `PNomConv`; invalid `offset`/`limite`; repeated series variables; `editar` of a component
    without `cambios`. `sitio crear` needs its weather file to exist (as `convertir_meteo` already did).
  - **Partial work is reported**: `rellenar` keeps the fields typed before a failure and adds `campos_que_difieren`, `completo:false`,
    `errores` and `formulario_error`; `gui_actuar accion:funcion` says when the expected window never opened; PVsyst sizing and horizon
    import compare what was saved; `crear_desde_ficha` stops before saving a datasheet that was not typed as asked and lists
    `diferencias_con_la_ficha`; a legacy MEF without readable variables is `completo:false`; a deliverable lists `secciones_omitidas`.
  - **No data loss on failure paths**: `disenar_sistema` undoes only its own project's files; `proyecto crear` writes the site into the
    PRJ before the pair is written (and in the call's workspace); `doctor` keeps checking after an unreadable variant; exports of a
    component or a project need `sobrescribir` to replace a file; `PVsyst.ini` and series files keep bytes outside UTF-8; truncated
    files are refused; folders are listed without dropping unreadable entries.
  - **Processes and cache**: PVsystCLI is always killed and reaped on an early exit; reader threads and `tar` stderr no longer block;
    a background job is recorded before it starts and a panic is its error; default outputs are cached and restored; `-s` enters
    the cache key; a damaged cache entry or ledger no longer breaks a run or skips the trial confirmation.
  - Simulation KPIs of `disenar_sistema` no longer carry an always-null `kwp` (the design answer has `kwp_realizado`), and a PAN created
    by approximation no longer carries an always-empty `ignorados`.
- Review of the PR (cloud ultrareview), each finding checked against the code first:
  - A `gui_actuar accion:paso` that asks for no verification (`verificado:false`) is `ok` again; only a scene answer that counts
    `campos_que_difieren` is partial.
  - `escena_importar simular:true` answers `simulado:true`, so the contract offers the real call as the next step; `archivo` is made
    absolute (PVsyst's dialog resolves a relative name against its own folder); on a scene that already has objects PVsyst asks
    «¿Desea despejar la escena de sombreado actual?» and the answer is «No» (the import adds). Measured live: 66 tables into a scene of 3 → 69.
  - `escena` refuses an editor that already holds objects (it types positions into rows 0..N); the check exports the scene under a
    throwaway name and deletes it. An empty scene exports fine (`total_objetos:0`).
  - A failed step of `escena`, `escena_importar` or the export drops the open dialog («Cancelar», and «No» to «¿mantener
    modificaciones?») instead of leaving PVsyst behind a modal. Measured: a `unidades` PVsyst does not offer closes `TImportResultForm`.
  - `rellenar` applies check boxes and options first and reads the form again, so a field they enable can be filled in the same call;
    after a failure on another tab it returns to tab 0.
  - `proyecto dimensionar` accepts PVsyst's own rounding of the planned power (12.345 reads back 12.3).
  - The inverter phase of a datasheet goes through the schema options (`fase_*`), not a click by caption.
  - The PVsyst install lookup takes major 8 only (the GUI map was measured on 8.1.x); a PVsyst 9 needs `PVSYST_GUI`/`PVSYST_CLI`.
  - `proyecto dimensionar`, P50 and horizon are refused on the isolated desktop before PVsyst is touched: loading a project and saving its
    variant now go through menus, which need the keyboard (they used to click the toolbar with an emulated mouse; not measured on
    the isolated desktop, `tests/aislado.rs` has never run).
- Review of the PR (code-review), each finding checked against the code first:
  - `escena_importar`: a failure while opening the dialog or waiting for «Importar resultado» closes whatever it left open (file dialog,
    PVsyst's error box); an error box ends the wait at once instead of after 120 s. The protected-folder check compares path components of
    the absolute and the real path (`PROGRA~1`, junctions), not a substring of the text.
  - `rellenar` never clicks a check box whose state it could not read, and refuses an option asked as `false` before typing anything.
    A number reads back as `ok` only within half a unit of the last decimal PVsyst shows (12.345 reads 12.3; 100 read as 100.9 is
    `distinto`); counts are exact. The datasheet flow (`componente crear`) and the project steps fail, listing every field PVsyst did
    not take, instead of saving or sizing on top of it.
  - `gui_actuar accion:teclas` on a child control works again on the isolated desktop (`foco:false` if the control did not take the focus).
  - Scene text files: grouped objects count one by one, a file cut off while PVsyst writes it is refused, and a missing `total_objetos`
    is not read as an empty editor. `describir_escena` lists at most 1000 colliding pairs (`colisiones_total` counts all).
  - The install folders of PVsyst are listed once per process, not on every request that needs `DataRO`.
  - `escena_importar`: if «Importar resultado» does not close after «OK» or «Cancelar» (or the click fails), the form is dropped with
    «Cancelar» and, failing that, closed, and the error says whether it is gone. `componente crear` retries the focus of a save dialog
    once and logs when it still is not there. Not measured with PVsyst.
- Review of the PR (ultrareview #2), each finding checked against the code first:
  - `disenar_sistema` refuses a stand-alone design without `n_mod_serie` and `n_strings` in the preflight, instead of failing in the
    sizing step after the site and the project already exist.
  - A reused project reports whether the call compared anything: `proyecto_reutilizado:true` with `proyecto_comparable:false` means the
    call asked for no module, inverter or string layout. `proyecto_comparable` is `null` when the project was created.
  - `gui_actuar accion:cerrar_todo` counts each window once, when it is confirmed gone, instead of counting every attempt of every round.
  - `simular ejecutar` no longer lists `variables`/`unidades` in `ignorados` when `filas`/`parametros` make the handler read them. The
    ignored-keys table is per key, with the argument that makes it read, and a test fails when it drifts from the tool declarations.
  - `subarrays_de` reads the `pvSubArray` spans instead of building a second full tree, and `aviso_meteo_distinta` reads the PRJ line by
    line until `MeteoFileName` instead of loading and parsing the whole file for one value.
- **`disenar_sistema` sizing no longer loops over billions of candidates** when a database file carries an absurd voltage (a tiny `Voc`, a
  huge or non-finite `VAbsMax`): the series search stops at 1000 modules and a non-finite voltage finds no layout.
- **`http_get` (PVGIS, uncertainty lookups) refuses a body over 64 MiB** and stops reading it instead of buffering whatever the server sends.
- A `tools/call` that arrives as a notification and fails is logged; a failed rollback of `disenar_sistema` is logged instead of ignored.
- **GUI tools, QA round of 2026-11-08** (GUI-01..15, BUG-03/05/16/17/23/24):
  `escribir`/`seleccionar` read `campo{x,y}` in the coordinates `vista` publishes (from the window corner, not the client area), so a
  key like `edit_75_121` hits the field it names; `uia_invocar` waits at most 10 s for a pattern call and answers `retorno:false` instead of
  blocking the server when PVsyst opens a modal; `esperar_cierre` with `cerrado:false` and a `clic_texto` without OCR match are `parcial`;
  `click` refuses `texto`/`id` and disabled controls, resolves a whole title or class before a part of one, and `cerrar` names the
  confirmation it left open; `clic_fisico` honours `objetivo:{x,y}` (a screen point on the window); `leer` returns the value of an edit;
  `teclas` refuses to send when no control of the window holds the focus; `escena_importar` resolves a relative mesh in the workspace; the
  modal error names the dialog, not the UAC overlay; `menus` lists the bar without the system menu and names catalogued entries;
  `controles` `count` is the number returned; `vista` names the open windows that have no schema; `lanzar proyecto` loads the project
  through PVsyst's own window (PVsyst 8.1.8 ignores a project on its command line); `proyecto p50_p90` and `dimensionar` wait for
  PVsyst to finish writing the variant instead of reading it half written.

- QA of the MCP (`docs/INFORME_PRUEBAS_MCP_2026-11-07.md`), each bug reproduced first:
  - `resultados perdidas/mensual/entregable` take the array kWp from the report's own `Pnom total` (else from the produced energy and
    the specific production) instead of the variant's `PNomSizing`, so the deliverable's specific production matches PVsyst (BUG-01).
  - `disenar_sistema` reports PVsyst's own words in `pasos[simulacion].detail` when the run fails for a cause the MCP does not know
    by name (BUG-04).
  - `modulo` is accepted with or without the `.PAN` extension (BUG-07), and a dry run refuses a module no database holds before the
    site and the project exist.
  - The refusal for a design without `n_mod_serie`/`n_strings` names `via:'aproximacion'` only when the schema offers it, and the
    environment variable that enables it otherwise (BUG-08); `proyecto_reutilizado:true` with `proyecto_comparable:false` is reachable
    through the tool (BUG-06).
  - `sitio crear dry_run` reports `costo_estimado`, the trial executions the real call would spend (BUG-20).
  - `simular preparar` suggests running the batch it prepared, carrying `batch_params_file`/`batch_rvt_file` (BUG-22).
- **`escribir_archivo` no longer overwrites by default**: an existing file is replaced only with `sobrescribir: true`, so the
  argument means what it says; the refusal names the alternative and the history keeps the previous version.
- **`editar campos` no longer invents fields**: a key the object does not have is refused (the usual cause is a typo) unless
  the call asks for it with `nuevo: true`; the same policy applies to the `ediciones` form of a variant.
- **`componente crear` copies the template when the call asks for no `cambios`** (the description already said "from a
  template"), and the schema declares `campos`, the spelling the handler already accepted: a test fails when an action
  consumes a key its tool does not declare.
- **`sitio escribir_horizonte` writes the `.HOR` in `Shadings/` under its own name**: a path in `archivo`
  (`Sites/X.HOR`) no longer creates a folder inside `Shadings`, which is not where PVsyst looks for a horizon.
- **`sitio crear` from a CSV reports the `.SIT` it expects** (`output_sit_file`, else `Sites/<name>.SIT`) instead of
  `expected_sit:null`; both origins now use the same rule.
- **`entorno preferencias` reports `n_secciones`** (and `n_claves` when a `seccion` is asked for), so `vista.campos` is
  no longer empty: the file path and the section table were the only other values.
- **"Not found" messages no longer say something false**: `no_encontrado` names the options it has instead of
  "no hay ninguno" when it was given a list, `leer` names the projects of `Projects/`, `proyecto crear` says in
  Spanish that the project exists and names `sobrescribir`, `resultados comparar` says how many variants the
  call asked for, and `editar insertar_objeto` names `desde.archivo` instead of the top-level `archivo`.
- The `leer` hint points at the parameter that exists (`detalle:"completo"`), not at `completo:true`.
- **`workspace exportar` no longer archives its own temporary**: a `destino` inside the exported tree put
  `<zip>.mcp_tmp_<pid>.zip` (0 bytes) in the archive; both the destination and its temporary are excluded, and
  `workspace importar` names the workspace a relative `zip` was resolved against when it finds nothing.
- **The schema declares every key an action requires**, so a client that trusts `Required by action` no longer sends
  a call the handler rejects: `componente crear` (`tipo`), `borrar`/`exportar` (`componente`, `destino`), `importar`
  (`origen`), `sitio crear` (`latitud`, `longitud`), `sitio convertir_meteo` (its four files), `simular ejecutar`
  (`proyecto`), `workspace crear` (`ruta`), `importar` (`zip`, `destino`) and `exportar_proyecto` (`proyecto`).
  Actions that take one of several keys say so and accept any of them: `simular preparar` (`filas` or `parametros`,
  unless `operacion` selects the RVT manager), `workspace exportar` (`destino` or `permitir_fuera_del_workspace`),
  `resultados kpis`/`resumen` (`proyecto`, `archivo` or `report_pdf`) and `editar campos` (`archivo`, or
  `proyecto`+`ediciones`). A missing key now fails with the missing-argument message before the handler runs, and
  `sitio weather_data_file` documents that it belongs to `origen:csv`.
- QA of the MCP (`docs/INFORME_PRUEBAS_EXHAUSTIVAS_2026-11-11.md`), each bug reproduced first:
  - `leer` with `offset`, `max_bytes` or `base64` finds a bare project name under `Projects/` like the project branch does
    (BUG-13); a name that already resolves keeps its file.
  - `entorno exportar_logs` creates an explicit `carpeta_destino` before calling PVsystCLI, which refuses a folder that does
    not exist (BUG-23); without it the CLI default (`<workspace>/UserData/logs`) is unchanged.
  - A legacy `.MEF` whose binary columns are not decoded is `ok` with the `aviso`, not `parcial` with an invented failure
    (BUG-26): `estado` is `parcial` only when the answer names what failed or differed, not for `completo:false` alone.
  - `gui_actuar accion:reiniciar` says in its description, README and refusal that it only applies after a PVsyst failure
    dialog and never restarts a healthy instance (BUG-27).

### Changed
- The six `proyecto` actions that edit a variant (storage, pumping, injection limit, losses, economics, tracker) build their answer with one
  function (`respuesta_de_edicion`); the identical range check of losses and economics lives in `args::rango_opcional`.
- **`proyecto dimensionar` and the active-variant choice no longer use coordinates.** The System window and the project window are
  schema entities (`variante.system`, `proyecto.ventana`) and the flow fills them by key with `rellenar`; the coordinate constants and
  `Formulario::{leer, marcar}` are gone. Measured on PVsyst 8.1.8: 50 kWp gave 28 modules in series × 4 strings, saved as `NModSerie=28`,
  `NStringCh=4`, `PNomSizing=50.0`.
- `rellenar` no longer reports an option or a check box as "recalculated" when only its caption was read.

- **The project window's bar clicks are gone**: «Cargar proyecto» and «Guardar la variante activa» run their catalogued menu commands and
  the project list's first row is chosen with `accion:fila`, instead of clicking fixed offsets of a toolbar. `Formulario::clic_barra`
  is deleted.

### Added
- **`variante.orientation`** (`TOrientationForm`, fixed plane), **`variante.system`** and **`proyecto.ventana`** entities for
  `que:vista`/`rellenar`, checked against a variant PVsyst 8.1.8 saved (`FieldTilt`, `FieldAzim`, `NModSerie`, `NStringCh`, `NInverter`,
  `PNomSizing`). New field type `casilla` (`true`/`false`) for the options of a group; an option is cleared by choosing another.
- **Conditional fields**: `CampoEsquema::si` declares the check box a field depends on (`depende_de` in `que:esquema`); the
  inverter schema models its check boxes and phase/frequency options (`definir_corriente_ca_nominal`, `fase_trifasica`, ...), and
  `que:vista` lists the anchor of every unmapped control (`sin_mapear`).
- **`que:vista` of a window without a schema returns `abierta_con`**: what opened the instance whose title the map recorded, so the 15
  windows of class `TChoixObjetPVForm` (and the 4 `TDetailedLossesForm`, 3 `TDimensForm`) are told apart by title. `TDateTimePicker`
  is now read (checked live); `TListBox` is deliberately not (it would list the component databases through the GUI).
- **Crear menu of the 3D editor catalogued**: `TTrackerFieldForm`, `TShedFieldForm` (shared by tables, domes and sun shields),
  `TRectFieldForm` and `TPolyFieldForm` open from `escena_3d.crear.*`, with every tab read (map: 89 classes).
- **`gui_actuar accion:escena_importar`** (`archivo` + `escena`, optional `unidades`, `simular`) imports a 3D mesh (3DS, DAE, PVC, PVC2,
  PVZ2) into the scene open in the editor with PVsyst's own «Importar una escena 3D»: the menu entry, the Windows file dialog and
  «Importar resultado». PVsyst converts the mesh; the MCP exports the scene afterwards and returns it. Measured on PVsyst 8.1.8: the
  sample `SHD_PVC_Fixed_Example.pvc` gives 66 PV tables, 2 orientations and a group, as the dialog announced. `simular:true` reads the
  file and drops the import with «Cancelar». A path under `Program Files` is refused before PVsyst is touched (PVsyst refuses it).
- **`gui_actuar accion:menu` takes `subitem`** for an entry of a submenu (`menu` 0, `item` 4, `subitem` 0 = «Importar ▸ Importar una
  escena 3D»): the id is read from the submenu's own `HMENU`, so the submenu is never entered.
- **`gui_observar que:esquema entidad:escena_3d.formas`** describes the 27 shapes of the 3D editor (PVsyst type, combo entry, each parameter
  with label, unit, default, range) before the agent asks for a scene; the unfiltered listing points to it (`otras_entidades`).
- **`rellenar` fills several tabs** (`en_pagina` in the schema: tab 0 first, the others in order, back on tab 0). The inverter's
  efficiency fields live on tab 1.
- **`componente crear_desde_ficha` types by schema keys** instead of coordinates (`components.panels`, `components.inverters`; the
  datasheet key is the field's `clave_archivo`). On PVsyst 8.1.8 the old coordinate flow did not save: the «Guardar» button of the
  save dialog only answers when the file-name field has the focus, and the file name must not carry the extension. Checked live: a
  module (`Technol=mtSiMono`, `NCelS=72`) and an inverter (efficiencies on tab 1, `MonoTri=Tri`) saved. PVsyst recalculates
  `PSeuil` (0.00) when it builds the efficiency profile.
- **`rellenar`/`vista` entity `components.batteries`** (battery window, «Datos básicos», 10 fields), checked against a `.BTR` PVsyst
  saved. The file keeps the internal resistance in ohm and the coulombic efficiency as a fraction, the window shows mOhm and %.
- **`rellenar`/`vista` entity `components.inverters`** (grid inverter window, «Parámetros principales»): 15 text and number fields with
  units and file keys checked against an `.OND` PVsyst 8.1.8 saved (the trial licence saves components). Ranges are the MCP's.
- **`gui_actuar accion:escena_leer`** (`escena`) lists the objects of the scene open in the 3D editor. The table of
  objects has no controls per row, so PVsyst exports the scene with «Exportar escena (.SHD)» under that name and
  the file is read back (`origen:pvsyst`); the file stays in `Shadings`.
- **`gui_actuar accion:escena` builds a 3D scene in PVsyst's editor.** Tried on PVsyst 8.1.8 in PRUEBA, which
  does save scenes (`Archivo > Exportar escena (.SHD)` wrote the file; the project was not touched). With
  `simular:true` it only validates the request (`escena`, `objetos`) without PVsyst: types, ranges and
  collisions. Without it, with the editor open on an empty scene, it creates each object in the elementary-object
  dialog, sets the position in «Lista y gestión de objetos», has PVsyst save the scene and compares the saved
  file with the request, field by field (`comparacion` rows `{objeto, clave, pedido, leido, estado}`,
  `verificado`). The MCP never writes a scene file. Up to 20 objects (the table shows 20 rows) and the scene
  name must not exist in `Shadings`.
- **The 27 shapes of the elementary-object dialog** (`oeTree`, `oeHouse`, …) with their PVsyst type name, the
  label of each parameter and its position in `Params=`, measured live and proven against a scene PVsyst saved
  (`tests/fixtures/escena_3d/Formas27_8_1_8.SHD`), and again with a different value in every parameter of the 27
  shapes (0 differences); the 11th value of `turbina_eolica` and `pilon_electrico` is `pala_angulo_deg` and
  `espesor_malla`. The `cable` dialog has an extra «Objeto delgado» edit, which is left as is. Ranges are
  the MCP's, not PVsyst's. Breaking change for the
  unreleased `escena` request: dimensions now use PVsyst's own parameters (`anchura_dx`, `longitud_dy`,
  `altura_dz`, tree `altura_punto_medio`…) and the `tejado_dos_aguas`/`chimenea`/`arbol` keys of the first draft
  are gone.
- **`leer` reads the text form of a scene** (`PVObject_=pvShading`, UTF-8 with byte-order mark) that PVsyst 8.1.8
  saves; before, such a file failed with «Incoherencia entre extensión y clase raíz». The geometry reports
  `tipo_pvsyst` (`oeParallelepipede`).
- **Window coverage:** every one of the 82 window classes of `docs/gui_mapa.json` is registered in one of four
  states and listed in `gui_observar que:esquema` (`cobertura`). `que:vista` on a window without a schema
  returns its fields by control class and 96-DPI position (combo items included) or its buttons.

- **`gui_actuar accion:fila`** (`ventana`, `indice`) selects row N of a list or tree that has no window per
  row, with a physical click on geometry declared per control class (`TVirtualStringTree`: header 21 px, rows
  19 px; `TListBox`: border 2 px, rows 13 px; the object table of the 3D editor: header 32 px, rows 26 px, measured on
  PVsyst 8.1.8; px at 96 DPI, scaled by the window DPI). It checks the result
  without OCR: the highlighted band of a capture must be the requested row, and the click must not have opened
  or closed anything. A wrong row, an empty slot, an out-of-range index, a click PVsyst took as "open this item"
  and a control without measured geometry (class and hosting window) are errors. The window must be in front and
  the control focused before the click, the click point must lie inside the control, and the band is read by
  comparing the capture before and after; a window the click closed is an error too. It only selects (no Enter, no OK) and it is
  refused on the isolated desktop (not measured).
- **`teclas` with a control as `ventana` gives it the keyboard focus first** and fails when it cannot
  guarantee it, so an arrow no longer lands on a combo by accident. `fila` does the same before its click:
  with the focus on another control the first click only moved the focus.

- `gui_observar que:esquema` lists, per window class, `abierta_con`: the keys of the catalogue functions that
  open it (`accion:funcion` opens them). `herramientas.factor_transposicion` now declares the window it was
  seen to open (`TArrayCharactForm`, not `TTranspositionForm`).

### Fixed
- The PVsyst install is no longer pinned to `C:\Program Files\PVsyst8.1.7`: `PVsystCLI.exe` and `PVsyst.exe` are
  looked up in the newest `PVsyst<n>.x` folder (n ≥ 8) under Program Files, Program Files (x86) and
  `%LOCALAPPDATA%\Programs` (`PVSYST_CLI`/`PVSYST_GUI` still win). Seen on 8.1.8 (build 49527): `THomeForm`,
  `TChoixObjetPVForm`, `TPVModuleForm` (16 fields) keep their classes; still TRIAL.
- `vista.campos` gave a unit to text values whose key ends like one (`frente_a` → `A`); only numbers get a unit.
- `vista.estado` no longer says `ok` for a PVsystCLI run with `success:false`, a timeout or a `detected_error`
  (error code `fallo_cli`), nor for an answer with `completo:false`, `error` or `errores` (`fallo_parcial`). A run
  that produced its output file only warns about its process error.
- A dry run (`dry_run`/`simular`) is read from the answer, not from the request, and is refused up front in the
  actions that have none (`entorno preferencias_editar`, `sitio escribir_serie`, `simular preparar`, `gui_actuar
  cerrar_todo`...) and in `simular ejecutar` with `filas`/`parametros`, which write files before running.
- `gui_actuar accion:escena` is validation only and no longer fails for lack of an undeclared `simular`; the
  object limit (500) and a minimum of one are enforced at runtime.
- The trimmer keeps a `<clave>_total` the domain already counted, `vista.recortado` lists real cuts only, and
  scene collisions are no longer capped by the handler.
- `vista.claves_validas` names the keys the runtime found missing; `teclas` validates its keys before looking
  for windows, sends them only to a window of the target process and checks it is in the foreground.
- `vista.campos` is capped for forms too; component steps list the kind created; duplicate keys are dropped.
- `herramientas.factor_transposicion` declares `TArrayCharactForm`, the window it was seen to open.

### Changed
- `gui_actuar accion:rellenar` returns the form read back under `formulario` (it was `vista`, which now
  belongs to the answer block).
- **GUI `teclas` sends HOME, END, PGUP/PAGEUP and PGDN/PAGEDOWN** to move through lists and trees.
  DELETE, DEL, SUPR, INSERT and INS are refused with a message, alone or with a modifier: in PVsyst lists
  they can remove or insert items.

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

### Added

- **History covers creations.** `escribir_archivo`, `escribir_horizonte`,
  `escribir_serie`, `editar`, `componente`, `proyecto` and the batch writers take
  the snapshot before writing, so the first version of a new file is in
  `historial` (`historial_id` is not null) and `workspace restaurar` can bring it
  back. The snapshot of a file that did not exist records `existia:false`.
- **`workspace restaurar` accepts `dry_run`:** it lists what it would restore and
  writes nothing (`simulado:true`). Before, it restored for real and listed
  `dry_run` in `ignorados`.
- **`gui_actuar accion:fila` takes `clase`** and selects rows in the three
  controls whose geometry was measured (project tree `TVirtualStringTree` of
  `TProjectLoadForm`, module chooser `TListBox` of `TChoixObjetPVForm`, object
  table `TVirtualStringTree` of `TShdObjectSelectionForm`). Any other control is
  refused, naming the measured ones.
- **`disenar_sistema` resumes:** a project an earlier run left behind with the
  same parameters is reused instead of failing with `Project exists`.

### Fixed

- `editar` refuses object and array values in `campos` instead of writing their
  JSON text into a field PVsyst reads as one value.
- `escena_leer` exports the scene under a temporary name and deletes that file
  before answering, so a later `escena_leer` or `escena` with the same name no
  longer fails with `already exists`.
- `reiniciar` verifies that the process ended after its dialog button and
  escalates to a forced kill; `muerte` says which one ended it.
- The three GUI routes (`proyecto dimensionar`, `proyecto p50_p90`,
  `sitio horizonte_pvgis`) load the project they act on, report the cause PVsyst
  logged instead of the symptom, and drop the benign `TEnergyMixSet`/`EndTags`
  parse notices.
- `gui_actuar accion:teclas` does not send keys on the user desktop when the
  control cannot take the focus; the isolated desktop reports `foco:false` and
  sends them.
- `esperar` re-checks once (1.5 s) before reporting a timeout, so a slow load
  that ends just after the deadline is not a false negative.

### Changed

- **The `initialize` instructions were rewritten** with 12 facts checked against
  the runtime: components by name, key names, window waits, `resultados
  que:resumen`, where `dry_run` exists, truncation, the `workspace` override,
  the modal `TDEMOForm`, the measured rows and `ignorados`.
- Tool descriptions say what the runtime does: `componente` by name,
  `pvsyst_cli` arguments in `opciones`, `resultados que:resumen`, `simular`
  `dry_run`/`costo_estimado`, `esperar` as a window wait, `fila` in the measured
  controls, `disenar_sistema` module choice/resume/Meteonorm, and the `workspace`
  override says what it replaces instead of asking for a trusted client.
- `mcpb/manifest.json` and the alias tables of both READMEs are generated from
  the registry by `tools/generar_contrato.py` (`--check` fails on a stale copy),
  and a test fails when the manifest drifts from the registry.
- Two runtime messages of the server layer (`include_text` outside `que:csv`, and
  `dry_run` with `filas`/`parametros`) answer in English through
  `crate::idioma::Texto`.

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
