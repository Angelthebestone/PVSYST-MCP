# Changelog

All notable changes to this project are documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

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
