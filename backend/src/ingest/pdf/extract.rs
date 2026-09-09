//! Positioned text extraction from PDF statements via `pdfium-render`.
//!
//! `libpdfium` is loaded dynamically at runtime (it is not bundled). Binding is
//! attempted once, lazily, against `$PDFIUM_LIB_PATH` (a file or a directory) and
//! then the system library. If neither is present, [`words`] returns `Err` and
//! callers fall back to the text-only parser.

use std::path::Path;
use std::sync::OnceLock;

use anyhow::{anyhow, Result};
use pdfium_render::prelude::*;

/// A single positioned text segment lifted from a page.
#[derive(Debug, Clone, PartialEq)]
pub struct Word {
    pub text: String,
    pub x0: f32,
    pub x1: f32, // left, right  (points)
    pub y0: f32,
    pub y1: f32, // bottom, top  (points; larger = higher)
    pub page: usize, // 0-based
}

/// How pdfium was bound, recorded on first success so per-call [`Pdfium`]
/// instances can re-acquire the (already loaded) library cheaply.
#[derive(Debug, Clone)]
enum BindTarget {
    /// An explicit `$PDFIUM_LIB_PATH` (file or directory).
    Path(String),
    /// The system library resolved by name.
    System,
}

static BOUND: OnceLock<std::result::Result<BindTarget, String>> = OnceLock::new();

/// Called once at startup. Binds to `$PDFIUM_LIB_PATH` (file or dir), then to
/// the system library. Idempotent; returns the bind result.
pub fn init(lib_path: Option<&str>) -> std::result::Result<(), String> {
    BOUND
        .get_or_init(|| resolve_binding(lib_path))
        .clone()
        .map(|_| ())
}

/// True once [`init`] has bound to a libpdfium library this process.
pub fn is_available() -> bool {
    matches!(BOUND.get(), Some(Ok(_)))
}

fn resolve_binding(lib_path: Option<&str>) -> std::result::Result<BindTarget, String> {
    if let Some(p) = lib_path.map(str::trim).filter(|p| !p.is_empty()) {
        if bind_path(p).is_ok() {
            return Ok(BindTarget::Path(p.to_string()));
        }
    }
    Pdfium::bind_to_system_library()
        .map(|_| BindTarget::System)
        .map_err(|e| format!("pdfium: could not bind to a libpdfium library: {e}"))
}

fn bind_path(p: &str) -> std::result::Result<Box<dyn PdfiumLibraryBindings>, PdfiumError> {
    let path = Path::new(p);
    if path.is_dir() {
        Pdfium::bind_to_library(Pdfium::pdfium_platform_library_name_at_path(path))
    } else {
        Pdfium::bind_to_library(p)
    }
}

fn bindings() -> Result<Box<dyn PdfiumLibraryBindings>> {
    match BOUND.get() {
        Some(Ok(BindTarget::Path(p))) => {
            bind_path(p).map_err(|e| anyhow!("pdfium unavailable: {e}"))
        }
        Some(Ok(BindTarget::System)) => {
            Pdfium::bind_to_system_library().map_err(|e| anyhow!("pdfium unavailable: {e}"))
        }
        _ => Err(anyhow!("pdfium is not available")),
    }
}

/// Extract every positioned text segment from every page.
/// `Err` if pdfium is not available or the PDF cannot be loaded.
pub fn words(bytes: &[u8]) -> Result<Vec<Word>> {
    let pdfium = Pdfium::new(bindings()?);
    let doc = pdfium
        .load_pdf_from_byte_slice(bytes, None)
        .map_err(|e| anyhow!("pdfium could not load PDF: {e}"))?;

    let mut out = Vec::new();
    for (pi, page) in doc.pages().iter().enumerate() {
        let text = match page.text() {
            Ok(t) => t,
            Err(_) => continue,
        };
        for segment in text.segments().iter() {
            let s = segment.text();
            if s.trim().is_empty() {
                continue;
            }
            let b = segment.bounds();
            out.push(Word {
                text: s,
                x0: b.left().value,
                x1: b.right().value,
                y0: b.bottom().value,
                y1: b.top().value,
                page: pi,
            });
        }
    }
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn pdfium_ready() -> bool {
        init(std::env::var("PDFIUM_LIB_PATH").ok().as_deref()).is_ok()
    }

    /// A one-page A4 PDF containing the text "HELLO", generated with printpdf.
    pub(crate) fn hello_pdf() -> Vec<u8> {
        use printpdf::*;
        let (doc, page1, layer1) =
            PdfDocument::new("hello", Mm(210.0), Mm(297.0), "Layer 1");
        let font = doc.add_builtin_font(BuiltinFont::Helvetica).unwrap();
        let layer = doc.get_page(page1).get_layer(layer1);
        layer.use_text("HELLO", 24.0, Mm(20.0), Mm(250.0), &font);
        doc.save_to_bytes().unwrap()
    }

    #[test]
    fn words_is_empty_error_without_pdfium_or_returns_segments_with_it() {
        let pdf = hello_pdf();
        if !pdfium_ready() {
            assert!(words(&pdf).is_err(), "no pdfium -> Err");
            return;
        }
        let ws = words(&pdf).unwrap();
        assert!(
            ws.iter().any(|w| w.text.contains("HELLO")),
            "expected a HELLO segment, got {ws:?}"
        );
        let h = ws.iter().find(|w| w.text.contains("HELLO")).unwrap();
        assert!(h.x1 > h.x0 && h.y1 > h.y0 && h.page == 0);
    }

    #[test]
    fn hello_pdf_is_a_pdf() {
        assert!(hello_pdf().starts_with(b"%PDF"));
    }
}
