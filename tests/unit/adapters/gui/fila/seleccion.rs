//! Choosing the control behind `fila`: the host window names a class, and the
//! selection must be unambiguous before anything is clicked.

use super::{unico_de_clase, unico_medido};

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

/// The project chooser has one measured tree among its children: `fila` needs no `clase`.
#[test]
fn should_pick_the_only_measured_control_of_a_form_without_a_class() {
    let hijos = vec![
        (1, "TEdit".to_string()),
        (2, "TVirtualStringTree".to_string()),
        (3, "TPngBitBtn".to_string()),
    ];

    let medido = unico_medido(&hijos, "TProjectLoadForm", "Proyectos").unwrap();

    assert_eq!(medido, Some(2));
}

#[test]
fn should_pick_nothing_when_no_child_is_measured_and_refuse_when_two_are() {
    let sin = vec![(1, "TEdit".to_string())];
    assert_eq!(unico_medido(&sin, "TProjectLoadForm", "").unwrap(), None);

    let dos = vec![
        (1, "TVirtualStringTree".to_string()),
        (2, "TVirtualStringTree".to_string()),
    ];
    let error = unico_medido(&dos, "TProjectLoadForm", "").expect_err("two measured");
    assert!(error.to_string().contains("hwnd"), "{error}");
}
