//! Pure domain logic for OCR-based GUI inspection: geometric association of
//! OCR lines to interactive form controls and caption normalization.
//!
//! No I/O, no Win32/WinRT dependencies, zero side effects.

const MAX_DISTANCIA_FILA_PX: i32 = 400;
const MAX_DISTANCIA_COLUMNA_PX: i32 = 80;
const TOLERANCIA_SOLAPAMIENTO_PX: i32 = 4;
const UMBRAL_AMBIGUEDAD_PX: i32 = 4;
const PENALIZACION_DESALINEACION_PX: i32 = 1;
const TOLERANCIA_COLUMNA_IZQ_PX: i32 = 24;

/// A rectangle in image/screen physical pixels.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub(crate) struct RectOcr {
    pub(crate) left: i32,
    pub(crate) top: i32,
    pub(crate) right: i32,
    pub(crate) bottom: i32,
}

impl RectOcr {
    /// Creates a new rectangle.
    pub(crate) const fn new(left: i32, top: i32, right: i32, bottom: i32) -> Self {
        Self {
            left,
            top,
            right,
            bottom,
        }
    }

    /// Vertical center.
    pub(crate) const fn centro_y(&self) -> i32 {
        (self.top + self.bottom) / 2
    }

    /// The smallest rectangle that holds both.
    pub(crate) fn union(&self, otro: &Self) -> Self {
        Self::new(
            self.left.min(otro.left),
            self.top.min(otro.top),
            self.right.max(otro.right),
            self.bottom.max(otro.bottom),
        )
    }

    /// The rectangle with every coordinate multiplied by `k` (rounded). Used
    /// to bring a capture made at 96-DPI layout to physical pixels.
    pub(crate) fn escalado(&self, k: f64) -> Self {
        let f = |v: i32| (f64::from(v) * k).round() as i32;
        Self::new(f(self.left), f(self.top), f(self.right), f(self.bottom))
    }
}

/// A recognized word with its bounding box.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub(crate) struct PalabraOcr {
    pub(crate) texto: String,
    pub(crate) rect: RectOcr,
}

/// A recognized line of text composed of words.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub(crate) struct LineaOcr {
    pub(crate) texto: String,
    pub(crate) rect: RectOcr,
    pub(crate) palabras: Vec<PalabraOcr>,
}

impl LineaOcr {
    /// The line and its words with coordinates multiplied by `k`.
    pub(crate) fn escalada(self, k: f64) -> Self {
        Self {
            texto: self.texto,
            rect: self.rect.escalado(k),
            palabras: self
                .palabras
                .into_iter()
                .map(|p| PalabraOcr {
                    texto: p.texto,
                    rect: p.rect.escalado(k),
                })
                .collect(),
        }
    }
}

/// Result of associating an OCR label to a control.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub(crate) struct ResultadoEtiqueta {
    pub(crate) etiqueta: Option<String>,
    pub(crate) ambiguo: bool,
    pub(crate) candidatos: Vec<String>,
}

impl ResultadoEtiqueta {
    fn sin_etiqueta() -> Self {
        Self {
            etiqueta: None,
            ambiguo: false,
            candidatos: Vec::new(),
        }
    }

    fn asignada(texto: &str) -> Self {
        Self {
            etiqueta: Some(normalizar_etiqueta(texto)),
            ambiguo: false,
            candidatos: vec![texto.to_string()],
        }
    }

    fn ambigua(candidatos: Vec<String>) -> Self {
        Self {
            etiqueta: None,
            ambiguo: true,
            candidatos,
        }
    }
}

/// Internal candidate label during geometric evaluation.
#[derive(Debug, Clone, PartialEq, Eq)]
struct CandidatoEtiqueta {
    texto: String,
    distancia: i32,
}

/// Normalizes caption text by removing VCL accelerator `&`, trailing colons,
/// and collapsing whitespace runs. Pure and functional.
pub(crate) fn normalizar_etiqueta(texto: &str) -> String {
    let sin_acelerador = desescapar_ampersand(texto);
    let trimmed = sin_acelerador.trim().trim_end_matches(':').trim();
    colapsar_espacios(trimmed)
}

fn desescapar_ampersand(texto: &str) -> String {
    texto
        .split("&&")
        .map(|segmento| segmento.replace('&', ""))
        .collect::<Vec<_>>()
        .join("&")
}

fn colapsar_espacios(texto: &str) -> String {
    texto.split_whitespace().collect::<Vec<_>>().join(" ")
}

