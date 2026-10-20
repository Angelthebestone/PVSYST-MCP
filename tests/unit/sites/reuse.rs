use super::coincide;

const SIT: &str = "\u{feff}Source=NASA SSE grid cell 7/-74\r\nLatitude=7.0708\r\nLongitude=-73.8385\r\nAltitude=100\r\n";

#[test]
fn should_match_when_coordinates_and_nasa_source_match() {
    assert!(coincide(SIT, (7.0708, -73.8385), None));
}

#[test]
fn should_not_match_when_latitude_differs() {
    assert!(!coincide(SIT, (7.0, -73.8385), None));
}

#[test]
fn should_only_compare_altitude_when_provided() {
    assert!(!coincide(SIT, (7.0708, -73.8385), Some(250.0)));
    assert!(coincide(SIT, (7.0708, -73.8385), Some(100.0)));
}

#[test]
fn should_not_reuse_sites_from_other_sources() {
    let otro = SIT.replace("NASA SSE grid cell", "Otro origen");
    assert!(!coincide(&otro, (7.0708, -73.8385), None));
}
