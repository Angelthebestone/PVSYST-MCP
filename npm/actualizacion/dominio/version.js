// Release versions of pvsyst-mcp: plain `x.y.z`. Prereleases are rejected on
// purpose so a test build published under a tag never replaces a stable one.

const PATRON_VERSION = /^\d+\.\d+\.\d+$/;
const COMPONENTES = Object.freeze(["mayor", "menor", "parche"]);

/** `{mayor, menor, parche}` for a valid `x.y.z` text, else `null`. */
export function parsearVersion(texto) {
  if (typeof texto !== "string" || !PATRON_VERSION.test(texto)) {
    return null;
  }
  const [mayor, menor, parche] = texto.split(".").map(Number);
  return Object.freeze({ mayor, menor, parche });
}

/** -1, 0 or 1 as `a` is older than, equal to or newer than `b`. */
export function compararVersiones(a, b) {
  const [va, vb] = [parsearVersion(a), parsearVersion(b)];
  if (va === null || vb === null) {
    throw new TypeError(`Versión no válida: ${va === null ? a : b}`);
  }
  const diferencia = COMPONENTES.map((c) => Math.sign(va[c] - vb[c])).find((d) => d !== 0);
  return diferencia ?? 0;
}

/** Newest valid version among `textos`, or `null` when none is valid. */
export function versionMasReciente(textos) {
  return textos
    .filter((t) => parsearVersion(t) !== null)
    .reduce((mejor, t) => (mejor === null || compararVersiones(t, mejor) > 0 ? t : mejor), null);
}
