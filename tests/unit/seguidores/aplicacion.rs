use super::*;

const CON_ESCENA: &[u8] = b"PVObject_=pvVCalcul\r\n  PVObject_=pvShading\r\n    ListeObjets, list of=1 tObjet\r\n  End of PVObject pvShading\r\nEnd of PVObject pvVCalcul\r\n";

#[test]
fn should_refuse_a_variant_with_a_3d_scene() {
    assert!(matches!(
        rechazar_escena_3d(CON_ESCENA),
        Err(Error::Invalid(_))
    ));
}

#[test]
fn should_accept_a_variant_without_shading_object() {
    let sin_sombras = b"PVObject_=pvVCalcul\r\n  Comment=x\r\nEnd of PVObject pvVCalcul\r\n";

    assert!(rechazar_escena_3d(sin_sombras).is_ok());
    assert_eq!(
        texto_de_objeto(sin_sombras, "pvShading").expect("sin error"),
        None
    );
}

#[test]
fn should_refuse_to_convert_an_unbalanced_variant() {
    let ws = tempfile::tempdir().expect("ws");
    let dir = ws.path().join("Projects");
    std::fs::create_dir_all(&dir).expect("Projects");
    std::fs::write(
        dir.join("P.PRJ"),
        "PVObject_=pvProject\r\nEnd of PVObject pvProject\r\n",
    )
    .expect("prj");
    std::fs::write(
        dir.join("P.VC0"),
        b"PVObject_=pvVCalcul\r\n  PVObject_=pvShading\r\n    ListeObjets, list of=1 tObjet\r\n",
    )
    .expect("vc0");
    let paths = Paths::para_test(ws.path());
    let args = json!({ "proyecto": "P", "variante": "VC0" });

    let res = convertir_a_seguidor(&paths, &args);

    assert!(matches!(res, Err(Error::Format(_))), "{res:?}");
    assert!(std::fs::read(dir.join("P.VC0"))
        .expect("vc0")
        .ends_with(b"tObjet\r\n"));
}
