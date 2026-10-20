# Changelog

All notable changes to this project are documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.1.1] - 2026-10-19

### Added

- **npm package**: the release binary is also published as `pvsyst-mcp` on npm
  (Windows x64), so an MCP client can launch the server with `npx -y pvsyst-mcp`.
  The tarball bundles the license and the attribution notice.
- **GitHub Releases**: every `v*` tag builds the MSVC release binary in CI and
  attaches it as a zip together with `LICENSE`, `NOTICE` and the READMEs.

### Changed

- Unit tests moved from `src/` to `tests/unit/`, so the shipped sources contain
  production code only.
- The official branch is now `main`.

## [0.1.0] - 2026-10-11

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