/// Checks if an OCR line is a valid label to the left of the control on the same row.
fn candidato_izquierda(linea: &RectOcr, control: &RectOcr) -> Option<i32> {
    if linea.right > control.left + TOLERANCIA_SOLAPAMIENTO_PX {
        return None;
    }
    let dx = (control.left - linea.right).max(0);
    if dx > MAX_DISTANCIA_FILA_PX {
        return None;
    }
    let solapamiento_y = linea.bottom.min(control.bottom) - linea.top.max(control.top);
    let centro_linea = linea.centro_y();
    let dentro_fila = solapamiento_y > 0
        || (centro_linea >= control.top - TOLERANCIA_SOLAPAMIENTO_PX
            && centro_linea <= control.bottom + TOLERANCIA_SOLAPAMIENTO_PX);
    if !dentro_fila {
        return None;
    }
    let desalineacion = (linea.centro_y() - control.centro_y()).abs();
    Some(dx + desalineacion * PENALIZACION_DESALINEACION_PX)
}

/// Checks if an OCR line is a valid label directly above the control.
fn candidato_arriba(linea: &RectOcr, control: &RectOcr) -> Option<i32> {
    if linea.bottom > control.top + TOLERANCIA_SOLAPAMIENTO_PX {
        return None;
    }
    let dy = (control.top - linea.bottom).max(0);
    if dy > MAX_DISTANCIA_COLUMNA_PX {
        return None;
    }
    let solapamiento_x = linea.right.min(control.right) - linea.left.max(control.left);
    let alineacion_izq = (linea.left - control.left).abs();
    if solapamiento_x <= 0 && alineacion_izq > TOLERANCIA_COLUMNA_IZQ_PX {
        return None;
    }
    let desalineacion = (linea.left - control.left).abs();
    Some(dy + desalineacion * PENALIZACION_DESALINEACION_PX)
}

/// Distance of every OCR line that qualifies under one placement rule
/// (`candidato_izquierda` or `candidato_arriba`).
fn evaluar_candidatos(
    control: &RectOcr,
    lineas: &[LineaOcr],
    regla: fn(&RectOcr, &RectOcr) -> Option<i32>,
) -> Vec<CandidatoEtiqueta> {
    lineas
        .iter()
        .filter_map(|linea| {
            regla(&linea.rect, control).map(|distancia| CandidatoEtiqueta {
                texto: linea.texto.clone(),
                distancia,
            })
        })
        .collect()
}

fn mejores_dos_candidatos(
    candidatos: &[CandidatoEtiqueta],
) -> (Option<&CandidatoEtiqueta>, Option<&CandidatoEtiqueta>) {
    let mejor = candidatos.iter().min_by_key(|c| c.distancia);
    let segundo = mejor.and_then(|m| {
        candidatos
            .iter()
            .filter(|c| !std::ptr::eq(*c, m))
            .min_by_key(|c| c.distancia)
    });
    (mejor, segundo)
}

/// Associates an OCR line to a control by geometry. Nearest candidate wins;
/// ties/ambiguity are reported rather than guessed.
pub(crate) fn asociar_etiqueta(control: &RectOcr, lineas: &[LineaOcr]) -> ResultadoEtiqueta {
    // VCL forms put the caption to the left of a field; text above a field is
    // usually a heading or formula for the group, so it only counts when no
    // caption sits to the left.
    let izquierda = evaluar_candidatos(control, lineas, candidato_izquierda);
    let candidatos = if izquierda.is_empty() {
        evaluar_candidatos(control, lineas, candidato_arriba)
    } else {
        izquierda
    };
    match mejores_dos_candidatos(&candidatos) {
        (None, _) => ResultadoEtiqueta::sin_etiqueta(),
        (Some(mejor), None) => ResultadoEtiqueta::asignada(&mejor.texto),
        (Some(mejor), Some(segundo)) => {
            let diferencia = segundo.distancia.saturating_sub(mejor.distancia);
            if diferencia <= UMBRAL_AMBIGUEDAD_PX && mejor.texto != segundo.texto {
                ResultadoEtiqueta::ambigua(vec![mejor.texto.clone(), segundo.texto.clone()])
            } else {
                ResultadoEtiqueta::asignada(&mejor.texto)
            }
        }
    }
}

#[cfg(test)]
#[path = "../../../../tests/unit/adapters/gui/ocr/dominio.rs"]
mod tests;
