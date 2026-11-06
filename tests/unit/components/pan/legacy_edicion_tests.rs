//! Tests for legacy Delphi Real48 roundtrip fidelity and in-place byte editing.

use super::dominio::*;
use super::edicion::*;
use crate::format::real48::*;

const FIXTURE_LEGACY_PAN: &[u8] = include_bytes!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/tests/fixtures/workspace/ComposPV/PVModules/Synth_Legacy_300W.PAN"
));

#[test]
fn should_roundtrip_encode_decode_for_fixture_slots() {
    let start = block1_start(FIXTURE_LEGACY_PAN).expect("3rd CRLF in fixture");
    let term = FIXTURE_LEGACY_PAN[start..]
        .iter()
        .position(|&b| b == 0x2F)
        .map(|p| start + p)
        .expect("block terminator in fixture");

    let n_slots = (term - start) / BYTES_POR_SLOT;
    assert_eq!(n_slots, 25, "fixture must have 25 slots");

    for i in 0..n_slots {
        let offset = start + i * BYTES_POR_SLOT;
        let mut slot_bytes = [0u8; 6];
        slot_bytes.copy_from_slice(&FIXTURE_LEGACY_PAN[offset..offset + 6]);

        let decoded = real48(&slot_bytes);
        let re_encoded = encode_real48(decoded).expect("re-encoding must succeed");
        assert_eq!(
            re_encoded, slot_bytes,
            "Slot {i} roundtrip mismatch: orig={slot_bytes:?}, dec={decoded}, re={re_encoded:?}"
        );
    }
}

#[test]
fn should_patch_single_field_leaving_all_other_bytes_identical() {
    let original = FIXTURE_LEGACY_PAN;
    let start = block1_start(original).expect("block1 start");
    let slot0_offset = start;

    let changes = vec![("pnom_w".to_string(), Some("350.0".to_string()))];
    let (patched, outcomes) =
        parchear_campos_legacy(original, &changes).expect("patching pnom_w must succeed");

    assert_eq!(patched.len(), original.len(), "length must not change");
    assert_eq!(outcomes.len(), 1);
    assert_eq!(outcomes[0].key, "pnom_w");

    // Bytes strictly before slot 0 must be 100% identical.
    assert_eq!(&patched[..slot0_offset], &original[..slot0_offset]);

    // Bytes strictly after slot 0 must be 100% identical.
    assert_eq!(
        &patched[slot0_offset + BYTES_POR_SLOT..],
        &original[slot0_offset + BYTES_POR_SLOT..]
    );

    // Slot 0 itself must decode to 350.0.
    let mut slot0 = [0u8; 6];
    slot0.copy_from_slice(&patched[slot0_offset..slot0_offset + 6]);
    assert_eq!(real48(&slot0), 350.0);

    // Re-decode with parse_legacy and verify.
    let text = String::from_utf8_lossy(&patched);
    let parsed = parse_legacy(&patched, &text).expect("re-parsing patched legacy");
    assert_eq!(parsed["pnom"], 350.0);
    assert_eq!(parsed["voc"], 40.0, "voc must remain unchanged");
    assert_eq!(parsed["isc"], 9.5, "isc must remain unchanged");
}

#[test]
fn should_edit_legacy_pan_via_editar_objeto() {
    let temp = tempfile::tempdir().expect("tempdir");
    let mods_dir = temp.path().join("ComposPV").join("PVModules");
    std::fs::create_dir_all(&mods_dir).expect("create dir");
    let pan_path = mods_dir.join("Synth_Legacy_300W.PAN");
    std::fs::write(&pan_path, FIXTURE_LEGACY_PAN).expect("write");

    let paths = crate::config::Paths::para_test(std::env::temp_dir());
    let args = serde_json::json!({
        "workspace": temp.path().to_string_lossy(),
        "archivo": "ComposPV/PVModules/Synth_Legacy_300W.PAN",
        "campos": { "pnom_w": 315.0 }
    });

    let res = crate::objects::editar_objeto(&paths, &args).expect("editar_objeto");
    assert_eq!(res["object"], "pvModule");
    assert!(res["historial_id"].is_string());

    let updated_bytes = std::fs::read(&pan_path).expect("read");
    let parsed = crate::components::pan::parse_pan(&updated_bytes).expect("parse_pan");
    assert_eq!(parsed["pnom"], 315.0);
    assert_eq!(parsed["voc"], 40.0);
}
