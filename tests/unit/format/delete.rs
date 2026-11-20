use super::*;

const SINTETICO: &[u8] = b"PVObject_Root=pvRoot\r\n  FieldTilt=20\r\n  FieldTilt=25\r\n  Name=x\r\nEnd of PVObject pvRoot\r\n";

#[test]
fn should_detect_and_remove_duplicate_keys() {
    let dup = claves_duplicadas(SINTETICO, "pvRoot");
    assert_eq!(
        dup,
        vec![(
            "FieldTilt".to_string(),
            vec!["20".to_string(), "25".to_string()]
        )]
    );
    let limpio = quitar_duplicados(SINTETICO, "pvRoot").expect("quitar");
    let texto = String::from_utf8(limpio).expect("utf8");
    assert_eq!(texto.matches("FieldTilt=").count(), 1);
    assert!(texto.contains("FieldTilt=20"));
    assert!(texto.contains("Name=x"));
    assert!(texto.ends_with("\r\n"));
}

#[test]
fn should_eliminate_object() {
    let doc = b"PVObject_Root=pvRoot\r\n  PVObject_Hijo=pvChild\r\n    X=1\r\n  End of PVObject pvChild\r\nEnd of PVObject pvRoot\r\n";
    let sin = eliminar_objeto(doc, "pvChild").expect("eliminar");
    let txt = String::from_utf8(sin).expect("utf8");
    assert!(!txt.contains("pvChild"));
    assert!(txt.contains("pvRoot"));
}
