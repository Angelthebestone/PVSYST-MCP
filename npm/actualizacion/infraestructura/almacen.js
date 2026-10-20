// On-disk store of downloaded pvsyst-mcp versions:
//   <almacen>/versiones/<x.y.z>/node_modules/pvsyst-mcp/vendor/pvsyst-mcp.exe
//   <almacen>/versiones/<x.y.z>/.completa      (written last: install finished)
//   <almacen>/staging-<x.y.z>/                 (npm installs here first)
//   <almacen>/actualizando.lock                (one updater at a time)
// A version only becomes visible through one directory rename, so a launcher
// never sees a half-installed binary.
import {
  closeSync,
  existsSync,
  mkdirSync,
  openSync,
  readdirSync,
  renameSync,
  rmSync,
  statSync,
  writeFileSync,
} from "node:fs";
import { join } from "node:path";
import { parsearVersion } from "../dominio/version.js";

/** A lock older than this belongs to a crashed updater and may be taken over. */
export const CADUCIDAD_CERROJO_MS = 10 * 60 * 1000;
const MARCADOR_COMPLETA = ".completa";
const ARCHIVO_CERROJO = "actualizando.lock";

const directorioVersiones = (almacen) => join(almacen, "versiones");
const directorioVersion = (almacen, version) => join(directorioVersiones(almacen), version);

/** Where `npm install --prefix <prefijo> pvsyst-mcp` leaves the server binary. */
export function exeEnPrefijo(prefijo) {
  return join(prefijo, "node_modules", "pvsyst-mcp", "vendor", "pvsyst-mcp.exe");
}

/** Staging directory where `version` is installed before publication. */
export function directorioStaging(almacen, version) {
  return join(almacen, `staging-${version}`);
}

/** Names of the entries of `dir`, or `[]` when it does not exist. */
function entradas(dir) {
  return existsSync(dir) ? readdirSync(dir, { withFileTypes: true }) : [];
}

/** Completed installs as `{version, exe}`, in no particular order. */
export function listarVersionesCompletas(almacen) {
  return entradas(directorioVersiones(almacen))
    .filter((e) => e.isDirectory() && parsearVersion(e.name) !== null)
    .map((e) => ({ version: e.name, dir: directorioVersion(almacen, e.name) }))
    .filter(({ dir }) => existsSync(join(dir, MARCADOR_COMPLETA)))
    .map(({ version, dir }) => Object.freeze({ version, exe: exeEnPrefijo(dir) }))
    .filter(({ exe }) => existsSync(exe));
}

/** Creates the lock file exclusively; `true` on success, `false` if it exists. */
function crearCerrojo(ruta) {
  try {
    closeSync(openSync(ruta, "wx"));
    return true;
  } catch (error) {
    if (error.code === "EEXIST") {
      return false;
    }
    throw error;
  }
}

/** `true` when the lock at `ruta` is recent enough to belong to a live updater. */
function cerrojoVigente(ruta, ahora) {
  return ahora - statSync(ruta).mtimeMs < CADUCIDAD_CERROJO_MS;
}

/**
 * Takes the updater lock. Returns `liberar()` or `null` when another updater
 * holds a recent lock; a stale lock (crashed updater) is replaced.
 */
export function tomarCerrojo(almacen, ahora = Date.now()) {
  mkdirSync(almacen, { recursive: true });
  const ruta = join(almacen, ARCHIVO_CERROJO);
  const tomado = crearCerrojo(ruta) || (!cerrojoVigente(ruta, ahora) && reemplazarCerrojo(ruta));
  return tomado ? () => liberarCerrojo(ruta) : null;
}

/**
 * Releases the lock; never throws, because it runs in a `finally` whose
 * result must survive. A lock that cannot be removed simply goes stale and is
 * taken over after CADUCIDAD_CERROJO_MS. `true` when it was removed.
 */
function liberarCerrojo(ruta) {
  try {
    return retirarCerrojo(ruta);
  } catch {
    return false;
  }
}

