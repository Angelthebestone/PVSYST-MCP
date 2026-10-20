import assert from "node:assert/strict";
import { existsSync, mkdirSync, mkdtempSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import { dirname, join } from "node:path";
import { test } from "node:test";
import * as almacenamiento from "../infraestructura/almacen.js";
import { actualizar } from "./actualizar.js";

const nuevoAlmacen = () => mkdtempSync(join(tmpdir(), "pvsyst-mcp-test-"));

/** Fake npm: reports `publicada` and "installs" by writing the binary. */
function npmFalso(publicada, { instalacionCorrecta = true } = {}) {
  const instaladas = [];
  return {
    instaladas,
    consultarVersionPublicada: async () => publicada,
    instalarVersion: async (version, prefijo) => {
      instaladas.push(version);
      if (!instalacionCorrecta) {
        return false;
      }
      const exe = almacenamiento.exeEnPrefijo(prefijo);
      mkdirSync(dirname(exe), { recursive: true });
      writeFileSync(exe, "exe");
      return true;
    },
  };
}

/** Publishes `version` in `almacen` as a completed install. */
async function instalarPrevia(almacen, version) {
  const staging = almacenamiento.prepararStaging(almacen, version);
  const exe = almacenamiento.exeEnPrefijo(staging);
  mkdirSync(dirname(exe), { recursive: true });
  writeFileSync(exe, "exe");
  await almacenamiento.publicarVersion(almacen, version);
}

const ejecutar = (almacen, npm, enUso = "0.1.1") =>
  actualizar({ almacen, enUso, npm, almacenamiento, registrar: () => true });

const completa = (almacen, version) => join(almacen, "versiones", version, ".completa");

test("should publish newer version when registry has one", async () => {
  const almacen = nuevoAlmacen();
  const r = await ejecutar(almacen, npmFalso("0.1.2"));
  assert.deepEqual({ ...r }, { estado: "actualizada", version: "0.1.2" });
  assert.ok(existsSync(completa(almacen, "0.1.2")));
  assert.deepEqual(
    almacenamiento.listarVersionesCompletas(almacen).map((v) => v.version),
    ["0.1.2"],
  );
});

test("should report up to date and install nothing when versions match", async () => {
  const almacen = nuevoAlmacen();
  const npm = npmFalso("0.1.1");
  const r = await ejecutar(almacen, npm);
  assert.equal(r.estado, "al_dia");
  assert.deepEqual(npm.instaladas, []);
});

test("should report busy when another updater holds a recent lock", async () => {
  const almacen = nuevoAlmacen();
  writeFileSync(join(almacen, "actualizando.lock"), "");
  const npm = npmFalso("0.1.2");
  const r = await ejecutar(almacen, npm);
  assert.equal(r.estado, "ocupado");
  assert.deepEqual(npm.instaladas, []);
});

test("should take over lock when it is stale", () => {
  const almacen = nuevoAlmacen();
  writeFileSync(join(almacen, "actualizando.lock"), "");
  const futuro = Date.now() + almacenamiento.CADUCIDAD_CERROJO_MS + 60_000;
  const liberar = almacenamiento.tomarCerrojo(almacen, futuro);
  assert.equal(typeof liberar, "function");
  liberar();
  assert.equal(existsSync(join(almacen, "actualizando.lock")), false);
});

test("should report error and publish nothing when install fails", async () => {
  const almacen = nuevoAlmacen();
  const r = await ejecutar(almacen, npmFalso("0.1.2", { instalacionCorrecta: false }));
  assert.equal(r.estado, "error");
  assert.equal(existsSync(join(almacen, "versiones", "0.1.2")), false);
  assert.equal(existsSync(join(almacen, "staging-0.1.2")), false);
});

test("should report error when registry cannot be queried", async () => {
  const r = await ejecutar(nuevoAlmacen(), npmFalso(null));
  assert.equal(r.estado, "error");
});

test("should keep version in use and drop older ones when pruning", async () => {
  const almacen = nuevoAlmacen();
  await instalarPrevia(almacen, "0.1.0");
  await instalarPrevia(almacen, "0.1.1");
  const r = await ejecutar(almacen, npmFalso("0.1.2"), "0.1.1");
  assert.equal(r.estado, "actualizada");
  assert.deepEqual(
    almacenamiento
      .listarVersionesCompletas(almacen)
      .map((v) => v.version)
      .sort(),
    ["0.1.1", "0.1.2"],
  );
});

test("should release lock after an update pass", async () => {
  const almacen = nuevoAlmacen();
  await ejecutar(almacen, npmFalso("0.1.2"));
  assert.equal(existsSync(join(almacen, "actualizando.lock")), false);
});
