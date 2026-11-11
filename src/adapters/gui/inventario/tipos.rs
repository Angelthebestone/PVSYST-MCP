//! The data shapes of the GUI inventory: a window as Win32 reports it, what a
//! control is for and the rows an inventory answers with.
//!
//! Nothing here is a caption written by hand: classes, window handles and
//! rectangles are the same in any PVsyst language.

use crate::adapters::gui::ffi::{Hwnd, Rect};
use serde::Serialize;

/// VCL class of a page of a page control; its window text is the tab caption.
pub(crate) const CLASE_PAGINA: &str = "TTabSheet";

/// A window of the form as Win32 reports it, hidden ones included.
#[derive(Debug, Clone, PartialEq)]
pub(crate) struct NodoWin32 {
    pub(crate) hwnd: Hwnd,
    pub(crate) padre: Hwnd,
    pub(crate) clase: String,
    pub(crate) texto: String,
    pub(crate) rect: Rect,
    pub(crate) visible: bool,
    pub(crate) habilitado: bool,
    pub(crate) marcado: Option<bool>,
}

/// What a control is for. The names are the ones the UI Automation roles use.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
pub(crate) enum Rol {
    #[serde(rename = "button")]
    Boton,
    #[serde(rename = "checkbox")]
    Casilla,
    #[serde(rename = "radio button")]
    Opcion,
    #[serde(rename = "edit")]
    Campo,
    #[serde(rename = "combo box")]
    Combo,
    #[serde(rename = "list")]
    Lista,
    #[serde(rename = "tree")]
    Arbol,
    #[serde(rename = "tab")]
    Pestanas,
    #[serde(rename = "tab page")]
    Pagina,
    #[serde(rename = "group")]
    Grupo,
    #[serde(rename = "text")]
    Texto,
    #[serde(rename = "pane")]
    Panel,
    #[serde(rename = "other")]
    Otro,
}

const BOTONES: &[&str] = &[
    "TButton",
    "TBitBtn",
    "TPngBitBtn",
    "TSpeedButton",
    "TToolButton",
    "Button",
];
const CAMPOS: &[&str] = &[
    "TEdit",
    "TMemo",
    "TLabeledEdit",
    "TMaskEdit",
    "TSpinEdit",
    "TDateTimePicker",
    "TUpDown",
    "TTrackBar",
];
const LISTAS: &[&str] = &["TListBox", "TCheckListBox", "TListView"];
const ARBOLES: &[&str] = &["TVirtualStringTree", "TTreeView"];
const GRUPOS: &[&str] = &["TGroupBox", "TRadioGroup", "TCheckGroup"];
const PANELES: &[&str] = &[
    "TPanel",
    "TGridPanel",
    "TFlowPanel",
    "TScrollBox",
    "TActionToolBar",
];

impl Rol {
    pub(crate) fn de_clase(clase: &str) -> Self {
        match clase {
            "TCheckBox" => Self::Casilla,
            "TRadioButton" => Self::Opcion,
            "TComboBox" | "TComboBoxEx" => Self::Combo,
            "TPageControl" | "TTabControl" => Self::Pestanas,
            "TStaticText" => Self::Texto,
            CLASE_PAGINA => Self::Pagina,
            c if BOTONES.contains(&c) => Self::Boton,
            c if CAMPOS.contains(&c) => Self::Campo,
            c if LISTAS.contains(&c) => Self::Lista,
            c if ARBOLES.contains(&c) => Self::Arbol,
            c if GRUPOS.contains(&c) => Self::Grupo,
            c if PANELES.contains(&c) => Self::Panel,
            _ => Self::Otro,
        }
    }

    /// Whether the control's own window text is its label.
    pub(super) fn rotula_con_su_texto(self) -> bool {
        matches!(
            self,
            Self::Boton | Self::Casilla | Self::Opcion | Self::Pagina | Self::Grupo | Self::Texto
        )
    }

    /// Whether a caption drawn next to the control names it.
    pub(super) fn espera_rotulo_vecino(self) -> bool {
        matches!(self, Self::Campo | Self::Combo | Self::Lista | Self::Arbol)
    }

    /// Whether `gui_actuar` can fill it by its anchor (`campo`).
    pub(super) fn es_rellenable(self) -> bool {
        matches!(
            self,
            Self::Campo | Self::Combo | Self::Casilla | Self::Opcion | Self::Lista
        )
    }
}

/// Where the label of a control was read.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
pub(crate) enum FuenteEtiqueta {
    /// The control's own window text, or the text of a window next to it.
    #[serde(rename = "window_text")]
    TextoDeVentana,
    /// Text recognised on a capture of the form.
    #[serde(rename = "ocr")]
    Ocr,
}

/// A control located by its class and its position in the form's 96-DPI layout:
/// the way `gui_actuar accion:escribir` and `campo` address a field.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub(crate) struct Ancla {
    pub(crate) clase: String,
    pub(crate) x: i32,
    pub(crate) y: i32,
}

/// One row of the inventory.
#[derive(Clone, Debug, PartialEq, Serialize)]
pub(crate) struct Control {
    pub(crate) clase: String,
    pub(crate) hwnd: String,
    pub(crate) rect: Rect,
    pub(crate) texto: String,
    pub(crate) visible: bool,
    pub(crate) enabled: bool,
    pub(crate) parent: Option<String>,
    pub(crate) pestana: Option<String>,
    pub(crate) indice_pestana: Option<usize>,
    pub(crate) rol: Rol,
    /// Position among the controls of the same class, reading the form top to bottom.
    pub(crate) indice_clase: usize,
    pub(crate) etiqueta: Option<String>,
    pub(crate) etiqueta_fuente: Option<FuenteEtiqueta>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) marcado: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) campo: Option<Ancla>,
}

/// A tab page of the form with the controls it holds.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub(crate) struct Pestana {
    pub(crate) indice: Option<usize>,
    pub(crate) nombre: String,
    pub(crate) hwnd: String,
    pub(crate) seleccionada: bool,
    pub(crate) controles: Vec<String>,
    /// A page that is not shown and holds no window: VCL may build its controls
    /// only when it is first shown, so it can look empty until then.
    pub(crate) creado_al_activar: bool,
}

/// Everything read from a form: its windows and where the form sits.
#[derive(Debug, Clone)]
pub(crate) struct Instantanea {
    pub(crate) nodos: Vec<NodoWin32>,
    /// Window rectangle of the form, the origin of the 96-DPI positions.
    pub(crate) origen: Rect,
    pub(crate) escala: f64,
    /// Tab captions in the order of the tab strip, as UI Automation lists them.
    pub(crate) pestanas_uia: Vec<String>,
    /// Page the page control reports as current.
    pub(crate) seleccionada: Option<usize>,
}

pub(crate) struct Inventario {
    pub(crate) controles: Vec<Control>,
    pub(crate) pestanas: Vec<Pestana>,
}
