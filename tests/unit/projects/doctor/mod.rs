use super::*;
use crate::config::Paths;
use serde_json::json;

const PRJ: &str = "PVObject_=pvProject\r\n  Comment=Planta\r\nEnd of PVObject pvProject\r\n";
const VC0: &str = "PVObject_=pvVCalcul\r\n  Comment=Planta\r\nEnd of PVObject pvVCalcul\r\n";

#[test]
fn should_report_an_unreadable_variant_and_still_check_the_others() {
    let ws = tempfile::tempdir().expect("ws");
    let dir = ws.path().join("Projects");
    std::fs::create_dir_all(dir.join("Planta.VC1")).expect("variante ilegible");
    std::fs::write(dir.join("Planta.PRJ"), PRJ).expect("prj");
    std::fs::write(dir.join("Planta.VC0"), VC0).expect("vc0");
    let paths = Paths::para_test(ws.path());

    let informe = doctor(&paths, &json!({ "proyecto": "Planta" })).expect("informe, no error");

    assert_eq!(informe["ok"], false);
    let errores = informe["errores"].to_string();
    assert!(errores.contains("Planta.VC1"), "{errores}");
    assert!(!errores.contains("Planta.VC0"), "{errores}");
}
