"""Unit tests for the reference Real48 / .PAN decoder (mirror of module.rs tests)."""

import pytest

from pan_decode import decode_record, find_block1, real48


@pytest.mark.parametrize(
    ("raw", "expected"),
    [
        (b"\x00\x00\x00\x00\x00\x00", 0.0),
        (b"\x81\x00\x00\x00\x00\x00", 1.0),
        (b"\x82\x00\x00\x00\x00\x00", 2.0),
        (b"\x80\x00\x00\x00\x00\x00", 0.5),
        (b"\x81\x00\x00\x00\x00\x80", -1.0),
    ],
)
def test_real48_known_values(raw: bytes, expected: float) -> None:
    assert real48(raw) == expected


def test_real48_rejects_wrong_length() -> None:
    with pytest.raises(ValueError):
        real48(b"\x81\x00")


def test_decode_record_names_known_slots_and_ignores_tail() -> None:
    data = b"\x81" + b"\x00" * 5 + b"\x82" + b"\x00" * 5 + b"\xff"
    assert decode_record(data, 0, len(data), {0: "A"}) == {"A": 1.0, "f01": 2.0}


def test_find_block1() -> None:
    assert find_block1(b"a\r\nb\r\nc\r\nXY") == 9
    with pytest.raises(ValueError):
        find_block1(b"a\r\nb\r\n")
