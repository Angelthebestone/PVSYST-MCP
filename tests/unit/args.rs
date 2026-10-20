use super::*;
use serde_json::json;

#[test]
fn aliases_are_tried_in_order_and_empty_counts_as_missing() {
    let args = json!({ "proyecto": "", "project": "P" });
    assert_eq!(opt_str(&args, &["proyecto", "project"]), Some("P"));
    assert_eq!(opt_str(&args, &["nada"]), None);
}

#[test]
fn required_error_names_canonical_key_and_hint() {
    let args = json!({});
    assert_eq!(
        req_str(&args, &["comando", "command"], "e.g. help")
            .unwrap_err()
            .to_string(),
        "Missing required argument 'comando' (e.g. help)"
    );
    assert_eq!(
        req_str(&args, &["archivo"], "").unwrap_err().to_string(),
        "Missing required argument 'archivo'"
    );
}

#[test]
fn kind_comes_from_tipo_or_the_file_extension() {
    assert_eq!(
        tipo_de(&json!({"tipo": "OND"}), "x.PAN").as_deref(),
        Some("OND")
    );
    assert_eq!(
        tipo_de(&json!({"tipo": ""}), "mod.pan").as_deref(),
        Some("PAN")
    );
    assert_eq!(tipo_de(&json!({}), "sin_extension"), None);
}

#[test]
fn candidates_prefer_substrings_then_close_names() {
    let opts: Vec<String> = ["PVModule_Canadian.PAN", "Inv_50kW.OND", "Other.PAN"]
        .map(String::from)
        .to_vec();
    assert_eq!(candidatos("canadian", &opts), vec!["PVModule_Canadian.PAN"]);
    assert_eq!(candidatos("Inv_50kV.OND", &opts), vec!["Inv_50kW.OND"]);
    assert!(candidatos("zzzzzzzzzz", &opts).is_empty());
}

#[test]
fn not_found_message_lists_candidates_or_folder() {
    let opts = vec!["Planta.PRJ".to_string()];
    assert_eq!(
        no_encontrado("Proyecto", "planta", &opts, "C:/ws").to_string(),
        "Proyecto 'planta' no encontrado; candidatos: Planta.PRJ"
    );
    assert_eq!(
        no_encontrado("Proyecto", "xyz", &[], "C:/ws").to_string(),
        "Proyecto 'xyz' no encontrado; no hay ninguno en C:/ws"
    );
}

mod props {
    use super::*;
    use proptest::prelude::*;

    proptest! {
        #[test]
        fn candidates_include_exact_match_when_present(
            word in "[a-z]{1,8}",
        ) {
            let options = vec![word.clone(), "other".to_string()];
            let found = candidatos(&word, &options);
            prop_assert!(found.contains(&word));
        }

        #[test]
        fn candidates_are_few_and_come_from_the_options(
            needle in "[a-z]{0,6}",
            options in prop::collection::vec("[a-z]{0,8}", 0..12),
        ) {
            let found = candidatos(&needle, &options);
            prop_assert!(found.len() <= 5);
            prop_assert!(found.iter().all(|c| options.contains(c)));
        }
    }
}
