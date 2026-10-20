// npm as a process: ask the registry for the published version and install a
// given version into a prefix. On Windows `npm` is `npm.cmd`, which only runs
// through a shell; every argument is fixed or validated before it gets here.
import { spawn } from "node:child_process";
import { existsSync } from "node:fs";
import { parsearVersion } from "../dominio/version.js";
import { exeEnPrefijo } from "./almacen.js";

export const TIEMPO_CONSULTA_MS = 30_000;
export const TIEMPO_INSTALACION_MS = 300_000;
const PAQUETE = "pvsyst-mcp";
/**
 * Runs `npm <args>`; resolves `{codigo, stdout}` (`codigo` null on timeout or
 * spawn failure). The command is one string because Node deprecates passing
 * an argument array together with `shell: true`.
 */
function ejecutarNpm(args, tiempoMaximoMs) {
  return new Promise((resolver) => {
    const hijo = spawn(["npm", ...args].join(" "), { shell: true, windowsHide: true });
    const trozos = [];
    const temporizador = setTimeout(() => hijo.kill(), tiempoMaximoMs);
    hijo.stdout.on("data", (trozo) => trozos.push(trozo));
    hijo.on("error", () => {
      clearTimeout(temporizador);
      resolver({ codigo: null, stdout: "" });
    });
    hijo.on("close", (codigo) => {
      clearTimeout(temporizador);
      resolver({ codigo, stdout: Buffer.concat(trozos).toString("utf8") });
    });
  });
}

/** `x.y.z` text of the latest stable version on the registry, else `null`. */
function versionDeSalida(stdout) {
  try {
    const valor = JSON.parse(stdout);
    return parsearVersion(valor) === null ? null : valor;
  } catch {
    return null;
  }
}

/** Latest published version of pvsyst-mcp, or `null` if npm or the network failed. */
export async function consultarVersionPublicada() {
  const { codigo, stdout } = await ejecutarNpm(
    ["view", PAQUETE, "version", "--json"],
    TIEMPO_CONSULTA_MS,
  );
  return codigo === 0 ? versionDeSalida(stdout) : null;
}

/** Installs `pvsyst-mcp@version` under `prefijo`; `true` only if the binary is there. */
export async function instalarVersion(version, prefijo) {
  if (parsearVersion(version) === null || prefijo.includes('"')) {
    return false;
  }
  const { codigo } = await ejecutarNpm(
    [
      "install",
      `${PAQUETE}@${version}`,
      "--prefix",
      `"${prefijo}"`,
      "--no-audit",
      "--no-fund",
      "--no-save",
    ],
    TIEMPO_INSTALACION_MS,
  );
  return codigo === 0 && existsSync(exeEnPrefijo(prefijo));
}
