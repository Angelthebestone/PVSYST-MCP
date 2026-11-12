use crate::adapters::gui::observar;
use crate::adapters::gui::ventana_falsa::{Hijo, VentanaFalsa};
use crate::config::Paths;
use serde_json::{json, Value};

/// `que:inventario` over a fake form, without UI Automation (the fake form has
/// no tab items to read) and without ever switching tab.
fn inventario(ventana: &VentanaFalsa, extra: &Value) -> Value {
    let paths = Paths::para_test(std::env::temp_dir());
    let mut args = json!({
        "que": "inventario",
        "ventana": ventana.referencia(0),
        "pids": [std::process::id()],
        "incluir_uia": false,
    });
    if let (Some(a), Some(e)) = (args.as_object_mut(), extra.as_object()) {
        a.extend(e.clone());
    }
    observar(&paths, &args).unwrap()
}

fn por_clase<'a>(respuesta: &'a Value, clase: &str) -> Vec<&'a Value> {
    respuesta["controles"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|c| c["clase"] == clase)
        .collect()
}

const PAGINA_P50: &str = "Estimación P50 - P90";
const PAGINA: (i32, i32, i32, i32) = (0, 20, 780, 560);

/// A page of the page control (window 1) with the caption it shows on its tab.
fn pagina(texto: &'static str) -> Hijo {
    Hijo::de("TTabSheet", PAGINA).en(1).con_texto(texto)
}

/// `TSystemeAddForm` with the P50-P90 page hidden behind the shown one: the 21
/// check boxes and edits of that page, read without switching to it.
fn formulario_p50() -> VentanaFalsa {
    let mut hijos = vec![
        Hijo::de("TPageControl", (0, 0, 780, 560)),
        pagina("Conexión a la red"),
        pagina(PAGINA_P50).oculto(),
        pagina("Parámetros heredados").oculto(),
        Hijo::de("TComboBox", (20, 40, 200, 60)).en(2),
    ];
    for fila in 0..11 {
        let arriba = 30 + fila * 24;
        let casilla = Hijo::de("TCheckBox", (20, arriba, 300, arriba + 20));
        hijos.push(casilla.en(3).con_texto("Activo").oculto());
    }
    for fila in 0..10 {
        let arriba = 30 + fila * 24;
        let edit = Hijo::de("TEdit", (320, arriba, 400, arriba + 20));
        hijos.push(edit.en(3).con_texto("95").oculto());
    }
    VentanaFalsa::crear("TSystemeAddForm", "Gestión de la energía", hijos)
}

#[test]
fn should_list_the_controls_of_a_tab_that_is_not_shown_without_switching_to_it() {
    let form = formulario_p50();

    let respuesta = inventario(&form, &json!({}));

    let casillas = por_clase(&respuesta, "TCheckBox");
    let edits = por_clase(&respuesta, "TEdit");
    assert_eq!((casillas.len(), edits.len()), (11, 10));
    for c in casillas.iter().chain(&edits) {
        assert_eq!(c["pestana"], PAGINA_P50);
        assert_eq!(c["visible"], false);
        assert_eq!(c["parent"], form.referencia(3));
        assert!(c["hwnd"].is_string() && c["rect"]["right"].as_i64() > c["rect"]["left"].as_i64());
    }
    let pagina = respuesta["pestanas"]
        .as_array()
        .unwrap()
        .iter()
        .find(|p| p["nombre"] == PAGINA_P50)
        .unwrap();
    assert_eq!(pagina["controles"].as_array().unwrap().len(), 21);
    assert_eq!(pagina["seleccionada"], false);
    assert_eq!(respuesta["truncado"], false);
    // Only the shown controls when asked: the combo of the first page.
    let visibles = inventario(&form, &json!({ "todos": false }));
    assert!(por_clase(&visibles, "TCheckBox").is_empty());
    assert_eq!(por_clase(&visibles, "TComboBox").len(), 1);
}

