use super::*;

#[test]
fn should_round_trip_latin1_and_refuse_wider_characters() {
    assert_eq!(codificar_latin1("Español"), Some(b"Espa\xf1ol".to_vec()));
    assert_eq!(decodificar_latin1(b"Espa\xf1ol"), "Español");
    assert_eq!(codificar_latin1("€"), None);
}
