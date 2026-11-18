"""Measure PVsyst windows through the MCP server and merge them into the inventory.

For each target of the manifest (`tools/gui_manifiesto.json`) it opens the window by
its chain of `gui_actuar funcion`, reads every tab with `gui_observar que:formulario`,
closes whatever it opened and writes the result into `docs/gui_inventario_pestanas.json`
under the target's `clave`. `tools/mapa_desde_inventario.py` then regenerates the map.

    python tools/medir_gui.py --servidor <pvsyst-mcp.exe> [--solo CLAVE ...]

A chain step `@proyecto:NOMBRE` loads that project through the search box of the
project loader (PVsyst's shipped `_DEMO_*` projects are read-only, so nothing is
saved: any «¿Desea guardarla?» or «¿mantener modificaciones?» prompt is answered No).
Only one agent may drive the PVsyst GUI at a time. Nothing is filled in or saved.

The transport (`ClienteMcp`) is exercised live only; the rest takes any object with
`tool(nombre, argumentos)`, which the tests fake.
"""

from __future__ import annotations

import argparse
import json
import os
import queue
import re
import subprocess
import sys
import tempfile
import threading
import time
from collections.abc import Callable
from pathlib import Path
from typing import Any, Protocol

Json = dict[str, Any]

RAIZ = Path(__file__).resolve().parent.parent
INVENTARIO = RAIZ / "docs" / "gui_inventario_pestanas.json"
MANIFIESTO = Path(__file__).resolve().parent / "gui_manifiesto.json"

CLASES_ROTULO = ("TTabSheet", "TGroupBox", "TRadioGroup")
CLAVES_CAMPO = ("clase", "etiqueta", "etiqueta_fuente", "marcado", "habilitado")
MANTENER = ("THomeForm", "TApplication", "TProjectForm")
# Search box of TProjectLoadForm, client coordinates at 96 DPI.
BUSCADOR = {"clase": "TEdit", "x": 1077, "y": 49}

# Multiplies every wait; the tests set it to 0.
PAUSA = 1.0
# Seconds to wait for the server to answer one call before giving up.
PLAZO = 120.0


class ErrorMedicion(Exception):
    """A read or a step failed: the measurement of this target cannot be trusted."""


def _dormir(segundos: float) -> None:
    time.sleep(segundos * PAUSA)


def _hasta(condicion: Callable[[], bool], plazo: float) -> bool:
    """Poll every half second until `condicion()` holds or `plazo` seconds pass."""
    for _ in range(int(plazo / 0.5) + 1):
        if condicion():
            return True
        _dormir(0.5)
    return condicion()


def _es_proyecto(w: Json, nombre: str) -> bool:
    """The project window of exactly `nombre` (not `nombre2`, not `x_nombre`)."""
    patron = rf"(?<!\w){re.escape(nombre)}(?!\w)"
    return w["class"] == "TProjectForm" and bool(re.search(patron, w.get("title", "")))


class Cliente(Protocol):
    def tool(self, nombre: str, argumentos: Json) -> Json: ...