/**
 * Moves the lock aside before deleting it. On Windows a deleted file keeps
 * its name while any handle (antivirus, indexer) is open, and `wx` on that
 * name fails with EPERM; a rename frees the name at once, and only one
 * updater can win it. `false` when another updater moved it first.
 */
function retirarCerrojo(ruta) {
  const apartado = `${ruta}.${process.pid}.${Date.now()}.retirado`;
  try {
    renameSync(ruta, apartado);
  } catch (error) {
    if (error.code === "ENOENT") {
      return false;
    }
    throw error;
  }
  rmSync(apartado, { force: true });
  return true;
}

function reemplazarCerrojo(ruta) {
  return retirarCerrojo(ruta) && crearCerrojo(ruta);
}

/** Fresh, empty staging directory for `version`. */
export function prepararStaging(almacen, version) {
  const staging = directorioStaging(almacen, version);
  rmSync(staging, { recursive: true, force: true });
  mkdirSync(staging, { recursive: true });
  return staging;
}

/** Removes the staging directory of `version`, if any. */
export function descartarStaging(almacen, version) {
  rmSync(directorioStaging(almacen, version), { recursive: true, force: true });
}

/** Rename attempts before giving up on a directory still held open. */
export const REINTENTOS_RENOMBRADO = 10;
/** Base pause between attempts; it grows linearly with each one. */
export const PAUSA_RENOMBRADO_MS = 500;
const CODIGOS_TRANSITORIOS = new Set(["EPERM", "EBUSY", "EACCES"]);

const esperar = (ms) => new Promise((resolver) => setTimeout(resolver, ms));

/**
 * `renameSync` retried while Windows still holds the fresh files open: the
 * antivirus scans a just-installed `.exe`, and the rename fails with EPERM
 * for a few seconds after `npm install` returns.
 */
async function renombrarConReintentos(origen, destino, intento = 1) {
  try {
    renameSync(origen, destino);
  } catch (error) {
    if (!CODIGOS_TRANSITORIOS.has(error.code) || intento >= REINTENTOS_RENOMBRADO) {
      throw error;
    }
    await esperar(PAUSA_RENOMBRADO_MS * intento);
    await renombrarConReintentos(origen, destino, intento + 1);
  }
}

/**
 * Marks the staged install complete and renames it into `versiones/`.
 * Idempotent: if the version is already published the staging is discarded.
 * On failure the staging is discarded too (best effort) and the error is
 * rethrown, so no half-published copy is left behind.
 */
export async function publicarVersion(almacen, version) {
  const destino = directorioVersion(almacen, version);
  if (existsSync(destino)) {
    descartarStaging(almacen, version);
    return;
  }
  const staging = directorioStaging(almacen, version);
  try {
    writeFileSync(join(staging, MARCADOR_COMPLETA), "");
    mkdirSync(directorioVersiones(almacen), { recursive: true });
    await renombrarConReintentos(staging, destino);
  } catch (error) {
    descartarStagingSiSePuede(almacen, version);
    throw error;
  }
}

/**
 * `descartarStaging` that cannot hide the failure being reported: if the
 * folder is still locked, the next run's `prepararStaging` removes it.
 */
function descartarStagingSiSePuede(almacen, version) {
  try {
    descartarStaging(almacen, version);
  } catch {
    return false;
  }
  return true;
}

/**
 * Deletes every version outside `conservar`. A version whose binary is still
 * running cannot be deleted on Windows (EBUSY/EPERM): that failure is logged
 * and the next update retries it.
 */
export function podarVersiones(almacen, conservar, registrar) {
  entradas(directorioVersiones(almacen))
    .filter((e) => e.isDirectory() && !conservar.has(e.name))
    .forEach((e) => {
      try {
        rmSync(directorioVersion(almacen, e.name), { recursive: true, force: true });
      } catch (error) {
        registrar({ evento: "poda_fallida", version: e.name, codigo: error.code ?? null });
      }
    });
}
