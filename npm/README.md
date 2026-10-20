# pvsyst-mcp

**Model Context Protocol server that automates [PVsyst](https://www.pvsyst.com) 8.1.7** —
batch simulations, project and variant creation, component and weather handling, and result
extraction — by reading and writing **only PVsyst's data files** and running the **documented
`PVsystCLI` commands** as a black box.

This package wraps the native Windows binary (`pvsyst-mcp.exe`, x64) so it can be launched with
`npx`. The server itself is free software under the Apache License 2.0.

> **Unofficial.** Not affiliated with, endorsed by or sponsored by PVsyst SA. "PVsyst" is a
> trademark of its respective owner, used here only to describe interoperability. This package
> contains no PVsyst code and redistributes no PVsyst software: you need your own licensed
> installation.

## Requirements

* **Windows x64** (the GUI bridge uses the Win32 API) and **PVsyst 8.1.7** installed.
* Node 18 or newer, only to launch the wrapper.

## Use with an MCP client

```json
{
  "mcpServers": {
    "pvsyst": {
      "command": "npx",
      "args": ["-y", "pvsyst-mcp"],
      "env": {
        "PVSYST_WORKSPACE": "C:\\Users\\<you>\\PVsyst8.1_Data"
      }
    }
  }
}
```

Optional environment variables:

| Variable | Meaning | Default |
|---|---|---|
| `PVSYST_WORKSPACE` | PVsyst user workspace | `%USERPROFILE%\PVsyst8.1_Data` |
| `PVSYST_CLI` | Path to `PVsystCLI.exe` | `C:\Program Files\PVsyst8.1.7\PVsystCLI.exe` |
| `PVSYST_GUI` | Path to `PVsyst.exe` | `C:\Program Files\PVsyst8.1.7\PVsyst.exe` |
| `PVSYST_MCP_EXPERTO` | `1` also exposes the raw CLI and GUI tools (15 tools instead of 12) | unset |

The server speaks MCP over **stdio** and opens no network port. Keep it local to the machine
that holds your PVsyst licence, and read the *Responsible use* section of the project README
before automating a licensed installation — the GUI tools must not run on a server or over
Remote Desktop, and trial execution limits must be respected.

## Documentation

* Tool reference, quick start, FAQ and file formats:
  [README.md](https://github.com/Angelthebestone/PVSYST-MCP/blob/master/README.md)
* Changelog: [CHANGELOG.md](https://github.com/Angelthebestone/PVSYST-MCP/blob/master/CHANGELOG.md)
* Prefer a plain download? Every release ships the same binary as a zip:
  [releases](https://github.com/Angelthebestone/PVSYST-MCP/releases)
* Any PVsyst-generated table or figure you publish must quote PVsyst
  (clause 8.4 of its General Conditions of Use).

## Licence

Apache License 2.0 — see `LICENSE`. Attribution notice in `NOTICE`: forks and redistributions
must keep it and mark the files they changed.