class ClienteMcp:
    """JSON-RPC over the stdio of a `pvsyst-mcp` process."""

    def __init__(
        self, servidor: Path, workspace: Path, argumentos: tuple[str, ...] = ()
    ) -> None:
        env = {k: v for k, v in os.environ.items() if not k.startswith("PVSYST_MCP")}
        env["PVSYST_WORKSPACE"] = str(workspace)
        self.proc = subprocess.Popen(
            [str(servidor), *argumentos],
            stdin=subprocess.PIPE,
            stdout=subprocess.PIPE,
            stderr=subprocess.DEVNULL,
            text=True,
            encoding="utf-8",
            env=env,
        )
        self._id = 0
        # A reader thread, so that a server that hangs cannot hang this process.
        self._lineas: queue.Queue[str | None] = queue.Queue()
        threading.Thread(target=self._leer, daemon=True).start()
        self._llamar(
            "initialize",
            {
                "protocolVersion": "2025-06-18",
                "capabilities": {},
                "clientInfo": {"name": "medir_gui", "version": "0"},
            },
        )
        self._enviar(
            {"jsonrpc": "2.0", "method": "notifications/initialized", "params": {}}
        )

    def _leer(self) -> None:
        assert self.proc.stdout
        for linea in self.proc.stdout:
            self._lineas.put(linea)
        self._lineas.put(None)

    def _enviar(self, obj: Json) -> None:
        assert self.proc.stdin
        self.proc.stdin.write(json.dumps(obj, ensure_ascii=False) + "\n")
        self.proc.stdin.flush()

    def _llamar(self, metodo: str, params: Json) -> Json:
        self._id += 1
        self._enviar(
            {"jsonrpc": "2.0", "id": self._id, "method": metodo, "params": params}
        )
        while True:
            try:
                linea = self._lineas.get(timeout=PLAZO)
            except queue.Empty:
                raise TimeoutError(
                    f"no answer to {metodo} in {PLAZO:g} s (waiting for id {self._id})"
                ) from None
            if linea is None:
                raise RuntimeError("the server closed stdout")
            if linea.strip():
                resp: Json = json.loads(linea)
                if resp.get("id") == self._id:
                    return resp

    def tool(self, nombre: str, argumentos: Json) -> Json:
        resp = self._llamar("tools/call", {"name": nombre, "arguments": argumentos})
        if "error" in resp:
            return {"jsonrpc_error": resp["error"]}
        res: Json = resp["result"]
        estructurado = res.get("structuredContent")
        if isinstance(estructurado, dict):
            return estructurado
        for item in res.get("content", []):
            if item.get("type") == "text":
                try:
                    datos = json.loads(item.get("text", ""))
                except json.JSONDecodeError:
                    continue
                if isinstance(datos, dict):
                    return datos
        return {}

    def cerrar(self) -> None:
        assert self.proc.stdin
        self.proc.stdin.close()
        try:
            self.proc.wait(timeout=5)
        except subprocess.TimeoutExpired:
            self.proc.kill()
            self.proc.wait(timeout=5)


def ventanas(c: Cliente) -> list[Json]:
    r = c.tool("gui_observar", {"que": "ventanas", "visibles": True})
    return list(r.get("windows", []))


def asegurar_pvsyst(c: Cliente) -> str | None:
    """Launch PVsyst when no PVsyst process is running; an error text, or None."""
    if "windows" in c.tool("gui_observar", {"que": "ventanas", "visibles": True}):
        return None
    c.tool("gui_actuar", {"accion": "lanzar"})
    for _ in range(30):
        _dormir(1.0)
        if "windows" in c.tool("gui_observar", {"que": "ventanas", "visibles": True}):
            return None
    return "PVsyst did not start"


def _boton_de(textos: str) -> str:
    """The answer to a message box that never saves: Sí to «cancel the
    modifications?» (measured on 8.1.8, rotulos/dialogos.rs), No to «save?»,
    OK otherwise."""
    if "cancelar las modificaciones" in textos:
        return "Sí"
    return "No" if ("guard" in textos or "mantener" in textos) else "OK"


def limpiar(c: Cliente) -> list[str]:
    """Close every window but the main ones; the classes still open if it gives up."""
    for _ in range(12):
        sobra = [w for w in ventanas(c) if w["class"] not in MANTENER]
        if not sobra:
            return []
        clase = sobra[0]["class"]
        if clase == "TMessageForm":
            f = c.tool("gui_observar", {"que": "formulario", "ventana": clase})
            boton = _boton_de(" ".join(f.get("textos") or []).lower())
            c.tool(
                "gui_actuar", {"accion": "clic_texto", "ventana": clase, "texto": boton}
            )
        else:
            c.tool("gui_actuar", {"accion": "cerrar", "ventana": clase})
        _dormir(0.6)
    return [w["class"] for w in ventanas(c) if w["class"] not in MANTENER]