/// UI Automation is on by default: its reading adds the `fuente` of every row
/// and never replaces the Win32 list, so the hidden controls are all still there.
#[test]
fn should_keep_the_hidden_controls_when_ui_automation_is_read_too() {
    let form = formulario_p50();
    let paths = Paths::para_test(std::env::temp_dir());
    let args =
        json!({ "que": "inventario", "ventana": form.referencia(0), "pids": [std::process::id()] });

    let respuesta = observar(&paths, &args).unwrap();

    assert_eq!(respuesta["uia_error"], Value::Null);
    assert_eq!(por_clase(&respuesta, "TEdit").len(), 10);
    let fuentes: Vec<_> = respuesta["controles"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|c| c["hwnd"].is_string())
        .map(|c| c["fuente"].as_str().unwrap())
        .collect();
    assert!(fuentes.iter().all(|f| ["win32", "win32+uia"].contains(f)));
}

#[test]
fn should_list_the_dimension_controls_of_a_module_form_that_no_schema_maps() {
    // `components.panels` maps the "Datos básicos" page only: the sizes live on another page.
    let form = VentanaFalsa::crear(
        "TPVModuleForm",
        "Definición de un módulo FV",
        vec![
            Hijo::de("TPageControl", (0, 0, 780, 560)),
            pagina("Datos básicos"),
            pagina("Tamaños y tecnología").oculto(),
            Hijo::de("TEdit", (121, 179, 200, 199))
                .en(2)
                .con_texto("410"),
            Hijo::de("TEdit", (116, 100, 200, 120))
                .en(3)
                .con_texto("1.72")
                .oculto(),
            Hijo::de("TEdit", (116, 130, 200, 150))
                .en(3)
                .con_texto("1.13")
                .oculto(),
            Hijo::de("TGridPanel", (0, 0, 400, 80)).en(3).oculto(),
        ],
    );

    let respuesta = inventario(&form, &json!({}));

    let largo_y_ancho: Vec<_> = por_clase(&respuesta, "TEdit")
        .into_iter()
        .filter(|c| c["pestana"] == "Tamaños y tecnología")
        .collect();
    assert_eq!(largo_y_ancho.len(), 2);
    assert_eq!(largo_y_ancho[0]["texto"], "1.72");
    assert!(largo_y_ancho.iter().all(|c| c["campo"]["clase"] == "TEdit"));
    let panel = por_clase(&respuesta, "TGridPanel");
    assert_eq!(panel[0]["rol"], "pane");
    assert_eq!(panel[0]["visible"], false);
}

#[test]
fn should_tell_three_fields_with_the_same_label_apart_by_class_index_and_handle() {
    let mut hijos = Vec::new();
    for fila in 0..3 {
        let arriba = 40 + fila * 30;
        let rotulo = Hijo::de("TStaticText", (10, arriba, 140, arriba + 20));
        hijos.push(rotulo.con_texto("Núm. de inversores"));
        hijos.push(Hijo::de("TEdit", (150, arriba, 220, arriba + 20)).con_texto("1"));
    }
    let form = VentanaFalsa::crear("TSystemegridForm", "Sistema", hijos);

    let respuesta = inventario(&form, &json!({}));

    let edits = por_clase(&respuesta, "TEdit");
    assert_eq!(edits.len(), 3);
    assert!(edits.iter().all(|c| c["etiqueta"] == "Núm. de inversores"));
    assert!(edits.iter().all(|c| c["etiqueta_fuente"] == "window_text"));
    let mut indices: Vec<_> = edits
        .iter()
        .map(|c| c["indice_clase"].as_u64().unwrap())
        .collect();
    indices.sort_unstable();
    assert_eq!(indices, [0, 1, 2]);
    let mut hwnds: Vec<_> = edits.iter().map(|c| c["hwnd"].as_str().unwrap()).collect();
    hwnds.dedup();
    assert_eq!(hwnds.len(), 3);
    let alturas: Vec<_> = edits
        .iter()
        .map(|c| c["campo"]["y"].as_i64().unwrap())
        .collect();
    assert!(alturas.windows(2).all(|p| p[0] != p[1]));
}
