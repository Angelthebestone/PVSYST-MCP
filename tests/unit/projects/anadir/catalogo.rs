use super::*;

const VARIANTE: &[u8] = b"PVObject_=pvVCalcul\r\n  PVObject_Horizon=pvHorizon\r\n    NPtsEff=0\r\n  End of PVObject pvHorizon\r\nEnd of PVObject pvVCalcul\r\n";
const BLOQUE: &[u8] = b"PVObject_=pvHorizon\r\n  NPtsEff=12\r\nEnd of PVObject pvHorizon\r\n";

#[test]
fn should_not_report_type_as_inserted_when_variant_already_has_it() {
    let (bytes, insertado) =
        insertar_si_falta(VARIANTE.to_vec(), ("pvHorizon", "pvVCalcul"), BLOQUE)
            .expect("sin cambio");

    assert!(!insertado);
    assert_eq!(bytes, VARIANTE);
}
