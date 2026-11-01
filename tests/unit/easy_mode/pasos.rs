use super::pasos::PasoSitioSalida;
use super::*;

fn paso(ok: bool, sit_file: &str) -> PasoSitioSalida {
    PasoSitioSalida {
        sit_file: sit_file.into(),
        site_name: Some("Planta_Site".into()),
        weather_source: None,
        annual_ghi_kwh_m2: None,
        reutilizado: false,
        ok,
    }
}

#[test]
fn should_stop_design_when_site_step_did_not_create_the_site() {
    for p in [paso(false, "Sites/Planta_Site.SIT"), paso(true, "")] {
        let res = exigir_sitio_creado(&p);
        assert!(matches!(res, Err(Error::Cli(m)) if m.contains("Planta_Site")));
    }
}

#[test]
fn should_continue_design_when_site_exists() {
    assert!(exigir_sitio_creado(&paso(true, "Sites/Planta_Site.SIT")).is_ok());
}