def cargar_proyecto(c: Cliente, nombre: str) -> str | None:
    """Load a project through the loader's search box; an error text, or None."""
    if any(_es_proyecto(w, nombre) for w in ventanas(c)):
        return None
    limpiar(c)
    if any(w["class"] == "TProjectForm" for w in ventanas(c)):
        c.tool("gui_actuar", {"accion": "cerrar", "ventana": "TProjectForm"})
        _dormir(0.8)
        limpiar(c)
    c.tool("gui_actuar", {"accion": "funcion", "funcion": "proyecto.cargar"})
    _dormir(1.0)
    c.tool(
        "gui_actuar",
        {
            "accion": "escribir",
            "ventana": "TProjectLoadForm",
            "campo": BUSCADOR,
            "texto": nombre,
        },
    )
    _dormir(1.0)
    c.tool(
        "gui_actuar",
        {"accion": "fila", "ventana": "TProjectLoadForm", "indice": 0, "abrir": True},
    )
    if not _hasta(lambda: any(_es_proyecto(w, nombre) for w in ventanas(c)), 15):
        titulos = [w.get("title") for w in ventanas(c)]
        return f"project {nombre} did not load: {titulos}"
    return None


def abrir(c: Cliente, cadena: str, clase: str) -> str | None:
    """Run the chain of steps; an error text, or None when `clase` is open."""
    for paso in cadena.split(","):
        if paso.startswith("@proyecto:"):
            fallo = cargar_proyecto(c, paso.split(":", 1)[1])
            if fallo:
                return fallo
            continue
        if any(w["class"] == "TDEMOForm" for w in ventanas(c)):
            c.tool("gui_actuar", {"accion": "cerrar", "ventana": "TDEMOForm"})
        r = c.tool("gui_actuar", {"accion": "funcion", "funcion": paso})
        # A host window that opens a different one than the catalogue expects still
        # leaves the target open; the check below decides.
        if r.get("error") and "did not open the expected window" not in str(r["error"]):
            return f"{paso}: {str(r['error'])[:200]}"
        _dormir(1.0)
    if _hasta(lambda: any(w["class"] == clase for w in ventanas(c)), 5):
        return None
    return f"{clase} did not open: {[w['class'] for w in ventanas(c)]}"


def leer_pagina(c: Cliente, clase: str) -> Json:
    f = c.tool("gui_observar", {"que": "formulario", "ventana": clase})
    fallo = f.get("error") or f.get("jsonrpc_error")
    if fallo:
        raise ErrorMedicion(f"reading {clase} failed: {str(fallo)[:200]}")
    return {
        "campos": [{k: x.get(k) for k in CLAVES_CAMPO} for x in f.get("campos") or []],
        "botones": [
            {"texto": b.get("texto"), "habilitado": b.get("habilitado")}
            for b in f.get("botones") or []
        ],
        "rotulos": [
            [o["clase"], o.get("texto")]
            for o in f.get("otros") or []
            if o.get("clase") in CLASES_ROTULO and o.get("texto")
        ],
        "titulo": f.get("titulo"),
        "pestanas": [p["nombre"] for p in f.get("pestanas") or []],
    }


def _cerrar_dialogos(c: Cliente, indice: int, errores: list[Json]) -> None:
    """A tab change can raise a message box (the window vetoes it): note it, dismiss it."""
    for _ in range(3):
        if not any(w["class"] == "TMessageForm" for w in ventanas(c)):
            return
        m = c.tool("gui_observar", {"que": "formulario", "ventana": "TMessageForm"})
        errores.append({"indice": indice, "dialogo": m.get("textos")})
        c.tool(
            "gui_actuar",
            {"accion": "clic_texto", "ventana": "TMessageForm", "texto": "OK"},
        )
        _dormir(0.5)


def inventariar(c: Cliente, clase: str) -> Json:
    """Every tab of the open window: fields, buttons and captions of the tab."""
    if c.tool("gui_observar", {"que": "formulario", "ventana": clase}).get("pestanas"):
        c.tool("gui_actuar", {"accion": "pagina", "ventana": clase, "indice": 0})
        _dormir(0.5)
    base = leer_pagina(c, clase)
    nombres: list[str] = base["pestanas"]
    errores: list[Json] = []
    pestanas: list[Json] = []
    for i in range(len(nombres) or 1):
        if nombres and i > 0:
            r = c.tool(
                "gui_actuar", {"accion": "pagina", "ventana": clase, "indice": i}
            )
            if r.get("error") or r.get("jsonrpc_error"):
                errores.append(
                    {"indice": i, "pestana": nombres[i], "texto": str(r)[:300]}
                )
                # The tab did not change: the screen shows another tab's page.
                _dormir(0.5)
                _cerrar_dialogos(c, i, errores)
                continue
            _dormir(0.5)
            _cerrar_dialogos(c, i, errores)
        p = leer_pagina(c, clase) if i > 0 else base
        pestanas.append(
            {
                "indice": i if nombres else None,
                "pestana": nombres[i] if nombres else None,
                **{k: p[k] for k in ("campos", "botones", "rotulos")},
            }
        )
    if nombres:
        c.tool("gui_actuar", {"accion": "pagina", "ventana": clase, "indice": 0})
    return {
        "ventana": clase,
        "titulo": base["titulo"],
        "pestanas": pestanas,
        "errores": errores,
    }


