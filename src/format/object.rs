//! Generic parser for the text format used by PVsyst >= 6.40.
//!
//! Format is a tree of typed objects opened by `PVObject_<Role>=<Type>`,
//! fields as `<Key>=<Value>`, and closed by `End of PVObject <Type>`
//! or implicit `End of T...` markers.

use crate::error::{Error, Result};
use serde_json::{json, Map, Value};

use super::cierre::{coincide_tipo_cierre, extraer_tipo_cierre};

#[derive(Debug, Clone, Default)]
pub(crate) struct PvNode {
    pub obj_type: String,
    pub role: Option<String>,
    pub fields: Vec<(String, String)>,
    pub children: Vec<PvNode>,
}

impl PvNode {
    pub(crate) fn field(&self, key: &str) -> Option<&str> {
        self.fields
            .iter()
            .find(|(k, _)| k.eq_ignore_ascii_case(key))
            .map(|(_, v)| v.as_str())
    }

    pub(crate) fn find_deep(&self, key: &str) -> Option<&str> {
        self.field(key)
            .or_else(|| self.children.iter().find_map(|c| c.find_deep(key)))
    }

    pub(crate) fn to_json(&self) -> Value {
        let mut map = Map::new();
        map.insert("_type".into(), json!(self.obj_type));
        if let Some(r) = &self.role {
            map.insert("_role".into(), json!(r));
        }
        for (k, v) in &self.fields {
            map.insert(k.clone(), json!(v));
        }
        if !self.children.is_empty() {
            let arr = self.children.iter().map(PvNode::to_json).collect();
            map.insert("_children".into(), Value::Array(arr));
        }
        Value::Object(map)
    }
}

fn tstart(s: &str) -> &str {
    s.trim_start_matches(|c: char| c.is_whitespace() || c == '\u{FEFF}')
}

pub(crate) fn list_header(trimmed: &str) -> Option<(&str, &str)> {
    let (name, rest) = trimmed.split_once(", list of=")?;
    Some((name.trim(), rest.split_whitespace().last().unwrap_or("")))
}

fn line_ends_list(line: &str, name: &str) -> bool {
    line.strip_prefix("End of ").is_some_and(|r| {
        let clean = r
            .strip_prefix("the List ")
            .or_else(|| r.strip_prefix("List "))
            .unwrap_or(r);
        clean.eq_ignore_ascii_case(name)
    })
}

pub(crate) struct Lists(Vec<(usize, String, String)>);

impl Lists {
    pub(crate) fn new() -> Self {
        Lists(Vec::new())
    }

    pub(crate) fn feed(&mut self, trimmed: &str, depth: usize) -> Option<usize> {
        let line = trimmed.trim_end();
        if let Some((name, elem)) = list_header(line) {
            self.0
                .push((depth, format!("End of {elem}"), name.to_string()));
            return None;
        }
        let (base, item_end, name) = self.0.last()?;
        let base = *base;
        if line_ends_list(line, name) {
            self.0.pop();
            Some(base)
        } else if line == item_end {
            Some(base)
        } else {
            None
        }
    }
}

fn open_node(rest: &str) -> PvNode {
    let (role, ty) = rest.split_once('=').map_or((None, ""), |(r, t)| {
        let r = r.trim();
        ((!r.is_empty()).then(|| r.to_string()), t.trim())
    });
    PvNode {
        obj_type: ty.to_string(),
        role,
        ..Default::default()
    }
}

fn close_to_depth(stack: &mut Vec<PvNode>, roots: &mut Vec<PvNode>, depth: usize) {
    while stack.len() > depth {
        let Some(node) = stack.pop() else { break };
        match stack.last_mut() {
            Some(parent) => parent.children.push(node),
            None => roots.push(node),
        }
    }
}

