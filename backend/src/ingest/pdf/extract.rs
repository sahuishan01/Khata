//! Positioned text extraction from PDF statements via `pdfium-render`.
//!
//! `libpdfium` is loaded dynamically at runtime (it is not bundled). Binding is
//! attempted once, at [`init`] time, against `$PDFIUM_LIB_PATH` (a file or a
//! directory) and then the system library. Exactly one [`Pdfium`] instance is
//! created for the whole process and never dropped — dropping it would call
//! `FPDF_DestroyLibrary()` and tear the library down under any concurrent
//! extraction. All extraction is serialized through a `Mutex`.

use std::path::Path;
use std::sync::{Mutex, OnceLock};

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

/// The single process-wide pdfium instance. Never dropped.
static PDFIUM: OnceLock<Mutex<Pdfium>> = OnceLock::new();
/// Guards one-time initialisation so concurrent `init()` callers cannot each
/// build (and then drop) a throwaway `Pdfium` — a drop calls
/// `FPDF_DestroyLibrary()` and would tear the library down under another thread.
static INIT: OnceLock<std::result::Result<(), String>> = OnceLock::new();

/// Hard caps for the coordinate path — the email worker ingests attachments
/// from an external mailbox, so a pathological file must not exhaust memory.
const MAX_PAGES: usize = 500;
const MAX_WORDS: usize = 300_000;

/// Called once at startup. Binds to `$PDFIUM_LIB_PATH` (file or dir), then to
/// the system library, and constructs the process-wide [`Pdfium`]. Idempotent:
/// once the instance exists this is a no-op returning `Ok(())`.
pub fn init(lib_path: Option<&str>) -> std::result::Result<(), String> {
    INIT.get_or_init(|| {
        let bindings = resolve_binding(lib_path)?;
        let _ = PDFIUM.set(Mutex::new(Pdfium::new(bindings)));
        Ok(())
    })
    .clone()
}

/// True once [`init`] has constructed the process-wide pdfium instance.
pub fn is_available() -> bool {
    PDFIUM.get().is_some()
}

fn resolve_binding(
    lib_path: Option<&str>,
) -> std::result::Result<Box<dyn PdfiumLibraryBindings>, String> {
    if let Some(p) = lib_path.map(str::trim).filter(|p| !p.is_empty()) {
        if let Ok(b) = bind_path(p) {
            return Ok(b);
        }
    }
    Pdfium::bind_to_system_library()
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

/// Extract every positioned text segment from every page.
/// `Err` if pdfium is not available or the PDF cannot be loaded.
pub fn words(bytes: &[u8]) -> Result<Vec<Word>> {
    let guard = PDFIUM
        .get()
        .ok_or_else(|| anyhow!("pdfium not initialised"))?
        .lock()
        .unwrap();

    let doc = guard
        .load_pdf_from_byte_slice(bytes, None)
        .map_err(|e| anyhow!("pdfium could not load PDF: {e}"))?;

    let pages = doc.pages();
    if pages.len() as usize > MAX_PAGES {
        return Err(anyhow!("PDF has too many pages ({})", pages.len()));
    }

    let mut out = Vec::new();
    for (pi, page) in pages.iter().enumerate() {
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
            if out.len() > MAX_WORDS {
                return Err(anyhow!("PDF produced too many text segments"));
            }
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

    #[test]
    fn concurrent_words_calls_are_serialized_and_succeed() {
        if !pdfium_ready() {
            return;
        }
        let pdf = std::sync::Arc::new(hello_pdf());
        let handles: Vec<_> = (0..2)
            .map(|_| {
                let p = pdf.clone();
                std::thread::spawn(move || {
                    let ws = words(&p).expect("words() under concurrency");
                    assert!(ws.iter().any(|w| w.text.contains("HELLO")));
                })
            })
            .collect();
        for h in handles {
            h.join().unwrap();
        }
    }
}
