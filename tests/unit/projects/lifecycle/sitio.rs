use super::*;

const PRJ: &str = "Version=8.1.0\r\nMeteoFileName=Geneva_MN90_SYN.MET\r\nPVObject_=pvProject\r\n  MeteoFileName=Nested.MET\r\nEnd of PVObject pvProject\r\n";

#[test]
fn should_read_the_weather_file_of_the_project() {
    assert_eq!(
        meteo_del_proyecto(PRJ.as_bytes()).as_deref(),
        Some("Geneva_MN90_SYN.MET")
    );
}

#[test]
fn should_read_the_weather_file_of_a_project_on_disk() {
    let dir = tempfile::tempdir().expect("dir");
    let ruta = dir.path().join("P.PRJ");
    std::fs::write(&ruta, PRJ).expect("prj");

    assert_eq!(
        meteo_del_proyecto_en(&ruta).as_deref(),
        Some("Geneva_MN90_SYN.MET")
    );
}

#[test]
fn should_report_no_weather_file_when_the_project_has_none() {
    let dir = tempfile::tempdir().expect("dir");
    let ruta = dir.path().join("P.PRJ");
    std::fs::write(&ruta, "Version=8.1.0\r\n").expect("prj");

    assert_eq!(meteo_del_proyecto_en(&ruta), None);
}
