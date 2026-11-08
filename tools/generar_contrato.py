"""Regenerates the client-facing copies of the tool registry.

The registry in `src/server/tools` is the only source: this script asks a built
server for `tools/list` and for the `pvsyst://cobertura` resource (which carries
the synonym table) and rewrites the copies that clients and the bundle read:

  - `mcpb/manifest.json`: `_meta.pvsyst-mcp.tool_aliases` and `tools`
  - `README.md` and `README.es.md`: the block between `<!-- alias:inicio -->`
    and `<!-- alias:fin -->`

Usage:
    python tools/generar_contrato.py [--exe PATH] [--check]

`--check` writes nothing and exits 1 when a file is out of date, so a stale
copy fails instead of drifting. The manifest half is also covered by
`tests/unit/server/tools/manifiesto_tests.rs`, which needs no built binary.

Why a subprocess: the registry is `pub(crate)`, so the only public way to read
it is the MCP protocol the server already speaks.
"""

from __future__ import annotations

import argparse
import json
import os
import subprocess
import sys
import tempfile
from pathlib import Path
from typing import Any, cast

RAIZ = Path(__file__).resolve().parent.parent
MANIFIESTO = RAIZ / "mcpb" / "manifest.json"
LEEMES = [RAIZ / "README.md", RAIZ / "README.es.md"]
INICIO = "<!-- alias:inicio -->"
FIN = "<!-- alias:fin -->"
SIN_VALORES = "—"
PROTOCOLO = "2025-06-18"

Peticion = tuple[str, dict[str, Any]]
TablaSinonimos = dict[str, dict[str, dict[str, str]]]


def _peticion(ident: int, metodo: str, params: dict[str, Any]) -> str:
    return json.dumps(
        {"jsonrpc": "2.0", "id": ident, "method": metodo, "params": params}
    )


def _responder(proceso: subprocess.Popen[str], ident: int) -> dict[str, Any]:
    """Reads lines until the answer to `ident` arrives."""
    salida = proceso.stdout
    if salida is None:
        raise SystemExit("el servidor no tiene stdout")
    while True:
        linea = salida.readline()
        if not linea:
            raise SystemExit(f"el servidor cerró sin responder a la petición {ident}")
        mensaje: dict[str, Any] = json.loads(linea)
        if mensaje.get("id") == ident:
            if "error" in mensaje:
                raise SystemExit(f"error del servidor: {mensaje['error']}")
            return cast(dict[str, Any], mensaje["result"])


def _consultar(
    exe: Path, aproximaciones: bool, peticiones: list[Peticion]
) -> list[dict[str, Any]]:
    entorno = dict(os.environ)
    entorno["PVSYST_MCP_IDIOMA"] = "en"
    entorno["PVSYST_MCP_LOG"] = "error"
    entorno.pop("PVSYST_MCP_APROXIMACIONES", None)
    if aproximaciones:
        entorno["PVSYST_MCP_APROXIMACIONES"] = "1"
    with tempfile.TemporaryDirectory() as workspace:
        entorno["PVSYST_WORKSPACE"] = workspace
        proceso = subprocess.Popen(
            [str(exe)],
            stdin=subprocess.PIPE,
            stdout=subprocess.PIPE,
            stderr=subprocess.DEVNULL,
            text=True,
            encoding="utf-8",
            env=entorno,
        )
        entrada = proceso.stdin
        if entrada is None:
            raise SystemExit("el servidor no tiene stdin")
        try:
            respuestas: list[dict[str, Any]] = []
            for ident, (metodo, params) in enumerate(peticiones, start=1):
                entrada.write(_peticion(ident, metodo, params) + "\n")
                entrada.flush()
                respuestas.append(_responder(proceso, ident))
            return respuestas
        finally:
            entrada.close()
            proceso.wait(timeout=30)


def herramientas(exe: Path) -> list[dict[str, str]]:
    """`tools/list` as a client sees it by default: English, no approximations."""
    initialize, listado = _consultar(
        exe,
        aproximaciones=False,
        peticiones=[
            ("initialize", {"protocolVersion": PROTOCOLO, "capabilities": {}}),
            ("tools/list", {}),
        ],
    )
    del initialize
    return [
        {"name": str(t["name"]), "description": str(t["description"])}
        for t in listado["tools"]
    ]


