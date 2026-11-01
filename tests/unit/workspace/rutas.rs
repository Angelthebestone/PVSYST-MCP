use super::*;
use tempfile::tempdir;

#[test]
fn should_treat_named_components_as_plain_relative_path() {
    assert!(es_relativa_simple(Path::new("Projects/Planta.PRJ")));
    assert!(es_relativa_simple(Path::new("./Planta.PRJ")));
}

#[test]
fn should_not_treat_rooted_prefixed_or_parent_paths_as_plain_relative() {
    for p in ["\\Users\\x", "/x", "C:x", "C:\\x", "a/../b", ".."] {
        assert!(!es_relativa_simple(Path::new(p)), "{p}");
    }
}

#[test]
#[ignore = "requiere privilegios de SO para crear enlaces simbólicos"]
fn should_reject_dangling_symlink_that_points_outside_when_resolving() {
    let ws = tempdir().expect("ws");
    let fuera = tempdir().expect("fuera");
    let destino = fuera.path().join("no_existe.txt");
    #[cfg(windows)]
    std::os::windows::fs::symlink_file(&destino, ws.path().join("enlace.txt")).expect("enlace");
    #[cfg(unix)]
    std::os::unix::fs::symlink(&destino, ws.path().join("enlace.txt")).expect("enlace");

    assert!(ruta_segura(ws.path(), "enlace.txt").is_err());
}

#[test]
fn should_accept_new_file_inside_workspace_when_resolving() {
    let ws = tempdir().expect("ws");

    let ruta = ruta_segura(ws.path(), "nuevo/archivo.txt").expect("dentro");

    assert_eq!(ruta, ws.path().join("nuevo/archivo.txt"));
}

#[test]
fn should_find_file_in_second_base_and_skip_missing_bases() {
    let a = tempdir().expect("a");
    let b = tempdir().expect("b");
    fs::write(b.path().join("X.HOR"), "x").expect("hor");
    let bases = [
        a.path().join("no_existe"),
        a.path().to_path_buf(),
        b.path().to_path_buf(),
    ];

    let hallado = primer_existente_en_bases(&bases, &["X".into(), "X.HOR".into()]);

    assert_eq!(hallado, Some(b.path().join("X.HOR")));
}

#[test]
fn should_not_leave_base_when_name_is_absolute_or_has_parent() {
    let base = tempdir().expect("base");
    let fuera = tempdir().expect("fuera");
    let ajeno = fuera.path().join("ajeno.txt");
    fs::write(&ajeno, "x").expect("ajeno");
    let nombres = [ajeno.to_string_lossy().into_owned(), "../ajeno.txt".into()];

    assert_eq!(
        primer_existente_en_bases(&[base.path().to_path_buf()], &nombres),
        None
    );
}

#[test]
fn should_propagate_escape_error_when_searching_workspace_candidates() {
    let ws = tempdir().expect("ws");

    let res = primer_existente_en_workspace(ws.path(), &["../x".into()], Alcance::Workspace);

    assert!(matches!(res, Err(Error::Invalid(_))));
}

#[test]
fn should_reject_console_device_names() {
    for nombre in ["CONIN$", "conout$", "CONOUT$.txt"] {
        assert!(
            matches!(
                crate::workspace::validar_nombre(nombre),
                Err(Error::Invalid(_))
            ),
            "{nombre}"
        );
    }
    assert!(crate::workspace::validar_nombre("CONSOLA.txt").is_ok());
}
