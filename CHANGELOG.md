# Changelog

All notable changes to this project are documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

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
