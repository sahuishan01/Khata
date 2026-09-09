//! Diagnose why a statement PDF falls back to the legacy parser.
//!
//!     PDFIUM_LIB_PATH=backend/.pdfium/lib \
//!         cargo run --bin dump_pdf_layout -- /path/to/statement.pdf
//!
//! Prints the candidate header rows and the column bands each bank profile
//! would derive. Transaction rows are NOT printed — only the header area — so
//! the output is safe to share.
use khata::ingest::pdf::{extract, table};
use khata::ingest::profiles::registry;

fn main() {
    let path = std::env::args().nth(1).expect("usage: dump_pdf_layout <file.pdf>");
    let bytes = std::fs::read(&path).expect("read pdf");

    match extract::init(std::env::var("PDFIUM_LIB_PATH").ok().as_deref()) {
        Ok(()) => println!("pdfium: loaded"),
        Err(e) => {
            println!("pdfium: UNAVAILABLE ({e})");
            println!("-> the coordinate parser cannot run at all; set PDFIUM_LIB_PATH.");
            return;
        }
    }

    let words = match extract::words(&bytes) {
        Ok(w) => w,
        Err(e) => {
            println!("extraction failed: {e}");
            return;
        }
    };
    println!("words extracted: {}", words.len());
    if words.is_empty() {
        println!("-> no text layer (scanned image?); nothing to match a header against.");
        return;
    }

    let rows = table::rows(words);
    println!("visual rows: {}\n", rows.len());

    println!("--- first 20 rows, as the column matcher sees them ---");
    for (i, row) in rows.iter().take(20).enumerate() {
        let cells: Vec<String> = row
            .iter()
            .map(|w| format!("{}@{:.0}", w.text, w.x0))
            .collect();
        println!("row {i:02} (page {}): {}", row[0].page, cells.join(" | "));
    }

    println!("\n--- what each profile derives ---");
    let page0: Vec<Vec<_>> = rows.iter().filter(|r| r[0].page == 0).cloned().collect();
    for p in registry() {
        match table::columns(&page0, &p) {
            Some(bands) => {
                let cols: Vec<String> = bands
                    .iter()
                    .map(|b| format!("{:?}[{:.0}..{:.0}]", b.kind, b.x0.max(0.0), b.x1.min(9999.0)))
                    .collect();
                println!("{:<12} MATCHED -> {}", p.name, cols.join(" "));
            }
            None => println!("{:<12} no header row matched (needs >= 3 known column titles)", p.name),
        }
    }
}
