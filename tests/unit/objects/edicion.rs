use super::*;

const OBJETO: &[u8] =
    b"PVObject_=pvModule\r\n  PNom=300\r\n  Width=1.0\r\nEnd of PVObject pvModule\r\n";

fn cambio(k: &str, v: &str) -> Vec<(String, Option<String>)> {
    vec![(k.to_string(), Some(v.to_string()))]
}

#[test]
fn should_refuse_the_edit_when_the_field_reads_back_differently() {
    let res = verificar_span(OBJETO, "pvModule", &cambio("PNom", "310"));

    assert!(matches!(res, Err(Error::Format(_))), "{res:?}");
}
