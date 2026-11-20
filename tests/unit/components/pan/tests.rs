//! Tests for PAN parsing and legacy decoding.

use super::legacy::dominio::*;
use super::*;
use crate::error::Error;
use crate::format::real48::BYTES_POR_SLOT;

#[test]
fn should_reject_legacy_block_when_slots_are_truncated_or_misaligned() {
    let header = "pvModule;Soitec\tCX-M500\tManufacturer;\r\n\
                  Version 5.67; Calcul=$00003043;\r\n\
                  GaInP2/GaAs/Ge;240;10;2;0.000;0.000;\r\n";
    // 24 slots (144 bytes) is fewer than SLOTS_LEGACY_REQUERIDOS (25).
    let mut truncated = header.as_bytes().to_vec();
    truncated.extend(std::iter::repeat_n(0x81, 24 * BYTES_POR_SLOT));
    truncated.push(b'/');
    // 25 slots (150 bytes) plus one stray byte is not a whole number of slots.
    let mut misaligned = header.as_bytes().to_vec();
    misaligned.extend(std::iter::repeat_n(0x81, 25 * BYTES_POR_SLOT + 1));
    misaligned.push(b'/');

    for bytes in [truncated, misaligned] {
        match parse_legacy(&bytes, header) {
            Err(Error::Format(msg)) => assert!(msg.contains("PAN legacy incompleto"), "{msg}"),
            other => panic!("expected Error::Format, got {other:?}"),
        }
    }
}

#[test]
fn should_decode_text_module_fixture() {
    let fixture = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/tests/fixtures/workspace/ComposPV/PVModules/Synth_400W.PAN"
    ));
    let val = parse_pan(fixture.as_bytes()).expect("parse pan");
    assert_eq!(val["format"], "pvsyst8-text");
    assert_eq!(val["vendor"], "Synthetic");
    assert_eq!(val["model"], "Synth 400W");
    assert_eq!(val["pnom"], 400.0);
    assert_eq!(val["voc"], 49.5);
    assert_eq!(val["isc"], 10.2);
}
