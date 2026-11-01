use super::*;

fn mapa(v: &Value) -> serde_json::Map<String, Value> {
    v.as_object().cloned().unwrap_or_default()
}

#[test]
fn should_write_one_line_per_parameter_when_fields_are_clean() {
    let args = json!({ "unidades": { "Alpha": "%" }, "descripciones": { "Alpha": "pérdida" } });
    let params = mapa(&json!({ "Alpha": 1.5 }));

    let dat = construir_dat(&params, &ColumnasDat::de_args(&args)).expect("dat");

    assert_eq!(
        dat,
        "Hidden parameters: User's modifications;\nAlpha;0.0000;1.5;%;pérdida\n"
    );
}

#[test]
fn should_reject_separator_in_any_dat_field() {
    let casos = [
        (json!({ "A;B": 1 }), json!({})),
        (json!({ "A": "1\n2" }), json!({})),
        (json!({ "A": 1 }), json!({ "iniciales": { "A": "0;1" } })),
        (json!({ "A": 1 }), json!({ "unidades": { "A": "%\r" } })),
        (
            json!({ "A": 1 }),
            json!({ "descripciones": { "A": "x;y" } }),
        ),
    ];
    for (params, args) in casos {
        let res = construir_dat(&mapa(&params), &ColumnasDat::de_args(&args));
        assert!(matches!(res, Err(Error::Invalid(_))), "{params} {args}");
    }
}

#[test]
fn should_reject_rvt_variable_that_is_not_text_or_has_separator() {
    for variables in [
        json!(["HSol", 3]),
        json!(["HSol;E_Grid"]),
        json!(["HSol\n"]),
    ] {
        let args = json!({ "variables": variables });
        assert!(
            matches!(variables_rvt(&args), Err(Error::Invalid(_))),
            "{args}"
        );
    }
}

#[test]
fn should_return_no_rvt_variables_when_absent() {
    assert!(variables_rvt(&json!({})).expect("vacío").is_empty());
}
