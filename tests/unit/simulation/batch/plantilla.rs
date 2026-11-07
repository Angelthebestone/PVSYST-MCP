use super::*;
use serde_json::json;

#[test]
fn should_keep_the_positional_layout_when_writing_the_batch_csv() {
    let rows = [json!({ "tilt": 30, "azimut": "180", "comentario": "a", "lid": null })];
    let csv = batch_csv("P", "VC1", &rows, "01/01/2026").expect("csv");
    let lines: Vec<&str> = csv.lines().collect();
    assert_eq!(
        lines[2],
        format!("File created on 01/01/2026{}", ";".repeat(20))
    );
    assert_eq!(lines[4].matches(';').count(), 20, "Project line");
    assert_eq!(lines[5].matches(';').count(), 18, "Variants line");
    let sim = lines.iter().find(|l| l.starts_with("SIM_1;")).expect("row");
    assert_eq!(*sim, "SIM_1;P_SIM_1.csv;30;180;;;;;;;;;;;;;;;;;a");
    assert_eq!(sim.matches(';').count(), COLUMNS.len());
}

#[test]
fn should_reject_values_that_would_shift_batch_columns() {
    for malo in ["a;b", "linea\nnueva", "x\r"] {
        let rows = [json!({ "tilt": 30, "comentario": malo })];
        assert!(
            matches!(
                batch_csv("P", "VC1", &rows, "01/01/2026"),
                Err(Error::Invalid(_))
            ),
            "{malo:?}"
        );
    }
}
