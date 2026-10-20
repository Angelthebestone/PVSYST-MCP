#!/usr/bin/env python3
"""
Generate the synthetic legacy (.PAN Real48) fixture used by the Rust tests.

The fixture has the three ASCII header lines `parse_legacy_header` expects and
a 25-slot binary block terminated by a slash, exactly as `parse_legacy` reads it.
Run it from anywhere; the default destination is the bundled test workspace.
"""

import sys
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent))

from pan_decode import encode_real48  # noqa: E402

HEADER = (
    "pvModule;SynthSolar\tSynth_Legacy_300W\tSynthetic;\r\n"
    "Version 6.39; Calcul=$00000000;\r\n"
    "Mono-Si;60;6;3;0.000;0.000;\r\n"
).encode("ascii")
SLOTS = 25
VALUES = {0: 300.0, 8: 9.5, 10: 40.0, 12: 9.09, 13: 33.0}
TERMINATOR = b"/"


def build_fixture() -> bytes:
    """Return the complete synthetic legacy PAN file."""
    block = b"".join(encode_real48(VALUES.get(slot, 0.0)) for slot in range(SLOTS))
    return HEADER + block + TERMINATOR


def main(argv: list[str]) -> int:
    """Write the fixture to `argv[0]` or to the bundled test workspace."""
    project = Path(__file__).resolve().parent.parent
    default = (
        project
        / "tests"
        / "fixtures"
        / "workspace"
        / "ComposPV"
        / "PVModules"
        / "Synth_Legacy_300W.PAN"
    )
    destination = Path(argv[0]) if argv else default
    destination.parent.mkdir(parents=True, exist_ok=True)
    destination.write_bytes(build_fixture())
    print(destination)
    return 0


if __name__ == "__main__":
    raise SystemExit(main(sys.argv[1:]))
