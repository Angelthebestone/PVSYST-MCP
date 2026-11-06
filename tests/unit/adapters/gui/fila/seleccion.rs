//! Choosing the control behind `fila`: the host window names a class, and the
//! selection must be unambiguous before anything is clicked.

use super::unico_de_clase;

fn hijos() -> Vec<(isize, String)> {
    vec![
        (1, "TEdit".to_string()),
        (2, "TVirtualStringTree".to_string()),
    ]
}

#[test]
fn should_pick_the_tree_when_the_request_names_its_class() {
    assert_eq!(
        unico_de_clase(&hijos(), "TVirtualStringTree").expect("the tree is there"),
        2
    );
}

#[test]
fn should_list_the_visible_classes_when_the_requested_one_is_not_there() {
    let error = unico_de_clase(&hijos(), "TPageControl")
        .expect_err("missing class")
        .to_string();

    assert!(error.contains("TPageControl"), "{error}");
    assert!(error.contains("TEdit, TVirtualStringTree"), "{error}");
}

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
