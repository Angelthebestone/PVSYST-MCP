use super::*;
use serde_json::json;

#[test]
fn should_refuse_cli_options_that_are_not_texts() {
    for opciones in [json!(["-w:C:\\ws", 3]), json!("-w:C:\\ws"), json!(7)] {
        let res = opciones_de(&json!({ "opciones": opciones }));
        assert!(matches!(res, Err(Error::Invalid(_))), "{opciones}");
    }
}

/// The object form is `--nombre:valor`; a key that already carries a dash is used
/// as written, so the documented short aliases (`-w`, `-ocf`) can be expressed.
#[test]
fn should_express_short_aliases_flags_and_omitted_values_in_the_object_form() {
    let args = json!({ "opciones": {
        "-w": "C:/ws",
        "project": "P.PRJ",
        "odc": true,
        "site": null,
        "overwrite": false,
        "log-level": 3,
    }});
    let mut opciones = opciones_de(&args).expect("opciones válidas");
    opciones.sort();
    assert_eq!(
        opciones,
        vec!["--log-level:3", "--odc", "--project:P.PRJ", "-w:C:/ws"]
    );
}

/// A list or an object as a value used to be written as JSON text into the command.
#[test]
fn should_refuse_option_values_that_are_not_scalars() {
    for valor in [json!(["cover", "summary"]), json!({ "a": 1 })] {
        let res = opciones_de(&json!({ "opciones": { "rp": valor } }));
        assert!(matches!(res, Err(Error::Invalid(_))), "{valor}");
    }
}

#[test]
fn should_refuse_blank_and_repeated_options() {
    for opciones in [
        json!([""]),
        json!(["  "]),
        json!(["-ll:1", "-ll:1"]),
        json!({ "": "x" }),
    ] {
        let res = opciones_de(&json!({ "opciones": opciones }));
        assert!(matches!(res, Err(Error::Invalid(_))), "{opciones}");
    }
}
