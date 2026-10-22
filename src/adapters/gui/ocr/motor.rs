//! Windows.Media.Ocr integration for GUI image text recognition.
//!
//! Uses WinRT built-in OCR without external native dependencies.

use super::dominio::{LineaOcr, PalabraOcr, RectOcr};
use crate::error::{Error, Result};
use std::time::Duration;
use windows::core::HSTRING;
use windows::Globalization::Language;
use windows::Graphics::Imaging::{
    BitmapAlphaMode, BitmapDecoder, BitmapPixelFormat, SoftwareBitmap,
};
use windows::Media::Ocr::OcrEngine;
use windows::Storage::Streams::{DataWriter, InMemoryRandomAccessStream};
use windows::Win32::System::Com::{CoInitializeEx, CoUninitialize, COINIT_MULTITHREADED};

const PAUSA_SONDEO_MS: u64 = 2;
const ESTADO_ASYNC_INICIADO: i32 = 0;
const ESTADO_ASYNC_COMPLETADO: i32 = 1;

/// Input image carrier for OCR: either pre-encoded PNG bytes or raw BGRA pixels.
#[derive(Debug, Clone, Copy)]
pub(crate) enum EntradaOcr<'a> {
    Png(&'a [u8]),
    Bgra {
        ancho: u32,
        alto: u32,
        pixeles: &'a [u8],
    },
}

struct ComGuard(bool);

impl ComGuard {
    fn enter() -> Self {
        // SAFETY: CoInitializeEx is paired with CoUninitialize on drop if it initialized COM.
        let hr = unsafe { CoInitializeEx(None, COINIT_MULTITHREADED) };
        Self(hr.is_ok())
    }
}

impl Drop for ComGuard {
    fn drop(&mut self) {
        if self.0 {
            // SAFETY: Matched with successful CoInitializeEx on this thread.
            unsafe { CoUninitialize() };
        }
    }
}

macro_rules! esperar_async {
    ($op_expr:expr, $nombre:expr) => {{
        let op = $op_expr.map_err(|e| Error::Gui(format!("{} launch failed: {e}", $nombre)))?;
        loop {
            let s = op
                .Status()
                .map_err(|e| Error::Gui(format!("{} Status failed: {e}", $nombre)))?;
            if s.0 == ESTADO_ASYNC_COMPLETADO {
                break Ok(op
                    .GetResults()
                    .map_err(|e| Error::Gui(format!("{} GetResults failed: {e}", $nombre)))?);
            }
            if s.0 != ESTADO_ASYNC_INICIADO {
                let err = op
                    .ErrorCode()
                    .map(|h| format!("{h:?}"))
                    .unwrap_or_else(|_| "desconocido".into());
                return Err(Error::Gui(format!(
                    "{} failed with status {}, code {err}",
                    $nombre, s.0
                )));
            }
            std::thread::sleep(Duration::from_millis(PAUSA_SONDEO_MS));
        }
    }};
}

fn listar_idiomas_disponibles() -> String {
    let Ok(disponibles) = OcrEngine::AvailableRecognizerLanguages() else {
        return String::new();
    };
    let Ok(count) = disponibles.Size() else {
        return String::new();
    };
    let mut tags = Vec::new();
    for i in 0..count {
        if let Ok(l) = disponibles.GetAt(i) {
            if let Ok(tag) = l.LanguageTag() {
                tags.push(tag.to_string());
            }
        }
    }
    tags.join(", ")
}

fn crear_engine_idioma_o_fallback() -> Result<OcrEngine> {
    let disponibles = OcrEngine::AvailableRecognizerLanguages()
        .map_err(|e| Error::Gui(format!("AvailableRecognizerLanguages failed: {e}")))?;
    let count = disponibles
        .Size()
        .map_err(|e| Error::Gui(format!("Size failed: {e}")))?;
    if count == 0 {
        return Err(Error::Gui(
            "No Windows OCR language pack is installed. Install a language pack (e.g. Spanish 'es-ES' or English 'en-US') in Windows Settings -> Time & Language -> Language.".into(),
        ));
    }
    if let Ok(engine) = OcrEngine::TryCreateFromUserProfileLanguages() {
        return Ok(engine);
    }
    crear_engine_ingles()
}

fn crear_engine_ingles() -> Result<OcrEngine> {
    let en_tag = HSTRING::from("en-US");
    let en_lang = Language::CreateLanguage(&en_tag)
        .map_err(|e| Error::Gui(format!("CreateLanguage('en-US') failed: {e}")))?;
    if OcrEngine::IsLanguageSupported(&en_lang).unwrap_or(false) {
        if let Ok(engine) = OcrEngine::TryCreateFromLanguage(&en_lang) {
            return Ok(engine);
        }
    }
    let lista = listar_idiomas_disponibles();
    Err(Error::Gui(format!(
        "No compatible OCR language pack installed for user profile or English (en-US). Available OCR languages: [{lista}]. Please install Spanish (es-ES) or English (en-US) with OCR support."
    )))
}

