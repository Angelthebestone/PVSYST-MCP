//! Choosing the control behind `fila`: the host window names a class, and the
//! selection must be unambiguous before anything is clicked.

use super::unico_de_clase;

#[test]
fn should_refuse_the_class_when_two_controls_share_it() {
    let dos = vec![
        (1, "TVirtualStringTree".to_string()),
        (2, "TVirtualStringTree".to_string()),
    ];
    let error = unico_de_clase(&dos, "TVirtualStringTree")
        .expect_err("ambiguous class")
        .to_string();

    assert!(error.contains("More than one"), "{error}");
    assert!(error.contains("hwnd"), "{error}");
}
