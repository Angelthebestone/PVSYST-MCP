//! Tests for PAN parsing and legacy decoding.

use super::legacy::*;
use super::*;
use crate::error::Error;

#[test]
fn should_decode_real48_known_values() {
    assert_eq!(real48(&[0, 0, 0, 0, 0, 0]), 0.0);
    assert_eq!(real48(&[0x81, 0, 0, 0, 0, 0]), 1.0);
    assert_eq!(real48(&[0x82, 0, 0, 0, 0, 0]), 2.0);
    assert_eq!(real48(&[0x80, 0, 0, 0, 0, 0]), 0.5);
    assert_eq!(real48(&[0x81, 0, 0, 0, 0, 0x80]), -1.0);
}

#[test]
fn should_parse_valid_legacy_header() {
    let text = "pvModule;Soitec\tCX-M500 1.00.086\tManufacturer;\n\
                Version 5.67; Calcul=$00003043;\n\
                GaInP2/GaAs/Ge;240;10;2;0.000;0.000;\n";
    let h = parse_legacy_header(text).expect("encabezado válido");
    assert_eq!(h.vendor, "Soitec");
    assert_eq!(h.model, "CX-M500 1.00.086");
    assert_eq!(h.data_source, "Manufacturer");
    assert_eq!(h.version, "5.67");
    assert_eq!(h.technology, "GaInP2/GaAs/Ge");
    assert_eq!(h.cells_series, 240);
    assert_eq!(h.cells_parallel, 10);
    assert_eq!(h.diodes, 2);
}

#[test]
fn should_reject_legacy_header_when_cells_series_is_non_numeric() {
    let text = "pvModule;Soitec\tCX-M500 1.00.086\tManufacturer;\n\
                Version 5.67; Calcul=$00003043;\n\
                GaInP2/GaAs/Ge;abc;10;2;0.000;0.000;\n";
    let err = parse_legacy_header(text).unwrap_err();
    match err {
        Error::Format(msg) => {
            assert!(
                msg.contains("Valor no numérico para cells_series"),
                "msg={msg}"
            );
        }
        other => panic!("expected Error::Format, got {other:?}"),
    }
}

#[test]
fn should_reject_legacy_header_when_cells_parallel_is_non_numeric() {
    let text = "pvModule;Soitec\tCX-M500 1.00.086\tManufacturer;\n\
                Version 5.67; Calcul=$00003043;\n\
                GaInP2/GaAs/Ge;240;xyz;2;0.000;0.000;\n";
    let err = parse_legacy_header(text).unwrap_err();
    match err {
        Error::Format(msg) => {
            assert!(
                msg.contains("Valor no numérico para cells_parallel"),
                "msg={msg}"
            );
        }
        other => panic!("expected Error::Format, got {other:?}"),
    }
}

#[test]
fn should_reject_legacy_header_when_diodes_is_non_numeric() {
    let text = "pvModule;Soitec\tCX-M500 1.00.086\tManufacturer;\n\
                Version 5.67; Calcul=$00003043;\n\
                GaInP2/GaAs/Ge;240;10;dos;0.000;0.000;\n";
    let err = parse_legacy_header(text).unwrap_err();
    match err {
        Error::Format(msg) => {
            assert!(msg.contains("Valor no numérico para diodes"), "msg={msg}");
        }
        other => panic!("expected Error::Format, got {other:?}"),
    }
}

#[test]
fn should_reject_legacy_block_when_fewer_slots_than_required() {
    let header = "pvModule;Soitec\tCX-M500\tManufacturer;\r\n\
                  Version 5.67; Calcul=$00003043;\r\n\
                  GaInP2/GaAs/Ge;240;10;2;0.000;0.000;\r\n";
    let mut bytes = header.as_bytes().to_vec();
    // 24 slots (144 bytes) is fewer than SLOTS_LEGACY_REQUERIDOS (25)
    bytes.extend(std::iter::repeat_n(0x81, 24 * BYTES_POR_SLOT));
    bytes.push(b'/');

    let err = parse_legacy(&bytes, header).unwrap_err();
    match err {
        Error::Format(msg) => {
            assert_eq!(
                msg,
                "PAN legacy incompleto: 144 bytes / 24 slots, se esperaban 25"
            );
        }
        other => panic!("expected Error::Format, got {other:?}"),
    }
}

#[test]
fn should_reject_legacy_block_when_not_multiple_of_6() {
    let header = "pvModule;Soitec\tCX-M500\tManufacturer;\r\n\
                  Version 5.67; Calcul=$00003043;\r\n\
                  GaInP2/GaAs/Ge;240;10;2;0.000;0.000;\r\n";
    let mut bytes = header.as_bytes().to_vec();
    // 25 slots (150 bytes) + 1 extra byte = 151 bytes
    bytes.extend(std::iter::repeat_n(0x81, 25 * BYTES_POR_SLOT + 1));
    bytes.push(b'/');

    let err = parse_legacy(&bytes, header).unwrap_err();
    match err {
        Error::Format(msg) => {
            assert_eq!(
                msg,
                "PAN legacy incompleto: 151 bytes / 25 slots, se esperaban 25"
            );
        }
        other => panic!("expected Error::Format, got {other:?}"),
    }
}

#[test]
fn should_decode_valid_legacy_block_identically() {
    let header = "pvModule;Soitec\tCX-M500 1.00.086\tManufacturer;\r\n\
                  Version 5.67; Calcul=$00003043;\r\n\
                  GaInP2/GaAs/Ge;240;10;2;0.000;0.000;\r\n";
    let mut bytes = header.as_bytes().to_vec();
    // Exactly 25 slots with zero Real48 (0.0)
    bytes.extend(std::iter::repeat_n(0, 25 * BYTES_POR_SLOT));
    bytes.push(b'/');

    let val = parse_legacy(&bytes, header).expect("decodificación exitosa");
    assert_eq!(val["vendor"], "Soitec");
    assert_eq!(val["model"], "CX-M500 1.00.086");
    assert_eq!(val["cells_series"], 240);
    assert_eq!(val["cells_parallel"], 10);
    assert_eq!(val["diodes"], 2);
    assert_eq!(val["pnom"], 0.0);
    assert_eq!(val["raw_slots"].as_array().map(|a| a.len()), Some(25));
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

#[test]
fn should_reject_module_outside_workspace_without_libre_scope() {
    let paths = Paths::resolve();
    let outside = std::env::temp_dir().join("outside.PAN");
    let res = resolve_module_file(
        &paths.workspace,
        &paths,
        &outside.to_string_lossy(),
        Alcance::Workspace,
    );
    assert!(res.is_err());
}
