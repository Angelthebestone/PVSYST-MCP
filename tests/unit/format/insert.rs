use super::*;

const SINTETICO: &[u8] =
    b"PVObject_Root=pvRoot\r\n  FieldTilt=20\r\n  Name=x\r\nEnd of PVObject pvRoot\r\n";

#[test]
fn should_require_reemplazar_when_child_exists() {
    let bloque = b"PVObject_Hijo=pvChild\nX=1\nEnd of PVObject pvChild\n";
    let primera =
        insertar_objeto(SINTETICO, "pvRoot", bloque, None, SiExiste::Rechazar).expect("primera");
    let err = insertar_objeto(&primera, "pvRoot", bloque, None, SiExiste::Rechazar).unwrap_err();
    assert_eq!(err.to_string(), "Ya existe pvChild");
    let reemplazo =
        insertar_objeto(&primera, "pvRoot", bloque, None, SiExiste::Reemplazar).expect("reemplazo");
    let n = edit::spans_of(&reemplazo)
        .iter()
        .filter(|s| s.ty == "pvChild")
        .count();
    assert_eq!(n, 1);
}

#[test]
fn should_refuse_insert_when_destination_unbalanced() {
    let trunc_dest = b"PVObject_Root=pvRoot\r\n  Name=x\r\n";
    let bloque = b"PVObject_Hijo=pvChild\nEnd of PVObject pvChild\n";
    let err = insertar_objeto(trunc_dest, "pvRoot", bloque, None, SiExiste::Rechazar).unwrap_err();
    assert!(matches!(err, Error::Format(_)));
    assert!(err.to_string().contains("sin cerrar"));
}

#[test]
fn should_replace_object_and_keep_destination_indentation() {
    let doc = b"PVObject_=pvRoot\r\n  PVObject_=pvChild\r\n    Old=1\r\n  End of PVObject pvChild\r\nEnd of PVObject pvRoot\r\n";
    let bloque = b"PVObject_=pvChild\n  New=2\nEnd of PVObject pvChild\n";

    let out = reemplazar_objeto(doc, "pvChild", bloque).expect("reemplazar");
    let texto = String::from_utf8(out).expect("utf8");
    assert!(texto.contains("  PVObject_=pvChild\r\n    New=2\r\n  End of PVObject pvChild"));
    assert!(!texto.contains("Old=1"));
}

#[test]
fn should_reject_role_that_would_forge_a_header() {
    let bloque = b"PVObject_Old=pvChild\n  X=1\nEnd of PVObject pvChild\n";
    for rol in ["A=pvX", "A\nPVObject_B", "A\r", "", "con espacio"] {
        let res = insertar_objeto(SINTETICO, "pvRoot", bloque, Some(rol), SiExiste::Rechazar);
        assert!(matches!(res, Err(Error::Invalid(_))), "{rol:?}");
    }
}