def sinonimos(exe: Path) -> TablaSinonimos:
    """The synonym table, with approximations on so no alias is filtered out."""
    _, recurso = _consultar(
        exe,
        aproximaciones=True,
        peticiones=[
            ("initialize", {"protocolVersion": PROTOCOLO, "capabilities": {}}),
            ("resources/read", {"uri": "pvsyst://cobertura"}),
        ],
    )
    tabla = json.loads(recurso["contents"][0]["text"])["sinonimos"]
    return cast(TablaSinonimos, tabla)


def _pares(pares: dict[str, str]) -> str:
    """Declaration order, so an unchanged registry leaves the README as it is."""
    if not pares:
        return SIN_VALORES
    return ", ".join(f"`{alias}` → `{canonico}`" for alias, canonico in pares.items())


def tabla_alias(orden: list[str], tabla: TablaSinonimos) -> str:
    filas = [
        "| Herramienta / Tool | Argumentos (alias → canónico) | Valores (alias → canónico) |"
    ]
    filas.append("|---|---|---|")
    for nombre in orden:
        datos = tabla.get(nombre, {})
        filas.append(
            f"| `{nombre}` | {_pares(datos.get('argumentos', {}))} "
            f"| {_pares(datos.get('valores', {}))} |"
        )
    return "\n".join(filas)


def manifest_actualizado(
    manifest: dict[str, Any], tools: list[dict[str, str]], tabla: TablaSinonimos
) -> str:
    manifest["_meta"]["pvsyst-mcp"]["tool_aliases"] = {
        nombre: tabla[nombre] for nombre in sorted(tabla)
    }
    manifest["tools"] = tools
    return json.dumps(manifest, indent=2, sort_keys=True, ensure_ascii=False) + "\n"


def leeme_actualizado(texto: str, tabla_md: str) -> str:
    inicio = texto.find(INICIO)
    fin = texto.find(FIN)
    if inicio < 0 or fin < inicio:
        raise SystemExit(f"falta el bloque {INICIO} … {FIN}")
    return texto[: inicio + len(INICIO)] + "\n" + tabla_md + "\n" + texto[fin:]


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__.splitlines()[0])
    parser.add_argument("--exe", type=Path, default=None, help="servidor compilado")
    parser.add_argument(
        "--check", action="store_true", help="no escribe; falla si difiere"
    )
    args = parser.parse_args()

    exe = (
        args.exe
        or Path(os.environ.get("CARGO_TARGET_DIR", RAIZ / "target"))
        / "debug"
        / "pvsyst-mcp.exe"
    )
    if not exe.is_file():
        raise SystemExit(f"no está {exe}: compila antes (cargo build) o pasa --exe")

    tools = herramientas(exe)
    tabla = sinonimos(exe)
    orden = [t["name"] for t in tools]

    salidas: dict[Path, str] = {}
    salidas[MANIFIESTO] = manifest_actualizado(
        json.loads(MANIFIESTO.read_text(encoding="utf-8")), tools, tabla
    )
    tabla_md = tabla_alias(orden, tabla)
    for leeme in LEEMES:
        salidas[leeme] = leeme_actualizado(leeme.read_text(encoding="utf-8"), tabla_md)

    desfasados: list[Path] = []
    for ruta, contenido in salidas.items():
        actual = ruta.read_text(encoding="utf-8")
        if actual == contenido:
            continue
        desfasados.append(ruta)
        if not args.check:
            ruta.write_text(contenido, encoding="utf-8", newline="\n")

    if args.check and desfasados:
        for ruta in desfasados:
            print(f"desfasado: {ruta.relative_to(RAIZ)}", file=sys.stderr)
        return 1
    for ruta in desfasados:
        print(f"actualizado: {ruta.relative_to(RAIZ)}")
    return 0


if __name__ == "__main__":
    sys.exit(main())
