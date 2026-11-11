use super::*;

/// A window `(hwnd, parent)` of class and text `(clase, texto)` at `rect`.
fn nodo(
    (hwnd, padre): (Hwnd, Hwnd),
    (clase, texto): (&str, &str),
    rect: (i32, i32, i32, i32),
) -> NodoWin32 {
    NodoWin32 {
        hwnd,
        padre,
        clase: clase.into(),
        texto: texto.into(),
        rect: Rect {
            left: rect.0,
            top: rect.1,
            right: rect.2,
            bottom: rect.3,
        },
        visible: true,
        habilitado: true,
        marcado: None,
    }
}

fn oculto(mut nodo: NodoWin32) -> NodoWin32 {
    nodo.visible = false;
    nodo
}

fn instantanea(nodos: Vec<NodoWin32>, pestanas_uia: &[&str]) -> Instantanea {
    Instantanea {
        nodos,
        origen: Rect::default(),
        escala: 1.0,
        pestanas_uia: pestanas_uia.iter().map(ToString::to_string).collect(),
        seleccionada: Some(0),
    }
}

/// Page 0 is the one shown; page 1 holds a ticked check box and is hidden; page 2 holds none.
fn formulario_de_tres_paginas() -> Inventario {
    let todo = (0, 0, 780, 560);
    let casilla = nodo((10, 3), ("TCheckBox", "Activo"), (20, 30, 200, 50));
    let nodos = vec![
        nodo((1, 100), ("TPageControl", ""), todo),
        nodo((2, 1), (CLASE_PAGINA, "Conexión a la red"), todo),
        oculto(nodo((3, 1), (CLASE_PAGINA, "Estimación P50 - P90"), todo)),
        oculto(nodo((4, 1), (CLASE_PAGINA, "Parámetros heredados"), todo)),
        NodoWin32 {
            marcado: Some(true),
            ..oculto(casilla)
        },
    ];
    let tabs = [
        "Conexión a la red",
        "Estimación P50 - P90",
        "Parámetros heredados",
    ];
    inventariar(&instantanea(nodos, &tabs))
}

#[test]
fn should_name_the_tab_of_hidden_controls() {
    let inv = formulario_de_tres_paginas();

    let casilla = inv.controles.iter().find(|c| c.hwnd == "0xA").unwrap();

    assert_eq!(casilla.pestana.as_deref(), Some("Estimación P50 - P90"));
    assert_eq!(casilla.indice_pestana, Some(1));
    assert_eq!(casilla.parent.as_deref(), Some("0x3"));
    assert_eq!((casilla.visible, casilla.marcado), (false, Some(true)));
    assert_eq!(casilla.etiqueta.as_deref(), Some("Activo"));
    assert_eq!(
        casilla.etiqueta_fuente,
        Some(FuenteEtiqueta::TextoDeVentana)
    );
}

#[test]
fn should_flag_a_hidden_page_without_windows_as_created_on_activation() {
    let inv = formulario_de_tres_paginas();

    let paginas: Vec<_> = inv
        .pestanas
        .iter()
        .map(|p| (p.indice, p.seleccionada, p.controles.len()))
        .collect();
    let creadas_al_activar: Vec<_> = inv.pestanas.iter().map(|p| p.creado_al_activar).collect();

    assert_eq!(
        paginas,
        [(Some(0), true, 0), (Some(1), false, 1), (Some(2), false, 0)]
    );
    assert_eq!(creadas_al_activar, [false, false, true]);
}

#[test]
fn should_list_unknown_classes_as_other_and_give_fields_an_anchor() {
    let nodos = vec![
        nodo((1, 100), ("TGridPanel", ""), (0, 0, 400, 300)),
        nodo((2, 1), ("TVirtualStringTree", ""), (10, 10, 390, 200)),
        nodo((3, 1), ("TEdit", "1.7"), (120, 220, 200, 240)),
        nodo((4, 1), ("TStaticText", "Largo"), (10, 220, 100, 240)),
        nodo((5, 1), ("TMiBotonRaro", ""), (0, 0, 5, 5)),
    ];

    let inv = inventariar(&instantanea(nodos, &[]));

    let por_clase = |clase: &str| inv.controles.iter().find(|c| c.clase == clase).unwrap();
    assert_eq!(por_clase("TGridPanel").rol, Rol::Panel);
    assert_eq!(por_clase("TVirtualStringTree").rol, Rol::Arbol);
    assert_eq!(por_clase("TMiBotonRaro").rol, Rol::Otro);
    assert!(por_clase("TVirtualStringTree").campo.is_none());
    let campo = por_clase("TEdit");
    let ancla = Ancla {
        clase: "TEdit".into(),
        x: 120,
        y: 220,
    };
    assert_eq!(campo.campo, Some(ancla));
    assert_eq!(campo.etiqueta.as_deref(), Some("Largo"));
}
