//! Text extraction from PVsyst PDF reports (`-rpf`).
//!
//! **Nota de alcance:** Este módulo es un extractor específico para el subconjunto
//! verificado de informes generados por PVsyst (`-rpf`), no un parser general de PDF.
//! Extrae tokens de diagramas de pérdidas, tablas mensuales y resúmenes anuales
//! desde streams zlib creados por el motor de reportes de PVsyst.

mod extract;
mod parse;

pub(crate) use extract::tokens;
pub(crate) use parse::{mensual, perdidas, resumen};

#[cfg(test)]
mod tests {
    use super::parse::{parse_energia, parse_pct};
    use super::*;

    fn fixture() -> Vec<String> {
        let path = concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/tests/fixtures/results/MCP_E2E.pdf"
        );
        tokens(&std::fs::read(path).unwrap())
    }

    #[test]
    fn loss_diagram_gives_temperature_and_nominal_energy() {
        let v = perdidas(&fixture()).unwrap();
        let nodos = v["nodos"].as_array().unwrap();
        let temp = nodos
            .iter()
            .find(|n| n["concepto"] == "PV loss due to temperature")
            .unwrap();
        assert_eq!(temp["pct"].as_f64(), Some(-10.1));
        let nominal = nodos
            .iter()
            .find(|n| n["concepto"] == "Array nominal energy (at STC effic.)")
            .unwrap();
        assert_eq!(nominal["valor"].as_f64(), Some(16693.0));
        assert!(v["resumen"]
            .as_str()
            .unwrap()
            .starts_with("−10.1 % temperatura"));
    }

    #[test]
    fn monthly_table_gives_january_grid_energy_and_year_pr() {
        let v = mensual(&fixture()).unwrap();
        assert_eq!(v["meses"][0]["mes"], "January");
        assert_eq!(v["meses"][0]["valores"]["E_Grid"].as_f64(), Some(1344.0));
        assert_eq!(v["anual"]["PR"].as_f64(), Some(0.837));
    }

    #[test]
    fn summary_gives_produced_energy() {
        let v = resumen(&fixture()).unwrap();
        assert_eq!(v["energia_anual_kwh"].as_f64(), Some(14299.0));
        assert_eq!(v["pr_pct"].as_f64(), Some(83.68));
    }

    #[test]
    fn missing_loss_diagram_is_an_error() {
        assert!(perdidas(&["Something else".to_string()]).is_err());
    }

    #[test]
    fn percentage_and_energy_tokens_are_recognised() {
        assert_eq!(parse_pct("+0.9%"), Some(0.9));
        assert_eq!(parse_pct("-10.1%"), Some(-10.1));
        assert_eq!(parse_pct("1.2.3%"), None);
        assert_eq!(parse_energia("1854  kWh/m²"), Some((1854.0, "kWh/m²")));
        assert_eq!(parse_energia("14563 kWh"), Some((14563.0, "kWh")));
        assert_eq!(parse_energia("kWh"), None);
    }
}