fn is_structural_marker(trimmed: &str) -> bool {
    trimmed == "EndTags"
        || trimmed == "/"
        || trimmed.starts_with("End Remarks")
        || trimmed.ends_with(" End;")
        || trimmed.ends_with("=End")
        || trimmed.ends_with("=Start")
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub(crate) struct Estructura {
    pub(crate) abiertos_sin_cerrar: Vec<String>,
    pub(crate) cierres_sin_apertura: usize,
    pub(crate) cierres_discordantes: Vec<(String, String)>,
}

impl Estructura {
    pub(crate) fn esta_balanceada(&self) -> bool {
        self.abiertos_sin_cerrar.is_empty()
            && self.cierres_sin_apertura == 0
            && self.cierres_discordantes.is_empty()
    }
}

struct ParseState {
    roots: Vec<PvNode>,
    stack: Vec<PvNode>,
    lists: Lists,
    cierres_sin_apertura: usize,
    cierres_discordantes: Vec<(String, String)>,
}

impl ParseState {
    fn new() -> Self {
        Self {
            roots: Vec::new(),
            stack: Vec::new(),
            lists: Lists::new(),
            cierres_sin_apertura: 0,
            cierres_discordantes: Vec::new(),
        }
    }

    fn handle_closing(&mut self, trimmed: &str) -> bool {
        if let Some(depth) = self.lists.feed(trimmed, self.stack.len()) {
            close_to_depth(&mut self.stack, &mut self.roots, depth);
            true
        } else if let Some(tipo_cierre) = extraer_tipo_cierre(trimmed) {
            if let Some(top) = self.stack.last() {
                if let Some(enc) = tipo_cierre {
                    if !coincide_tipo_cierre(&top.obj_type, enc) {
                        self.cierres_discordantes
                            .push((top.obj_type.clone(), enc.to_string()));
                    }
                }
                let depth = self.stack.len() - 1;
                close_to_depth(&mut self.stack, &mut self.roots, depth);
            } else {
                self.cierres_sin_apertura += 1;
            }
            true
        } else {
            false
        }
    }
}

pub(crate) fn parse_con_diagnostico(text: &str) -> (Vec<PvNode>, Estructura) {
    let mut state = ParseState::new();

    for raw in text.lines() {
        let line = raw.trim_end_matches(['\r', '\n']);
        let trimmed = tstart(line);
        if trimmed.is_empty() {
            continue;
        }
        if let Some(rest) = trimmed.strip_prefix("PVObject_") {
            state.stack.push(open_node(rest));
            continue;
        }
        if state.handle_closing(trimmed)
            || list_header(trimmed).is_some()
            || is_structural_marker(trimmed)
        {
            continue;
        }
        if let Some((k, v)) = trimmed.split_once('=') {
            if let Some(top) = state.stack.last_mut() {
                top.fields
                    .push((k.trim().to_string(), v.trim().to_string()));
            }
        }
    }

    let abiertos_sin_cerrar = state.stack.iter().map(|n| n.obj_type.clone()).collect();
    while let Some(node) = state.stack.pop() {
        state.roots.push(node);
    }
    let estructura = Estructura {
        abiertos_sin_cerrar,
        cierres_sin_apertura: state.cierres_sin_apertura,
        cierres_discordantes: state.cierres_discordantes,
    };
    (state.roots, estructura)
}

pub(crate) fn parse(text: &str) -> Vec<PvNode> {
    parse_con_diagnostico(text).0
}

pub(crate) fn parse_first(text: &str) -> Option<PvNode> {
    parse(text).into_iter().next()
}

pub(crate) fn diagnosticar_estructura(text: &str) -> Estructura {
    parse_con_diagnostico(text).1
}

pub(crate) fn validar_estructura(bytes: &[u8]) -> Result<()> {
    let diag = diagnosticar_estructura(&String::from_utf8_lossy(bytes));
    if diag.esta_balanceada() {
        return Ok(());
    }
    let mut detalles = Vec::new();
    if !diag.abiertos_sin_cerrar.is_empty() {
        let n = diag.abiertos_sin_cerrar.len();
        let lista = diag.abiertos_sin_cerrar.join(", ");
        detalles.push(format!("{n} objeto(s) sin cerrar: [{lista}]"));
    }
    if diag.cierres_sin_apertura > 0 {
        detalles.push(format!(
            "{} cierre(s) sin apertura",
            diag.cierres_sin_apertura
        ));
    }
    if !diag.cierres_discordantes.is_empty() {
        detalles.push(format!(
            "{} cierre(s) discordante(s): {:?}",
            diag.cierres_discordantes.len(),
            diag.cierres_discordantes
        ));
    }
    Err(Error::Format(format!(
        "Estructura PVObject desbalanceada ({}): mutación rechazada",
        detalles.join("; ")
    )))
}

pub(crate) fn is_pvobject(text: &str) -> bool {
    text.lines()
        .take(5)
        .any(|l| tstart(l).starts_with("PVObject_"))
}

#[cfg(test)]
#[path = "../../tests/unit/format/object_tests.rs"]
mod tests;
