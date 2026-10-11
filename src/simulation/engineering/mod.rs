//! Engineering results derived from PVsyst result files: loss and monthly
//! tables, variant comparison, P50/P90 uncertainty, carbon balance, economics
//! and the Markdown deliverable. Figures come from the result CSV, its PDF
//! report and the project file; no simulation is run.

mod carbono;
mod caso;
mod comparacion;
mod economia;
mod entregable;
mod incertidumbre;
mod supuestos;

pub(crate) use carbono::carbono;
pub(crate) use comparacion::comparar;
pub(crate) use economia::economia;
pub(crate) use entregable::{entregable, mensual, perdidas};
pub(crate) use incertidumbre::p50_p90;

#[cfg(test)]
mod tests {
    use super::carbono::{ahorro_co2_kg, factor_pais};
    use super::caso::pais_simulado;
    use super::comparacion::excluida;
    use super::economia::{economia_de, Economia};
    use super::incertidumbre::{coef_variacion, incertidumbre};

    fn economia_base() -> Economia {
        Economia {
            capex_kwp: 1000.0,
            tarifa: 0.2,
            opex_kwp: 0.0,
            tasa: 0.0,
            anios: 10,
            deg: 0.0,
            moneda: "USD".into(),
        }
    }

    #[test]
    fn coef_variacion_of_two_values() {
        assert_eq!(coef_variacion(&[100.0, 100.0]), Some(0.0));
        let cv = coef_variacion(&[90.0, 110.0]).unwrap();
        assert!((cv - 0.1414).abs() < 1e-3);
        assert_eq!(coef_variacion(&[100.0]), None);
    }

    #[test]
    fn p90_and_p99_use_combined_sigma() {
        let v = incertidumbre(1000.0, 5.0, 3.0, "defecto");
        assert!((v["p90_kwh"].as_f64().unwrap() - 925.3).abs() < 0.5);
        assert!((v["p99_kwh"].as_f64().unwrap() - 864.4).abs() < 0.5);
    }

    #[test]
    fn simulated_country_comes_from_results_csv() {
        let csv = include_str!("../../../tests/fixtures/results/MCP_E2E.csv");
        assert_eq!(pais_simulado(csv).as_deref(), Some("Colombia"));
        assert_eq!(pais_simulado("Project;X;\n"), None);
    }

    #[test]
    fn carbon_factor_falls_back_to_world() {
        let tabla = "CO2_Emission_IEA_Countries;Start;\n  Colombia; 150;\n  World; 475;\n";
        assert_eq!(factor_pais(tabla, Some("colombia")).unwrap().0, 150.0);
        let (f, etiqueta) = factor_pais(tabla, Some("Atlantis")).unwrap();
        assert_eq!(f, 475.0);
        assert_eq!(etiqueta, "World (no encontrado Atlantis)");
    }

    #[test]
    fn avoided_co2_without_degradation_is_linear_in_lifetime() {
        assert_eq!(ahorro_co2_kg(1000.0, 0.0, 2, 500.0), 1000.0);
    }

    #[test]
    fn economics_without_discount_or_opex() {
        let v = economia_de(1000.0, 1.0, &economia_base());
        assert_eq!(v["lcoe_por_kwh"].as_f64(), Some(0.1));
        assert_eq!(v["payback_anios"].as_u64(), Some(5));
        assert_eq!(v["van"].as_f64(), Some(1000.0));
    }

    #[test]
    fn payback_is_null_when_never_reached() {
        let mut p = economia_base();
        p.tarifa = 0.01;
        let v = economia_de(1000.0, 1.0, &p);
        assert!(v["payback_anios"].is_null());
    }

    #[test]
    fn excluded_parameter_prefixes_are_skipped() {
        assert!(excluida("VarSimul_TempLss"));
        assert!(!excluida("FieldTilt"));
    }
}
