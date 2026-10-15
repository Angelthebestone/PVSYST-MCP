# Security Policy

## Supported versions

Only the latest release of `pvsyst-mcp` (0.1.x) is supported. Please reproduce a
report against the newest commit on the default branch before filing it.

## Reporting a vulnerability

Email **penaagamezangeldavid@gmail.com**. Do not open a public issue for a security
report, and do not include licence keys, customer IDs or private project data.

You can expect an acknowledgement within a few days and a decision (accept, ask for
more detail, or decline with reasons) within 30 days. If you want credit, say so and
say how you want to be named.

## Threat model

This is a **local, single-user** server. It speaks MCP over stdio and **opens no
network port** — nothing it does is reachable from another machine unless you expose
it yourself. Its outbound traffic is limited to the PVGIS endpoints used by
`horizonte_pvgis`, `meteo_pvgis` and the optional P50/P90 fetch.

Capabilities that matter when you decide which client to trust:

* **Filesystem confinement.** Every caller-supplied path (reads, listings, edits,
  import/export, capture output, CSV/report destinations, history restore) is confined
  to the workspace; `..` and out-of-workspace paths are rejected. Writes are
  snapshotted under `UserData/mcp_history/<id>/` before they happen, and restore
  validates the whole manifest first.
* **`permitir_fuera_del_workspace: true`** opts out of that confinement for a single
  call. Treat any caller that can set it as having filesystem write access.
* **The `workspace` argument moves the confinement root itself**, so an untrusted
  caller can redirect the server at another folder tree you can write to.
* **`PVSYST_MCP_EXPERTO=1` is a privileged mode.** It exposes `pvsyst_cli` (runs any
  `PVsystCLI` command with raw options) and the GUI tools (`gui_observar`,
  `gui_actuar`: synthetic input, window control, screenshots). Do not enable it for a
  client or a prompt chain you do not control.
* **The server never activates or deactivates the licence**: `entorno` with
  `accion: licencia_activar` / `licencia_desactivar` only returns the command for you
  to run yourself with your own key.
* **stdout is the protocol.** Nothing pre-existing is ever printed to it; logs go to
  stderr and are filtered with `PVSYST_MCP_LOG`.

## Out of scope

* Vulnerabilities in PVsyst itself, including `PVsyst.exe` and `PVsystCLI.exe` — report
  those to PVsyst SA.
* Bugs in an MCP client or model that drives the server.
* Anything requiring an attacker who already controls your machine, your workspace, or
  the MCP client configuration.
* Prompt-injection style attacks where the attacker is the model's own input chain: the
  mitigation is not to connect clients you do not trust, and to keep expert mode off.
