// Structured log lines (one JSON object per line). The launcher logs to stderr,
// which MCP clients keep as the server log; the detached updater has no stderr
// and logs to a file that is rotated once it reaches TAMANO_MAX_LOG.
import { appendFileSync, existsSync, renameSync, statSync } from "node:fs";

export const TAMANO_MAX_LOG = 1024 * 1024;

const lineaDe = (evento) => `${JSON.stringify({ fecha: new Date().toISOString(), ...evento })}\n`;

/** Logger writing to stderr. */
export function crearBitacoraStderr() {
  return (evento) => {
    process.stderr.write(lineaDe(evento));
  };
}

/**
 * Logger appending to `ruta`, rotating it to `<ruta>.1` past TAMANO_MAX_LOG.
 * Returns `false` when the line could not be written: logging must never
 * break an update, and a detached process has nowhere else to report it.
 */
export function crearBitacoraArchivo(ruta) {
  return (evento) => {
    try {
      if (existsSync(ruta) && statSync(ruta).size > TAMANO_MAX_LOG) {
        renameSync(ruta, `${ruta}.1`);
      }
      appendFileSync(ruta, lineaDe(evento));
      return true;
    } catch {
      return false;
    }
  };
}