fn bitmap_desde_png(png: &[u8]) -> Result<SoftwareBitmap> {
    let stream = InMemoryRandomAccessStream::new()
        .map_err(|e| Error::Gui(format!("InMemoryRandomAccessStream failed: {e}")))?;
    let writer = DataWriter::CreateDataWriter(&stream)
        .map_err(|e| Error::Gui(format!("CreateDataWriter failed: {e}")))?;
    writer
        .WriteBytes(png)
        .map_err(|e| Error::Gui(format!("WriteBytes failed: {e}")))?;
    let _stored: u32 = esperar_async!(writer.StoreAsync(), "DataWriter.StoreAsync")?;
    let _flushed: bool = esperar_async!(writer.FlushAsync(), "DataWriter.FlushAsync")?;
    writer
        .DetachStream()
        .map_err(|e| Error::Gui(format!("DetachStream failed: {e}")))?;
    stream
        .Seek(0)
        .map_err(|e| Error::Gui(format!("Seek failed: {e}")))?;
    let decoder: BitmapDecoder = esperar_async!(
        BitmapDecoder::CreateAsync(&stream),
        "BitmapDecoder.CreateAsync"
    )?;
    esperar_async!(
        decoder.GetSoftwareBitmapConvertedAsync(
            BitmapPixelFormat::Bgra8,
            BitmapAlphaMode::Premultiplied,
        ),
        "BitmapDecoder.GetSoftwareBitmapConvertedAsync"
    )
}

fn bitmap_desde_bgra(ancho: u32, alto: u32, pixeles: &[u8]) -> Result<SoftwareBitmap> {
    let writer =
        DataWriter::new().map_err(|e| Error::Gui(format!("DataWriter::new failed: {e}")))?;
    writer
        .WriteBytes(pixeles)
        .map_err(|e| Error::Gui(format!("WriteBytes failed: {e}")))?;
    let buffer = writer
        .DetachBuffer()
        .map_err(|e| Error::Gui(format!("DetachBuffer failed: {e}")))?;
    SoftwareBitmap::CreateCopyFromBuffer(
        &buffer,
        BitmapPixelFormat::Bgra8,
        ancho as i32,
        alto as i32,
    )
    .map_err(|e| Error::Gui(format!("CreateCopyFromBuffer failed: {e}")))
}

fn extraer_palabra(word: &windows::Media::Ocr::OcrWord) -> Result<PalabraOcr> {
    let texto = word
        .Text()
        .map_err(|e| Error::Gui(format!("Word Text failed: {e}")))?
        .to_string();
    let r = word
        .BoundingRect()
        .map_err(|e| Error::Gui(format!("BoundingRect failed: {e}")))?;
    let left = r.X.round() as i32;
    let top = r.Y.round() as i32;
    let right = (r.X + r.Width).round() as i32;
    let bottom = (r.Y + r.Height).round() as i32;
    Ok(PalabraOcr {
        texto,
        rect: RectOcr::new(left, top, right, bottom),
    })
}

fn bounding_box_palabras(palabras: &[PalabraOcr]) -> RectOcr {
    if palabras.is_empty() {
        return RectOcr::new(0, 0, 0, 0);
    }
    let min_x = palabras.iter().map(|p| p.rect.left).min().unwrap_or(0);
    let min_y = palabras.iter().map(|p| p.rect.top).min().unwrap_or(0);
    let max_x = palabras.iter().map(|p| p.rect.right).max().unwrap_or(0);
    let max_y = palabras.iter().map(|p| p.rect.bottom).max().unwrap_or(0);
    RectOcr::new(min_x, min_y, max_x, max_y)
}

fn extraer_linea(line: &windows::Media::Ocr::OcrLine) -> Result<LineaOcr> {
    let texto = line
        .Text()
        .map_err(|e| Error::Gui(format!("Line Text failed: {e}")))?
        .to_string();
    let words = line
        .Words()
        .map_err(|e| Error::Gui(format!("Words failed: {e}")))?;
    let count = words
        .Size()
        .map_err(|e| Error::Gui(format!("Words Size failed: {e}")))?;
    let mut palabras = Vec::with_capacity(count as usize);
    for i in 0..count {
        let w = words
            .GetAt(i)
            .map_err(|e| Error::Gui(format!("Word GetAt failed: {e}")))?;
        palabras.push(extraer_palabra(&w)?);
    }
    let rect = bounding_box_palabras(&palabras);
    Ok(LineaOcr {
        texto,
        rect,
        palabras,
    })
}

/// Runs Windows built-in OCR on the supplied image carrier (PNG or BGRA) and
/// returns detected lines and words with bounding boxes.
pub(crate) fn reconocer_texto(entrada: EntradaOcr) -> Result<Vec<LineaOcr>> {
    let _com = ComGuard::enter();
    let engine = crear_engine_idioma_o_fallback()?;
    let bitmap = match entrada {
        EntradaOcr::Png(bytes) => bitmap_desde_png(bytes)?,
        EntradaOcr::Bgra {
            ancho,
            alto,
            pixeles,
        } => bitmap_desde_bgra(ancho, alto, pixeles)?,
    };
    let resultado = esperar_async!(engine.RecognizeAsync(&bitmap), "OcrEngine.RecognizeAsync")?;
    let lines = resultado
        .Lines()
        .map_err(|e| Error::Gui(format!("Lines failed: {e}")))?;
    let line_count = lines
        .Size()
        .map_err(|e| Error::Gui(format!("Lines Size failed: {e}")))?;
    let mut lineas_out = Vec::with_capacity(line_count as usize);
    for i in 0..line_count {
        let line = lines
            .GetAt(i)
            .map_err(|e| Error::Gui(format!("Line GetAt failed: {e}")))?;
        lineas_out.push(extraer_linea(&line)?);
    }
    Ok(lineas_out)
}