def medir(c: Cliente, objetivo: Json) -> Json | str:
    """Open, inventory and close one target: the measurement, or an error text."""
    previas = limpiar(c)
    if previas:
        return f"could not close the windows left open: {previas}"
    try:
        fallo = abrir(c, objetivo["cadena"], objetivo["ventana"])
        if fallo:
            return fallo
        medida = inventariar(c, objetivo["ventana"])
    except ErrorMedicion as e:
        return str(e)
    finally:
        limpiar(c)
    if medida["errores"]:
        return f"the window reported errors, nothing merged: {medida['errores']}"
    if not any(p["campos"] or p["botones"] for p in medida["pestanas"]):
        return "the window read back with no fields and no buttons, nothing merged"
    return medida


def volcar(datos: Json) -> str:
    return json.dumps(datos, indent=1, ensure_ascii=False)


def fusionar(inventario: Json, clave: str, medida: Json) -> None:
    inventario[clave] = {
        k: medida[k] for k in ("ventana", "titulo", "pestanas", "errores")
    }


def main(
    argv: list[str] | None = None,
    fabrica: Callable[[Path, Path], Any] = ClienteMcp,
) -> int:
    parser = argparse.ArgumentParser(description=__doc__.split("\n")[0])
    parser.add_argument("--servidor", type=Path, required=True)
    parser.add_argument("--manifiesto", type=Path, default=MANIFIESTO)
    parser.add_argument("--inventario", type=Path, default=INVENTARIO)
    parser.add_argument("--solo", nargs="*", help="measure only these `clave`s")
    args = parser.parse_args(argv)

    objetivos: list[Json] = json.loads(args.manifiesto.read_text(encoding="utf-8"))
    if args.solo:
        desconocidas = set(args.solo) - {o["clave"] for o in objetivos}
        if desconocidas:
            print(f"not in the manifest: {sorted(desconocidas)}")
            return 2
        objetivos = [o for o in objetivos if o["clave"] in args.solo]

    texto = args.inventario.read_text(encoding="utf-8")
    inventario: Json = json.loads(texto)
    if volcar(inventario) != texto.replace("\r\n", "\n").rstrip("\n"):
        print(f"{args.inventario} is not in the format this tool writes")
        return 2

    workspace = Path(tempfile.mkdtemp(prefix="pvsyst_medir_")) / "PVsyst8.1_Data"
    workspace.mkdir(parents=True)
    c = fabrica(args.servidor, workspace)
    fallos = 0
    fusionados = 0
    try:
        fallo_arranque = asegurar_pvsyst(c)
        if fallo_arranque:
            print(fallo_arranque)
            return 1
        for o in objetivos:
            try:
                r = medir(c, o)
            except (RuntimeError, TimeoutError) as e:
                # The server is gone or stuck: keep what was measured, stop here.
                print(f"{o['clave']}: {e}")
                fallos += 1
                break
            if isinstance(r, str):
                print(f"{o['clave']}: {r}")
                fallos += 1
                continue
            fusionar(inventario, o["clave"], r)
            fusionados += 1
            n = sum(len(p["campos"]) for p in r["pestanas"])
            print(
                f"{o['clave']}: {len(r['pestanas'])} tabs, {n} fields, {len(r['errores'])} errors"
            )
    finally:
        c.cerrar()
        if fusionados:
            args.inventario.write_text(volcar(inventario) + "\n", encoding="utf-8")
    return 1 if fallos else 0


if __name__ == "__main__":
    sys.exit(main())
