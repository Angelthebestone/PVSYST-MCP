use super::*;

#[test]
fn should_report_error_when_string_voltage_exceeds_limits() {
    let mut inf = Informe::default();
    let d = DatosElectricos {
        ns: 30.0,
        tmin: -10.0,
        tmax: 60.0,
        voc: 50.0,
        vmpp: 41.0,
        pnom: 300.0,
        vabs: 1000.0,
        vmin: 500.0,
        vmax: 900.0,
        pconv: 10000.0,
    };
    comprobar_tensiones(&d, -0.0029, &mut inf);
    assert!(inf.errores.iter().any(|e| e.contains("Voc en frío")));
}

#[test]
fn should_pass_when_string_voltage_is_within_limits() {
    let mut inf = Informe::default();
    let d = DatosElectricos {
        ns: 10.0,
        tmin: -10.0,
        tmax: 60.0,
        voc: 40.0,
        vmpp: 33.0,
        pnom: 300.0,
        vabs: 1000.0,
        vmin: 250.0,
        vmax: 900.0,
        pconv: 10000.0,
    };
    comprobar_tensiones(&d, -0.0029, &mut inf);
    assert!(inf.errores.is_empty(), "{:?}", inf.errores);
}

#[test]
fn should_warn_when_electrical_component_cannot_be_read() {
    let msg = omitido("OND", "Generic_7.5kW.OND", "archivo ilegible");
    assert!(msg.starts_with("Revisión eléctrica omitida: OND 'Generic_7.5kW.OND'"));
}

#[test]
fn should_convert_datasheet_mu_voc_to_relative_when_checking_cold_voc() {
    let mu = mu_voc_relativo(-140.0, 49.5);
    let voc_frio = 49.5 * (1.0 + mu * (-10.0 - 25.0));
    assert!((voc_frio - 54.4).abs() < 0.01, "voc_frio={voc_frio}");
}
