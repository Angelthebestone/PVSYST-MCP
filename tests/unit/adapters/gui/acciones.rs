use super::*;

#[test]
fn should_click_buttons_with_bm_click_only_and_other_controls_with_the_mouse() {
    for boton in ["Button", "TButton", "tbitbtn"] {
        assert_eq!(
            mecanismo_para(boton),
            MecanismoClic::BotonEstandar,
            "{boton}"
        );
    }
    for otro in ["TPanel", "TListView", "SysTreeView32"] {
        assert_eq!(mecanismo_para(otro), MecanismoClic::Raton, "{otro}");
    }
}
