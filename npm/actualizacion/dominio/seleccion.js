import { compararVersiones } from "./version.js";

/**
 * Binary to run: the newest of the packaged one and the completed installs
 * (`{version, exe}` each). On a tie the packaged binary wins, so an install
 * left behind by an older run never shadows the package the user just chose.
 */
export function elegirBinario({ empaquetado, instaladas }) {
  return instaladas.reduce(
    (mejor, candidata) =>
      compararVersiones(candidata.version, mejor.version) > 0 ? candidata : mejor,
    empaquetado,
  );
}
