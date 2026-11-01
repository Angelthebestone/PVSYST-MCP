use super::*;
use crate::format::object;

fn variante(horizonte: &str) -> PvNode {
    let texto = format!("PVObject_=pvVCalcul\n{horizonte}End of PVObject pvVCalcul\n");
    object::parse_first(&texto).expect("variante")
}

#[test]
fn should_report_imported_horizon_when_points_and_factors_were_saved() {
    let raiz = variante(
        "  PVObject_=pvHorizon\n    NPtsEff=36\n    FHHDiffus=0.97\n    FHPAlbedo=0.9\n  End of PVObject pvHorizon\n",
    );

    let r = respuesta(&raiz, &[]).expect("horizonte");

    assert_eq!(r["puntos"], "36");
    assert_eq!(r["FHHDiffus"], "0.97");
    assert_eq!(r["FHPDiffus"], Value::Null);
}

#[test]
fn should_fail_when_saved_horizon_has_no_points_or_no_factors() {
    for cuerpo in ["    FHHDiffus=0.97\n", "    NPtsEff=36\n"] {
        let raiz = variante(&format!(
            "  PVObject_=pvHorizon\n{cuerpo}  End of PVObject pvHorizon\n"
        ));
        assert!(
            matches!(respuesta(&raiz, &[]), Err(Error::Gui(_))),
            "{cuerpo}"
        );
    }
}

#[test]
fn should_fail_when_variant_has_no_horizon() {
    assert!(matches!(
        respuesta(&variante(""), &[]),
        Err(Error::Format(_))
    ));
}
