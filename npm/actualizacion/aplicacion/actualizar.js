// Use case: bring the store up to the latest published version. Every side
// effect goes through the injected ports, so tests drive it with fakes.
import { compararVersiones, versionMasReciente } from "../dominio/version.js";

const resultado = (campos) => Object.freeze(campos);

/** Installs `publicada` into the store; the lock is already held. */
async function instalarYPublicar({ almacen, enUso, npm, almacenamiento, registrar }, publicada) {
  const staging = almacenamiento.prepararStaging(almacen, publicada);
  if (!(await npm.instalarVersion(publicada, staging))) {
    almacenamiento.descartarStaging(almacen, publicada);
    return resultado({ estado: "error", detalle: `npm install pvsyst-mcp@${publicada} falló` });
  }
  await almacenamiento.publicarVersion(almacen, publicada);
  almacenamiento.podarVersiones(almacen, new Set([publicada, enUso]), registrar);
  return resultado({ estado: "actualizada", version: publicada });
}

async function actualizarConCerrojo(puertos) {
  const { almacen, enUso, npm, almacenamiento } = puertos;
  const publicada = await npm.consultarVersionPublicada();
  if (publicada === null) {
    return resultado({ estado: "error", detalle: "no se pudo consultar la versión publicada" });
  }
  const instaladas = almacenamiento.listarVersionesCompletas(almacen).map((v) => v.version);
  const actual = versionMasReciente([enUso, ...instaladas]);
  if (actual !== null && compararVersiones(publicada, actual) <= 0) {
    return resultado({ estado: "al_dia" });
  }
  return instalarYPublicar(puertos, publicada);
}

const detalleDe = (error) => String(error?.message ?? error);

/** `{liberar}` (null when busy) or `{error}` if the store is unusable. */
function intentarCerrojo({ almacen, almacenamiento }) {
  try {
    return { liberar: almacenamiento.tomarCerrojo(almacen) };
  } catch (error) {
    return { error };
  }
}

/**
 * One update pass. Resolves `{estado: "ocupado" | "al_dia" | "actualizada" | "error"}`
 * (`version` when updated, `detalle` on error); never rejects.
 * Ports: `npm` {consultarVersionPublicada, instalarVersion}; `almacenamiento`
 * {tomarCerrojo, listarVersionesCompletas, prepararStaging, descartarStaging,
 * publicarVersion, podarVersiones}; `registrar(evento)`.
 */
export async function actualizar(puertos) {
  const { liberar, error } = intentarCerrojo(puertos);
  if (error !== undefined) {
    return resultado({ estado: "error", detalle: detalleDe(error) });
  }
  if (liberar === null) {
    return resultado({ estado: "ocupado" });
  }
  try {
    return await actualizarConCerrojo(puertos);
  } catch (fallo) {
    return resultado({ estado: "error", detalle: detalleDe(fallo) });
  } finally {
    liberar();
  }
}
