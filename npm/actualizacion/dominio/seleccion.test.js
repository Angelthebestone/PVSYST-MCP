import assert from "node:assert/strict";
import { test } from "node:test";
import { elegirBinario } from "./seleccion.js";

const empaquetado = { version: "0.1.1", exe: "vendor.exe" };

test("should choose installed binary when it is newer than the packaged one", () => {
  const nueva = { version: "0.1.2", exe: "almacen.exe" };
  assert.equal(elegirBinario({ empaquetado, instaladas: [nueva] }), nueva);
});

test("should keep packaged binary when an installed one has the same version", () => {
  const igual = { version: "0.1.1", exe: "almacen.exe" };
  assert.equal(elegirBinario({ empaquetado, instaladas: [igual] }), empaquetado);
});

test("should keep packaged binary when installs are older or absent", () => {
  assert.equal(elegirBinario({ empaquetado, instaladas: [] }), empaquetado);
  const vieja = { version: "0.1.0", exe: "almacen.exe" };
  assert.equal(elegirBinario({ empaquetado, instaladas: [vieja] }), empaquetado);
});
