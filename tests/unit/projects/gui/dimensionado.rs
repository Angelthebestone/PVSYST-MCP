use super::*;
use crate::format::object;

fn variante(n_serie: &str, n_cadenas: &str, pnom: Option<&str>) -> object::PvNode {
    let pnom = pnom.map_or(String::new(), |p| format!("  PNomSizing={p}\n"));
    let texto = format!(
        "PVObject_=pvVCalcul\n{pnom}  PVObject_=pvSubArray\n    NModSerie={n_serie}\n    NStringCh={n_cadenas}\n  End of PVObject pvSubArray\nEnd of PVObject pvVCalcul\n"
    );
    object::parse_first(&texto).expect("variante")
}

#[test]
fn should_flag_incomplete_when_saved_layout_differs_from_the_proposal() {
    let propuesta = json!({ "n_mod_serie": "18", "n_strings": "4" });

    let r = respuesta(&variante("17", "4", Some("28.5")), 30.0, &propuesta, &[]);

    assert_eq!(r["completo"], false);
    assert!(r["errores"][0]
        .as_str()
        .expect("error")
        .contains("NModSerie"));
}
