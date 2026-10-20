use super::*;
use proptest::prelude::*;

const SINTETICO: &[u8] =
    b"PVObject_Root=pvRoot\r\n  FieldTilt=20\r\n  Name=x\r\nEnd of PVObject pvRoot\r\n";

#[test]
fn should_insert_and_rebase_indentation() {
    let bloque = b"PVObject_Old=pvChild\n  X=1\nEnd of PVObject pvChild\n";
    let out = insertar_objeto(
        SINTETICO,
        "pvRoot",
        bloque,
        Some("Hijo"),
        SiExiste::Rechazar,
    )
    .expect("insertar");
    let texto = String::from_utf8(out.clone()).expect("utf8");
    assert!(texto.contains("  PVObject_Hijo=pvChild\r\n"));
    assert!(texto.contains("    X=1\r\n"));
    let spans = edit::spans_of(&out);
    let hijo = spans.iter().find(|s| s.ty == "pvChild").expect("hijo");
    assert_eq!(hijo.indent, 2);
    assert_eq!(hijo.role.as_deref(), Some("Hijo"));
}

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
fn should_refuse_insert_when_block_unbalanced() {
    let trunc_bloque = b"PVObject_Hijo=pvChild\n  X=1\n";
    let err =
        insertar_objeto(SINTETICO, "pvRoot", trunc_bloque, None, SiExiste::Rechazar).unwrap_err();
    assert!(matches!(err, Error::Format(_)));
    assert!(err.to_string().contains("sin cerrar"));
}

#[test]
fn should_refuse_insert_when_selector_invalid() {
    let bloque = b"PVObject_Hijo=pvChild\nEnd of PVObject pvChild\n";
    let err =
        insertar_objeto(SINTETICO, "pvRoot[abc]", bloque, None, SiExiste::Rechazar).unwrap_err();
    assert!(matches!(err, Error::Invalid(_)));
}

#[test]
fn should_extract_object_without_bom() {
    let mut con_bom = BOM.to_vec();
    con_bom.extend_from_slice(SINTETICO);
    let extraido = extraer_objeto(&con_bom, "pvRoot").expect("extraer");
    assert!(!extraido.starts_with(BOM));
    assert_eq!(extraido, SINTETICO);
}

fn campo() -> impl Strategy<Value = String> {
    ("[A-Za-z][A-Za-z0-9]{0,5}", "[A-Za-z0-9]{0,12}")
        .prop_map(|(clave, valor)| format!("{clave}={valor}"))
}

/// Valid PVObject documents (root plus N `pvX{i}` children) and a sibling block.
fn documentos_y_bloque() -> impl Strategy<Value = (String, String)> {
    (0usize..=6).prop_flat_map(|n| {
        let hijos = prop::collection::vec(prop::collection::vec(campo(), 0..=5), n);
        let bloque = prop::collection::vec(campo(), 0..=5);
        (hijos, bloque).prop_map(move |(hijos, bloque)| {
            let mut doc = String::from("PVObject_=pvRoot\r\n");
            for (i, campos) in hijos.iter().enumerate() {
                doc.push_str(&format!("  PVObject_=pvX{i}\r\n"));
                for campo in campos {
                    doc.push_str(&format!("    {campo}\r\n"));
                }
                doc.push_str(&format!("  End of PVObject pvX{i}\r\n"));
            }
            doc.push_str("End of PVObject pvRoot\r\n");

            let mut nuevo = String::from("PVObject_=pvNuevo\r\n");
            for campo in &bloque {
                nuevo.push_str(&format!("{campo}\r\n"));
            }
            nuevo.push_str("End of PVObject pvNuevo\r\n");
            (doc, nuevo)
        })
    })
}

fn reindentado(bloque: &str) -> String {
    bloque
        .lines()
        .map(|linea| format!("  {linea}\r\n"))
        .collect()
}

proptest! {
    #[test]
    fn inserted_object_roundtrips_through_extraction((doc, bloque) in documentos_y_bloque()) {
        let insertado = insertar_objeto(
            doc.as_bytes(),
            "pvRoot",
            bloque.as_bytes(),
            None,
            SiExiste::Rechazar,
        )
        .expect("insertar");
        let extraido = extraer_objeto(&insertado, "pvNuevo").expect("extraer");
        prop_assert_eq!(String::from_utf8(extraido).expect("utf8"), reindentado(&bloque));
    }

    #[test]
    fn inserted_object_can_be_removed_to_restore_the_document((doc, bloque) in documentos_y_bloque()) {
        let insertado = insertar_objeto(
            doc.as_bytes(),
            "pvRoot",
            bloque.as_bytes(),
            None,
            SiExiste::Rechazar,
        )
        .expect("insertar");
        let eliminado = eliminar_objeto(&insertado, "pvNuevo").expect("eliminar");
        prop_assert_eq!(String::from_utf8(eliminado).expect("utf8"), doc);
    }
}
