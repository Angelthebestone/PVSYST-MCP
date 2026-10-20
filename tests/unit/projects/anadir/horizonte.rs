use super::*;

fn demo(nombre: &str) -> Vec<u8> {
    std::fs::read(crate::config::proyectos_demo_ro().join(nombre)).expect("PVsyst demo data")
}

#[test]
#[ignore = "needs the PVsyst 8.1.7 demo data: run with --ignored"]
fn should_neutralize_horizon_removing_meteonorm_data() {
    let fuente = demo("_DEMO_Residential.VC1");
    let bloque = crate::format::insert::extraer_objeto(&fuente, "pvHorizon").expect("bloque");
    let neutro = neutralizar_horizonte(&bloque).expect("neutralizado");
    let texto = String::from_utf8_lossy(&neutro).to_ascii_lowercase();
    assert!(!texto.contains("meteonorm"));
    assert!(texto
        .lines()
        .filter(|l| l.trim().starts_with("point_"))
        .all(|l| l.trim().ends_with("=0.0,0.0")));
    assert!(texto.contains("nptseff=0"));
    assert!(texto.contains("fhhdiffus=1.000"));
}

#[test]
#[ignore = "needs the PVsyst 8.1.7 demo data: run with --ignored"]
fn should_replace_points_in_place_preserving_count() {
    let fuente = demo("_DEMO_Residential.VC1");
    let bloque = crate::format::insert::extraer_objeto(&fuente, "pvHorizon").expect("bloque");
    let neutro = neutralizar_horizonte(&bloque).expect("neutralizado");
    let mut cambios: Vec<(String, Option<String>)> = (1..=3)
        .map(|k| (format!("Point_{k}"), Some(format!("{k}.0,5.0"))))
        .collect();
    cambios.push(("NPtsEff".into(), Some("3".into())));
    let (con_puntos, _) = edit::edit_fields(&neutro, "pvHorizon", &cambios).expect("editar");
    let lineas_antes = neutro.split(|b| *b == b'\n').count();
    let lineas_despues = con_puntos.split(|b| *b == b'\n').count();
    assert_eq!(lineas_antes, lineas_despues, "ninguna clave nueva");
    let texto = String::from_utf8_lossy(&con_puntos).into_owned();
    let nodo = crate::format::object::parse_first(&texto).expect("parseable");
    assert_eq!(nodo.find_deep("Point_3"), Some("3.0,5.0"));
    assert_eq!(nodo.find_deep("Point_4"), Some("0.0,0.0"));
}
