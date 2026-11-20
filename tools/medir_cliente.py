"""The client a measurement talks to: JSON-RPC over the stdio of a `pvsyst-mcp` process.

`Cliente` is the whole contract the measurement needs from the server: any object
with `tool(nombre, argumentos)`. `ClienteMcp` is the only implementation and it is
exercised live only; the rest of the tool takes the protocol, which the tests fake.
"""

from __future__ import annotations

import json
import os
import queue
import subprocess
import threading
from pathlib import Path
from typing import Any, Protocol

Json = dict[str, Any]

# Seconds to wait for the server to answer one call before giving up.
PLAZO = 120.0


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
