import assert from "node:assert/strict";
import { join } from "node:path";
import { test } from "node:test";
import { leerConfiguracion } from "./configuracion.js";

test("should store versions under LOCALAPPDATA when no explicit store is set", () => {
  const c = leerConfiguracion({ LOCALAPPDATA: "C:\\Users\\u\\AppData\\Local" });
  assert.equal(c.almacen, join("C:\\Users\\u\\AppData\\Local", "pvsyst-mcp"));
  assert.equal(c.habilitada, true);
});

test("should prefer explicit store when PVSYST_MCP_ALMACEN is set", () => {
  const c = leerConfiguracion({ PVSYST_MCP_ALMACEN: " D:\\pmcp ", LOCALAPPDATA: "C:\\x" });
  assert.equal(c.almacen, "D:\\pmcp");
});

test("should disable updates when LOCALAPPDATA is blank and no store is set", () => {
  const c = leerConfiguracion({ LOCALAPPDATA: "  " });
  assert.deepEqual({ ...c }, { habilitada: false, almacen: null });
});

test("should disable updates when PVSYST_MCP_AUTO_ACTUALIZAR is 0", () => {
  const c = leerConfiguracion({ LOCALAPPDATA: "C:\\x", PVSYST_MCP_AUTO_ACTUALIZAR: "0" });
  assert.equal(c.habilitada, false);
});
