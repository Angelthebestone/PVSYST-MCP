#!/usr/bin/env python3
"""
Reference decoder for legacy (PVsyst < 6.40) binary .PAN module files.

This is the *reference* implementation of the same algorithm that the Rust MCP
implements in `src/components/pan.rs` (function `real48`). It exists so the
recovered format can be inspected and re-validated independently (e.g. with
`python tools/pan_decode.py "C:\\...\\CX-M500_2450Wp_1_00_086.PAN"`).

The format was recovered from the data alone (plus public documentation and the
open-source CASSYS binary-PAN reader); no PVsyst executable was decompiled.

Format:
  * ASCII line 1 : pvModule;<Manufacturer>\t<Model>\t<DataSource>;
  * ASCII line 2 : Version <v>; Calcul=$<flags>;
  * ASCII line 3 : <Technol>;<NCelS>;<NCelP>;<NDiode>;<x>;<y>;
  * BINARY block1: N x 6-byte Delphi Real48 values (module electrical record)
  * ASCII        : \x2f CRLF pvCommercial;; CRLF Version ...; Calcul=...; CRLF ; CRLF EUR;;; CRLF
  * BINARY block2: 6-byte Real48 values (commercial/dimensions record)
  * ASCII        : \x2f CRLF End Remarks CRLF

Delphi Real48 (6 bytes, b0..b5) -> double:
    if b0 == 0: return 0.0
    value = sign * (1 + mantissa/2**39) * 2**(b0 - 129)
      where sign     = -1 if (b5 & 0x80) else +1
            mantissa = b1 + b2<<8 + b3<<16 + b4<<24 + (b5 & 0x7F)<<32
"""

import sys
from pathlib import Path
from typing import Any

REAL48_SIZE = 6
HEADER_END = b"0.000;\r\n"
COMMERCIAL_START = b"EUR;;;\r\n"
BLOCK_END = b"\x2f\r\n"

# Field layout of the module record (block1), 0-based 6-byte slot.
# Names verified against the CASSYS binary-PAN reader (CanadianSolar/CASSYS
# DatabaseImportModule.vba) and cross-checked against module datasheets.
MODULE_FIELDS = {
    0: "PNom", 1: "VMaxIEC", 2: "Tolerance", 3: "AreaM", 4: "CellArea",
    5: "Gref", 6: "Tref", 7: "Absorb?", 8: "Isc", 9: "muISC",
    10: "Voc", 11: "muVocSpec", 12: "Imp", 13: "Vmp", 14: "VRevDiode",
    17: "Rshunt", 18: "Rserie", 23: "Rp_0", 24: "Rp_Exp", 25: "muPmpReq",
}  # fmt: skip
# Field layout of the commercial record (block2)
COMM_FIELDS = {0: "Width", 1: "Height", 2: "Depth", 3: "Weight"}


def real48(b: bytes) -> float:
    """Decode one 6-byte Delphi Real48 value."""
    if len(b) != REAL48_SIZE:
        raise ValueError(f"Real48 needs {REAL48_SIZE} bytes, got {len(b)}")
    if b[0] == 0:
        return 0.0
    mantissa = int.from_bytes(b[1:5] + bytes([b[5] & 0x7F]), "little")
    sign = -1.0 if b[5] & 0x80 else 1.0
    return sign * (1.0 + mantissa / 2**39) * 2.0 ** (b[0] - 129)


def decode_record(
    data: bytes, start: int, end: int, names: dict[int, str]
) -> dict[str, float]:
    """Decode the Real48 slots in data[start:end], naming known slots."""
    return {
        names.get(k, f"f{k:02d}"): real48(data[off : off + REAL48_SIZE])
        for k, off in enumerate(range(start, end - REAL48_SIZE + 1, REAL48_SIZE))
    }


def find_block1(data: bytes) -> int:
    """Offset of the first binary block: two bytes past the 3rd CR."""
    cr_positions = (i for i, b in enumerate(data) if b == 13)
    for count, pos in enumerate(cr_positions, start=1):
        if count == 3:
            return pos + 2
    raise ValueError("no 3rd CR found")


def parse_pan(path: str) -> dict[str, Any]:
    """Parse a legacy binary .PAN file into header, module and commercial records."""
    data = Path(path).read_bytes()

    header_end = data.index(HEADER_END) + len(HEADER_END)
    header = [h.decode("latin-1") for h in data[:header_end].split(b"\r\n") if h]

    b1 = find_block1(data)
    term1 = data.index(BLOCK_END, b1)
    b2 = data.index(COMMERCIAL_START) + len(COMMERCIAL_START)
    term2 = data.index(BLOCK_END, b2)

    return {
        "file": path,
        "size": len(data),
        "header_lines": header,
        "block1": {"start": b1, "end": term1 - 1, "length": term1 - b1},
        "module": decode_record(data, b1, term1, MODULE_FIELDS),
        "block2": {"start": b2, "end": term2 - 1, "length": term2 - b2},
        "commercial": decode_record(data, b2, term2, COMM_FIELDS),
    }


def print_report(r: dict[str, Any]) -> None:
    """Print a human-readable dump of a parsed .PAN."""
    print("=" * 72)
    print(r["file"], r["size"], "bytes")
    print("block1 =", r["block1"])
    print("block2 =", r["block2"])
    for title, record in (("module", r["module"]), ("commercial", r["commercial"])):
        print(f"-- {title} --")
        for k, v in record.items():
            print(f"   {k:>10} = {v!r}")


def main(argv: list[str]) -> int:
    """CLI entry point: decode every .PAN path given on the command line."""
    for p in argv:
        try:
            print_report(parse_pan(p))
        except (OSError, ValueError) as e:
            print(f"{p}: {e}", file=sys.stderr)
            return 1
    return 0


if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))
