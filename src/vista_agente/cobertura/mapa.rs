//! What the observed window map says about each window class.

use crate::adapters::gui::catalogo::mapa_de_ventanas;
use serde_json::Value;
use std::collections::{BTreeMap, HashMap, HashSet};
use std::sync::OnceLock;

/// Counts for one class, taken from its richest instance.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct ResumenMapa {
    pub(crate) instancias: usize,
    pub(crate) campos: usize,
    pub(crate) botones: usize,
}

/// Fields and buttons of one instance, the pair that exists together on screen.
type Conteo = (usize, usize);

fn mas_lleno(a: Conteo, b: Conteo) -> Conteo {
    if b.0 + b.1 > a.0 + a.1 {
        b
    } else {
        a
    }
}

/// The list a page records under `clave`; absent or not a list reads as empty.
fn lista<'a>(pagina: &'a Value, clave: &str) -> &'a [Value] {
    pagina
        .get(clave)
        .and_then(Value::as_array)
        .map_or(&[][..], Vec::as_slice)
}

/// The label a page or a control records under `clave`. An empty, blank or non-text value
/// is no label at all: the map writes one of those where it has none.
fn etiqueta<'a>(valor: &'a Value, clave: &str) -> Option<&'a str> {
    valor
        .get(clave)
        .and_then(Value::as_str)
        .filter(|texto| !texto.trim().is_empty())
}

/// Identity of one control inside its tab: the label the map gives it or, with no label, its
/// position in the page's list. Position tells two unlabelled fields of one page apart and
/// makes the field at the same position of two pages of the tab the same one, which can only
/// fall short.
#[derive(Debug, PartialEq, Eq, Hash)]
enum Rotulo<'a> {
    Texto(&'a str),
    Posicion(usize),
}

fn rotulo<'a>(control: &'a Value, clave: &str, posicion: usize) -> Rotulo<'a> {
    match etiqueta(control, clave) {
        Some(texto) => Rotulo::Texto(texto),
        None => Rotulo::Posicion(posicion),
    }
}

/// The distinct controls one tab shows, gathered from every page it appeared in: a field is its
/// VCL class plus its `Rotulo`, a button its `Rotulo`. A caption names a button, as a label does
/// a field.
#[derive(Default)]
struct Rotulos<'a> {
    campos: HashSet<(&'a str, Rotulo<'a>)>,
    botones: HashSet<Rotulo<'a>>,
}

/// The labels of each tab of one instance, by tab name; pages without a tab share an unnamed one.
fn rotulos_por_pestana(instancia: &Value) -> HashMap<Option<&str>, Rotulos<'_>> {
    let mut pestanas: HashMap<Option<&str>, Rotulos<'_>> = HashMap::new();
    for pagina in lista(instancia, "paginas") {
        let rotulos = pestanas.entry(etiqueta(pagina, "pestana")).or_default();
        for (posicion, campo) in lista(pagina, "campos").iter().enumerate() {
            let clase = campo
                .get("clase")
                .and_then(Value::as_str)
                .unwrap_or_default();
            rotulos
                .campos
                .insert((clase, rotulo(campo, "etiqueta", posicion)));
        }
        for (posicion, boton) in lista(pagina, "botones").iter().enumerate() {
            rotulos.botones.insert(rotulo(boton, "texto", posicion));
        }
    }
    pestanas
}

/// Fields and buttons of one instance: the distinct controls of each of its tabs, summed over
/// tabs. `subpestana` takes no part: every page of a tab, with or without one, joins the same
/// union, so what all its sub-tabs share counts once (the map records the form's own buttons,
/// `OK` and `Cancelar` among them, in every page of a tab) and a sub-tab adds only the labels
/// the others do not have. A page recorded with no sub-tab joins too: it is the parent page,
/// and the fields that live outside the sub-tab control are only there. A page the map repeats
/// for one tab adds only the labels the others lack, so the count is everything it saw on that
/// tab.
///
/// The union can also fall short: two different fields of one tab that share class and label
/// (two `TEdit` «Valor» on two pages) collapse into one. That is the safe error for a coverage
/// figure, one that claims less than the window holds; counting what every sub-tab shares once
/// is the point, not counting it once per sub-tab.
fn contar_instancia(instancia: &Value) -> Conteo {
    rotulos_por_pestana(instancia)
        .values()
        .fold((0, 0), |(campos, botones), rotulos| {
            (
                campos + rotulos.campos.len(),
                botones + rotulos.botones.len(),
            )
        })
}

pub(super) fn resumir(ventana: &Value) -> ResumenMapa {
    let instancias = lista(ventana, "instancias");
    let (campos, botones) = instancias
        .iter()
        .map(contar_instancia)
        .fold((0, 0), mas_lleno);
    ResumenMapa {
        instancias: instancias.len(),
        campos,
        botones,
    }
}

/// What opened each instance of `clase` that the map recorded under `titulo`: classes shared by
/// several windows (`TChoixObjetPVForm` has 15) are told apart by their title. Empty when the
/// map has no instance with that title.
pub(crate) fn abierta_con(clase: &str, titulo: &str) -> Vec<String> {
    mapa_de_ventanas()
        .get("ventanas")
        .and_then(|v| v.get(clase))
        .and_then(|v| v.get("instancias"))
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
        .filter(|i| i.get("titulo").and_then(Value::as_str) == Some(titulo))
        .filter_map(|i| i.get("abierta_con").and_then(Value::as_str))
        .map(str::to_string)
        .collect()
}

/// Every window class of `docs/gui_mapa.json`, by class name. The map never
/// changes while the server runs, so it is summarised once.
pub(crate) fn clases_del_mapa() -> &'static BTreeMap<String, ResumenMapa> {
    static CLASES: OnceLock<BTreeMap<String, ResumenMapa>> = OnceLock::new();
    CLASES.get_or_init(|| {
        mapa_de_ventanas()
            .get("ventanas")
            .and_then(Value::as_object)
            .into_iter()
            .flatten()
            .map(|(clase, ventana)| (clase.clone(), resumir(ventana)))
            .collect()
    })
}
