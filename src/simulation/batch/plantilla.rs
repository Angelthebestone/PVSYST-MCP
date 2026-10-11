//! Batch-mode parameters CSV (`-bpf`): the verbatim layout PVsystCLI reads.

use crate::error::{Error, Result};
use crate::format::value_text;
use serde_json::Value;
use std::fmt::Write as _;

/// Batch CSV columns after `Ident`: accepted row keys (canonical first), in the
/// verbatim column order PVsyst expects.
const COLUMNS: [&[&str]; 20] = [
    &["csv", "create_file", "hourly_file"],
    &["tilt", "inclinacion"],
    &["azim", "azimut"],
    &["pan", "modulo"],
    &["rserie"],
    &["rshunt"],
    &["rsh0", "rsh_0"],
    &["n_mod_serie", "nb_in_series"],
    &["n_strings", "nb_in_parallel"],
    &["ond", "inversor"],
    &["n_inverter", "nb_inverter"],
    &["thermal_uc"],
    &["thermal_uw"],
    &["rwiring"],
    &["soiling"],
    &["mod_quality"],
    &["mismatch"],
    &["lid"],
    &["iam"],
    &["comment", "comentario"],
];

/// First non-null value among `keys` in a batch row, as PVsyst text.
fn pick(row: &Value, keys: &[&str]) -> String {
    keys.iter()
        .filter_map(|k| row.get(*k))
        .find(|v| !v.is_null())
        .map(value_text)
        .unwrap_or_default()
}

/// Cells of one batch row; PVsyst's CSV has no quoting, so a separator or
/// line break inside a value would shift columns or add rows.
fn celdas(fila: usize, row: &Value) -> Result<Vec<String>> {
    COLUMNS
        .iter()
        .map(|keys| {
            let celda = pick(row, keys);
            if celda.contains([';', '\r', '\n']) {
                return Err(Error::Invalid(format!(
                    "Fila {fila}, columna '{}': el valor no puede contener ';' ni saltos de línea",
                    keys[0]
                )));
            }
            Ok(celda)
        })
        .collect()
}

/// Render the batch-mode parameters CSV (pure; `stamp` is the creation date).
pub(super) fn batch_csv(
    proyecto: &str,
    variante: &str,
    filas: &[Value],
    stamp: &str,
) -> Result<String> {
    let cuerpo = filas
        .iter()
        .enumerate()
        .map(|(i, row)| Ok(format!("SIM_{};{}\n", i + 1, celdas(i + 1, row)?.join(";"))))
        .collect::<Result<String>>()?;
    let sep = ";".repeat(20);
    let mut out = String::with_capacity(2048 + filas.len() * 128);
    // `write!` into a String is infallible, so the fmt::Result is discarded.
    let _ = writeln!(out, "PVsyst simulations Batch mode{sep}");
    let _ = writeln!(out, "Simulation parameters definition{sep}");
    let _ = writeln!(out, "File created on {stamp}{sep}");
    let _ = writeln!(out, "{sep}");
    let _ = writeln!(out, "Project;;;{proyecto}.PRJ;;;;;;;;;;;;;;;;;");
    let _ = writeln!(out, "Variants based on;;{variante};;;;;;;;;;;;;;;;");
    let _ = writeln!(out, "{sep}");
    let _ = writeln!(
        out,
        "Please define the parameters to be varied for each run{sep}"
    );
    let _ = writeln!(out, "Do not modify the column titles!{sep}");
    let _ = writeln!(
        out,
        "\"Only the lines beginning by \"\"SIM_\"\" will be executed\"{sep}"
    );
    let _ = writeln!(out, "{sep}");
    // Column titles (verbatim layout expected by PVsyst).
    out.push_str("Ident;Create hourly;Plane;Plane;PV module;Rserie;Rshunt;Rsh(0);Nb module;Nb strings;Inverter;Nb inverter;Thermal;Thermal;Rwiring;Soiling;Mod. quality;Mismatch;LID;IAM ashrae;Simul\n");
    out.push_str(";file;tilt;azim;*.PAN File;;;;in series;in parall;*.OND File;or MPPT;Uc fact;Uw fact;Array;Loss;loss;Loss;Loss;bo param;Comment\n");
    out.push_str(";File name;[deg];[deg];FileName;[Ohm];[Ohm];[Ohm];;;Filename;;[W/m2K];[W/m2K];[mOhm];[%];[%];[%];[%];;\n");
    let _ = writeln!(out, "{sep}");
    out.push_str(&cuerpo);
    let _ = writeln!(out, "{sep}");
    let _ = writeln!(
        out,
        "Don't forget to close this file in EXCEL before returning to PVsyst for executing the simulation!{sep}"
    );
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn batch_csv_keeps_the_positional_layout() {
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
        assert_eq!(*sim, "SIM_1;;30;180;;;;;;;;;;;;;;;;;a");
        assert_eq!(sim.matches(';').count(), COLUMNS.len());
    }

    #[test]
    fn should_reject_values_that_would_shift_batch_columns() {
        for malo in ["a;b", "linea\nnueva", "x\r"] {
            let rows = [json!({ "tilt": 30, "comentario": malo })];
            assert!(
                batch_csv("P", "VC1", &rows, "01/01/2026").is_err(),
                "{malo:?}"
            );
        }
    }
}
