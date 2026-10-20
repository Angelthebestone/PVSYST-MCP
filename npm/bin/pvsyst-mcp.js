#!/usr/bin/env node
// Launcher for the bundled Windows binary. It spawns the server with inherited
// stdio, so an MCP client that runs `npx pvsyst-mcp` speaks to the server
// directly, and forwards environment variables (PVSYST_WORKSPACE, PVSYST_CLI,
// PVSYST_GUI, PVSYST_MCP_EXPERTO) untouched.
import { spawn } from "node:child_process";
import { existsSync } from "node:fs";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";

const here = dirname(fileURLToPath(import.meta.url));
const exe = join(here, "..", "vendor", "pvsyst-mcp.exe");

if (!existsSync(exe)) {
  process.stderr.write(
    `pvsyst-mcp: missing ${exe}\n` +
      "Reinstall the package, or download the release binary from " +
      "https://github.com/Angelthebestone/PVSYST-MCP/releases\n",
  );
  process.exit(1);
}

const child = spawn(exe, process.argv.slice(2), {
  stdio: "inherit",
  windowsHide: true,
});

for (const signal of ["SIGINT", "SIGTERM"]) {
  process.on(signal, () => child.kill(signal));
}

child.on("error", (error) => {
  process.stderr.write(`pvsyst-mcp: cannot start ${exe}: ${error.message}\n`);
  process.exit(1);
});

child.on("exit", (code, signal) => {
  process.exit(signal ? 1 : code ?? 1);
});
