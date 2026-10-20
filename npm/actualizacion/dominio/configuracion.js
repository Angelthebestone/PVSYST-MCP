import { join } from "node:path";

/** Text of `valor` trimmed, or `null` when absent or blank. */
function textoUtil(valor) {
  const limpio = typeof valor === "string" ? valor.trim() : "";
  return limpio === "" ? null : limpio;
}

/**
 * Auto-update settings from the environment.
 * `almacen` is `PVSYST_MCP_ALMACEN`, else `%LOCALAPPDATA%\pvsyst-mcp`, else
 * `null`; updating is enabled unless `PVSYST_MCP_AUTO_ACTUALIZAR=0` or there is
 * nowhere to store versions.
 */
export function leerConfiguracion(env) {
  const explicito = textoUtil(env.PVSYST_MCP_ALMACEN);
  const localAppData = textoUtil(env.LOCALAPPDATA);
  const almacen = explicito ?? (localAppData === null ? null : join(localAppData, "pvsyst-mcp"));
  return Object.freeze({
    habilitada: env.PVSYST_MCP_AUTO_ACTUALIZAR !== "0" && almacen !== null,
    almacen,
  });
}
