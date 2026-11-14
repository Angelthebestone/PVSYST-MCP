use super::*;
use crate::format::object;

fn variante(horizonte: &str) -> PvNode {
    let texto = format!("PVObject_=pvVCalcul\n{horizonte}End of PVObject pvVCalcul\n");
    object::parse_first(&texto).expect("variante")
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
