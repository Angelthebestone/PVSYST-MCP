#!/usr/bin/env node
// Launcher for the bundled Windows binary. It spawns the server with inherited
// stdio, so an MCP client that runs `npx pvsyst-mcp` speaks to the server
// directly, and forwards environment variables (PVSYST_WORKSPACE, PVSYST_CLI,
// PVSYST_GUI, PVSYST_MCP_APROXIMACIONES) untouched.
//
// Auto-update: the session runs the newest of the packaged binary and the
// versions already downloaded to the store. After the server starts, a
// detached process downloads a newer release, if any, for the next start.
// Disable with PVSYST_MCP_AUTO_ACTUALIZAR=0.
import { spawn } from "node:child_process";
import { existsSync, readFileSync } from "node:fs";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";
import { leerConfiguracion } from "../actualizacion/dominio/configuracion.js";
import { elegirBinario } from "../actualizacion/dominio/seleccion.js";
import { listarVersionesCompletas } from "../actualizacion/infraestructura/almacen.js";
import { crearBitacoraStderr } from "../actualizacion/infraestructura/bitacora.js";

const here = dirname(fileURLToPath(import.meta.url));
const registrar = crearBitacoraStderr();
const { habilitada, almacen } = leerConfiguracion(process.env);
const { version } = JSON.parse(readFileSync(join(here, "..", "package.json"), "utf8"));
const empaquetado = Object.freeze({ version, exe: join(here, "..", "vendor", "pvsyst-mcp.exe") });

/** Downloaded versions; a broken store must never keep the server from starting. */
function versionesDescargadas() {
  if (!habilitada) {
    return [];
  }
  try {
    return listarVersionesCompletas(almacen);
  } catch (error) {
    registrar({ evento: "almacen_ilegible", almacen, detalle: error.message });
    return [];
  }
}

const instaladas = versionesDescargadas();
const elegido = elegirBinario({ empaquetado, instaladas });

if (!existsSync(elegido.exe)) {
  process.stderr.write(
    `pvsyst-mcp: missing ${elegido.exe}\n` +
      "Reinstall the package, or download the release binary from " +
      "https://github.com/Angelthebestone/PVSYST-MCP/releases\n",
  );
  process.exit(1);
}

const child = spawn(elegido.exe, process.argv.slice(2), {
  stdio: "inherit",
  windowsHide: true,
});

for (const signal of ["SIGINT", "SIGTERM"]) {
  process.on(signal, () => child.kill(signal));
}

child.on("error", (error) => {
  process.stderr.write(`pvsyst-mcp: cannot start ${elegido.exe}: ${error.message}\n`);
  process.exit(1);
});

child.on("exit", (code, signal) => {
  process.exit(signal ? 1 : code ?? 1);
});

if (habilitada) {
  const actualizador = spawn(
    process.execPath,
    [join(here, "..", "actualizacion", "proceso_actualizacion.js"), elegido.version],
    { detached: true, stdio: "ignore", windowsHide: true },
  );
  // Without a listener a spawn failure would be thrown and end the launcher.
  actualizador.on("error", (error) =>
    registrar({ evento: "actualizador_no_iniciado", detalle: error.message }),
  );
  actualizador.unref();
}

registrar({
  evento: "arranque",
  version: elegido.version,
  origen: elegido === empaquetado ? "empaquetada" : "almacen",
});
