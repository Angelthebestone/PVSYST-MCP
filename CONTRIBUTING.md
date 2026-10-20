# Contributing to pvsyst-mcp

Thanks for taking the time to improve this project. It is an **unofficial** tool:
it is not affiliated with, endorsed by or sponsored by PVsyst SA, and contributions
have to respect that.

## Ground rules

* **Never commit PVsyst software, licence files or vendor data.** No `PVsyst.exe`,
  no `PVsystCLI.lic`, no `DataRO` binaries and no component database copied out of an
  installation. Tests run against the synthetic fixtures in `tests/fixtures/workspace`.
* **No decompilation.** `PVsyst.exe` is never opened in a disassembler. Knowledge of a
  file format comes from files the user owns and from the installed documentation.
* **No reimplementation of the simulation engine.** PVsyst does the physics; when a
  simulation is needed the code shells out to the documented `PVsystCLI` commands.

## Development setup

Windows, a recent stable Rust, and a linker (MSVC, or MinGW-w64 with the GNU
toolchain as described in the [README](README.md#requirements)).

```powershell
cargo build
cargo test
```

PVsyst is **not** required for the default test run: the hermetic tests use the
synthetic workspace in `tests/fixtures/workspace` and the `fake_pvsystcli` binary.
Tests that need a real installation or a running GUI are marked `#[ignore]`:

```powershell
cargo test -- --ignored     # consumes trial executions if you have no CLI licence
```

## Before you open a pull request

All of these must pass — they are exactly what CI runs:

```powershell
cargo fmt --check
cargo clippy --all-targets -- -D warnings
cargo test
python tools/check_deps.py
ruff check tools; ruff format --check tools; mypy --strict tools; pytest -q tools
```

## Code rules

Inherited from `agents.md`, and enforced by lints wherever possible:

* **Package by feature.** No `utils/`, `helpers/` or `common/` modules; a module is
  named after the PVsyst business area it serves. One concept per file, under 300
  lines — split into a directory module instead of growing a file.
* **Dependencies point inward** to `format/` (pure core, no I/O). The graph is
  checked by `tools/check_deps.py`.
* **Typed errors** (`src/error.rs`). No `unwrap`, `expect`, `panic` or `todo` in
  production paths; `unsafe` blocks need a `// SAFETY:` justification.
* **Behavior tests, not implementation tests.** Assert on observable input → output,
  not on mock call counts. Unit tests live beside the module, integration tests in
  `tests/`. Reach for `proptest` for invariants such as byte-preserving edits.
* **stdout carries the MCP protocol.** Never print to it; log to stderr with
  `tracing` (`PVSYST_MCP_LOG`, default `info`).
* **Comments explain why, not what.** Module docs and comments are in Spanish in most
  modules — match the file you are touching.
* **Line endings.** `.rs` files are LF. `tests/fixtures/workspace/**` is CRLF on
  purpose (marked `-text` in `.gitattributes`) because it mimics real PVsyst files;
  do not re-encode it.

## Commits and pull requests

* One concern per pull request, explained in terms of **why**.
* [Conventional Commits](https://www.conventionalcommits.org/): `feat:`, `fix:`,
  `refactor:`, `test:`, `docs:`, `chore:`.
* Do not add AI co-author trailers to commits.
* Add a `CHANGELOG.md` entry under `## [Unreleased]` when behavior changes, and
  update `README.md` when the tool surface, environment variables or requirements do.

## Licensing of contributions

This program is licensed under the [Apache License 2.0](LICENSE), and contributions
are accepted under the same terms (section 5 of the licence: inbound equals
outbound). By opening a pull request you confirm you have the right to submit the
work under that licence.

Two consequences worth knowing before you fork:

* **Attribution travels with the code.** Section 4(b) requires modified files to
  carry prominent notices stating that they were changed, 4(c) requires every
  copyright and attribution notice to be retained, and 4(d) requires the
  [`NOTICE`](NOTICE) file to be propagated. A fork is welcome; a fork that strips
  the attribution is a licence violation.
* **The name is not licensed.** Section 6 grants no trademark rights, so a fork must
  not present itself as this project or imply endorsement by its author.

Never commit a PVsyst binary, a licence file or vendor data: see the ground rules
above and the responsible-use section of the [README](README.md#responsible-use-licensing-and-trademark).

## Reporting bugs and security issues

* Bugs and feature requests: use the issue templates.
* Security: see [SECURITY.md](SECURITY.md) — please do not open a public issue.
* By participating you agree to the [Code of Conduct](CODE_OF_CONDUCT.md).
