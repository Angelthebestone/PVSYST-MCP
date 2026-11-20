"""Tests for medir_cliente.py: the MCP client a measurement talks to."""

import sys
from pathlib import Path

import pytest

import medir_cliente
from medir_cliente import ClienteMcp


def test_a_server_that_never_answers_times_out_instead_of_hanging(
    monkeypatch: pytest.MonkeyPatch,
) -> None:
    monkeypatch.setattr(medir_cliente, "PLAZO", 1.0)
    # Answers `initialize`, then reads forever without answering anything else.
    servidor = (
        "import sys, json\n"
        "l = sys.stdin.readline()\n"
        "print(json.dumps({'jsonrpc': '2.0', 'id': 1, 'result': {}}), flush=True)\n"
        "sys.stdin.read()\n"
    )
    c = ClienteMcp(Path(sys.executable), Path("."), ("-c", servidor))
    try:
        with pytest.raises(TimeoutError, match="no answer to tools/call"):
            c.tool("gui_observar", {"que": "ventanas"})
    finally:
        c.cerrar()
