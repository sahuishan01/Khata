use calamine::{open_workbook_auto, Data, Reader};
use std::env;

fn main() {
    let path = env::args().nth(1).expect("usage: dump_statement <file>");
    let mut wb = open_workbook_auto(&path).expect("open workbook");
    let sheet = wb.sheet_names()[0].clone();
    println!("Sheet: {sheet}");
    let range = wb.worksheet_range(&sheet).expect("read sheet");
    for (i, row) in range.rows().take(15).enumerate() {
        let cells: Vec<String> = row.iter().map(|c| match c {
            Data::String(s) => format!("\"{}\"", s.trim()),
            Data::Float(f) => format!("{f}"),
            Data::Int(n) => format!("{n}"),
            Data::Empty => "_".to_string(),
            _ => format!("{c:?}"),
        }).collect();
        println!("row {i:02}: {}", cells.join(" | "));
    }
}
