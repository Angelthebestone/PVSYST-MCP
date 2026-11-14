use super::*;
use serde_json::json;

fn objeto_de(n: usize) -> Value {
    let hijos: Map<String, Value> = (0..n).map(|i| (format!("k{i}"), json!(i))).collect();
    json!({ "origen": "pvsyst", "grupo": hijos })
}

/// `proyecto p50_p90` answers the ten fields PVsyst stored in one object: they
/// are values of the result, not a table to leave out of `vista.campos`.
#[test]
fn should_list_the_ten_scalars_of_a_nested_group() {
    let (campos, recorte) = campos_de(&objeto_de(10));

    assert_eq!(campos.len(), 10);
    assert!(recorte.is_none());
    assert_eq!(campos[0].clave, "grupo.k0");
}

#[test]
fn should_leave_out_a_nested_object_that_is_a_table_of_twelve_months() {
    let (campos, _) = campos_de(&objeto_de(12));

    assert!(campos.is_empty());
}
