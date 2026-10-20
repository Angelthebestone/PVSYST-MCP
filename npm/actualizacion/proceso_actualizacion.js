// Entry point of the detached updater the launcher starts after the server.
// argv[2] is the version the current session runs; the result is logged to
// <almacen>/actualizacion.log and the process always exits 0.
import { mkdirSync } from "node:fs";
import { join } from "node:path";
import { actualizar } from "./aplicacion/actualizar.js";
import { leerConfiguracion } from "./dominio/configuracion.js";
import { parsearVersion } from "./dominio/version.js";
import * as almacenamiento from "./infraestructura/almacen.js";
import { crearBitacoraArchivo } from "./infraestructura/bitacora.js";
import { consultarVersionPublicada, instalarVersion } from "./infraestructura/npm.js";

const enUso = process.argv[2];
const { habilitada, almacen } = leerConfiguracion(process.env);

if (habilitada && parsearVersion(enUso) !== null) {
  mkdirSync(almacen, { recursive: true });
  const registrar = crearBitacoraArchivo(join(almacen, "actualizacion.log"));
  const estado = await actualizar({
    almacen,
    enUso,
    npm: { consultarVersionPublicada, instalarVersion },
    almacenamiento,
    registrar,
  });
  registrar({ evento: "actualizacion", en_uso: enUso, ...estado });
}
process.exit(0);
