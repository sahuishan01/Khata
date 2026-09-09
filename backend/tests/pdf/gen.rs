//! Synthetic bank/CC statement PDF generator for parser tests. No PII.
#![allow(dead_code)]
use printpdf::*;

pub struct Col {
    pub title: &'static str,
    pub x_mm: f32,
}
pub struct Row {
    pub cells: Vec<String>,
}
pub struct StatementSpec {
    pub cols: Vec<Col>,
    pub rows: Vec<Row>,
    pub repeat_header_pages: bool,
    pub preamble: Vec<String>,
}

/// Render `spec` to a single- or multi-page A4 PDF. Columns sit at their
/// `x_mm`; rows march down the page at a fixed line pitch; on page break the
/// header row is re-emitted when `repeat_header_pages` is set.
pub fn statement(spec: &StatementSpec) -> Vec<u8> {
    let (doc, page1, layer1) =
        PdfDocument::new("stmt", Mm(210.0), Mm(297.0), "l1");
    let font = doc.add_builtin_font(BuiltinFont::Helvetica).unwrap();
    let mut layer = doc.get_page(page1).get_layer(layer1);

    let top: f32 = 270.0;
    let pitch: f32 = 6.0;
    let mut y = top;

    for line in &spec.preamble {
        layer.use_text(line, 10.0, Mm(15.0), Mm(y), &font);
        y -= pitch;
    }
    y -= 4.0;

    // header row
    for c in &spec.cols {
        layer.use_text(c.title, 9.0, Mm(c.x_mm), Mm(y), &font);
    }
    y -= pitch;

    for row in &spec.rows {
        for (c, val) in spec.cols.iter().zip(&row.cells) {
            if !val.is_empty() {
                layer.use_text(val, 9.0, Mm(c.x_mm), Mm(y), &font);
            }
        }
        y -= pitch;
        if y < 20.0 {
            let (p, l) = doc.add_page(Mm(210.0), Mm(297.0), "l");
            layer = doc.get_page(p).get_layer(l);
            y = top;
            if spec.repeat_header_pages {
                for c in &spec.cols {
                    layer.use_text(c.title, 9.0, Mm(c.x_mm), Mm(y), &font);
                }
                y -= pitch;
            }
        }
    }

    doc.save_to_bytes().unwrap()
}
