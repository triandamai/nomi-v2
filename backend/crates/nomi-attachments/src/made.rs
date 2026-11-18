//! Files the crew makes: building a CSV or spreadsheet from rows, and reading the first rows
//! of one back for the chat's preview.

use std::io::{Cursor, Write};

/// Rows a preview shows when the caller doesn't say.
pub const DEFAULT_PREVIEW_ROWS: usize = 20;
/// The most rows a preview ever returns.
pub const MAX_PREVIEW_ROWS: usize = 200;

/// The first rows of a table-like file.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
pub struct RowsPreview {
    pub rows: Vec<Vec<String>>,
    /// All the rows the file holds, the first one (a header, usually) included.
    pub total_rows: usize,
    pub truncated: bool,
}

/// `rows` as CSV text (RFC 4180 quoting).
pub fn csv_bytes(rows: &[Vec<String>]) -> Vec<u8> {
    let mut out = String::new();
    for row in rows {
        let cells: Vec<String> = row
            .iter()
            .map(|cell| {
                if cell.contains([',', '"', '\n', '\r']) {
                    format!("\"{}\"", cell.replace('"', "\"\""))
                } else {
                    cell.clone()
                }
            })
            .collect();
        out.push_str(&cells.join(","));
        out.push_str("\r\n");
    }
    out.into_bytes()
}

fn xml_escape(text: &str) -> String {
    text.chars()
        .filter(|c| !c.is_control() || matches!(c, '\n' | '\t'))
        .map(|c| match c {
            '&' => "&amp;".to_string(),
            '<' => "&lt;".to_string(),
            '>' => "&gt;".to_string(),
            '"' => "&quot;".to_string(),
            c => c.to_string(),
        })
        .collect()
}

/// A1-style column letters for a zero-based column.
fn column_letters(mut index: usize) -> String {
    let mut letters = Vec::new();
    loop {
        letters.push(b'A' + (index % ALPHABET) as u8);
        if index < ALPHABET {
            break;
        }
        index = index / ALPHABET - 1;
    }
    letters.reverse();
    String::from_utf8(letters).unwrap_or_default()
}

const ALPHABET: usize = 26;

/// `rows` as a one-sheet XLSX workbook, every cell kept as text.
pub fn xlsx_bytes(rows: &[Vec<String>]) -> Result<Vec<u8>, String> {
    let mut sheet = String::from("<?xml version=\"1.0\" encoding=\"UTF-8\" standalone=\"yes\"?><worksheet xmlns=\"http://schemas.openxmlformats.org/spreadsheetml/2006/main\"><sheetData>");
    for (r, row) in rows.iter().enumerate() {
        sheet.push_str(&format!("<row r=\"{}\">", r + 1));
        for (c, cell) in row.iter().enumerate() {
            sheet.push_str(&format!("<c r=\"{}{}\" t=\"inlineStr\"><is><t xml:space=\"preserve\">{}</t></is></c>", column_letters(c), r + 1, xml_escape(cell)));
        }
        sheet.push_str("</row>");
    }
    sheet.push_str("</sheetData></worksheet>");
    let parts: [(&str, String); 5] = [
        ("[Content_Types].xml", "<?xml version=\"1.0\" encoding=\"UTF-8\"?><Types xmlns=\"http://schemas.openxmlformats.org/package/2006/content-types\"><Default Extension=\"rels\" ContentType=\"application/vnd.openxmlformats-package.relationships+xml\"/><Default Extension=\"xml\" ContentType=\"application/xml\"/><Override PartName=\"/xl/workbook.xml\" ContentType=\"application/vnd.openxmlformats-officedocument.spreadsheetml.sheet.main+xml\"/><Override PartName=\"/xl/worksheets/sheet1.xml\" ContentType=\"application/vnd.openxmlformats-officedocument.spreadsheetml.worksheet+xml\"/></Types>".to_string()),
        ("_rels/.rels", "<?xml version=\"1.0\" encoding=\"UTF-8\"?><Relationships xmlns=\"http://schemas.openxmlformats.org/package/2006/relationships\"><Relationship Id=\"rId1\" Type=\"http://schemas.openxmlformats.org/officeDocument/2006/relationships/officeDocument\" Target=\"xl/workbook.xml\"/></Relationships>".to_string()),
        ("xl/workbook.xml", "<?xml version=\"1.0\" encoding=\"UTF-8\"?><workbook xmlns=\"http://schemas.openxmlformats.org/spreadsheetml/2006/main\" xmlns:r=\"http://schemas.openxmlformats.org/officeDocument/2006/relationships\"><sheets><sheet name=\"Sheet1\" sheetId=\"1\" r:id=\"rId1\"/></sheets></workbook>".to_string()),
        ("xl/_rels/workbook.xml.rels", "<?xml version=\"1.0\" encoding=\"UTF-8\"?><Relationships xmlns=\"http://schemas.openxmlformats.org/package/2006/relationships\"><Relationship Id=\"rId1\" Type=\"http://schemas.openxmlformats.org/officeDocument/2006/relationships/worksheet\" Target=\"worksheets/sheet1.xml\"/></Relationships>".to_string()),
        ("xl/worksheets/sheet1.xml", sheet),
    ];
    let mut zip = zip::ZipWriter::new(Cursor::new(Vec::new()));
    for (name, body) in parts {
        zip.start_file(name, zip::write::SimpleFileOptions::default()).map_err(|e| e.to_string())?;
        zip.write_all(body.as_bytes()).map_err(|e| e.to_string())?;
    }
    Ok(zip.finish().map_err(|e| e.to_string())?.into_inner())
}

/// Splits CSV text into rows, keeping only the first `keep` and counting them all.
fn parse_csv(text: &str, keep: usize) -> (Vec<Vec<String>>, usize) {
    let mut kept = Vec::new();
    let mut total = 0;
    let mut row: Vec<String> = Vec::new();
    let mut cell = String::new();
    let mut quoted = false;
    let mut touched = false;
    let mut chars = text.trim_start_matches('\u{feff}').chars().peekable();
    let mut finish_row = |row: &mut Vec<String>, cell: &mut String, touched: &mut bool| {
        if *touched || !row.is_empty() {
            row.push(std::mem::take(cell));
            total += 1;
            if kept.len() < keep {
                kept.push(std::mem::take(row));
            } else {
                row.clear();
            }
        }
        *touched = false;
    };
    while let Some(c) = chars.next() {
        match (quoted, c) {
            (true, '"') if chars.peek() == Some(&'"') => {
                cell.push('"');
                chars.next();
            }
            (true, '"') => quoted = false,
            (true, c) => cell.push(c),
            (false, '"') => {
                quoted = true;
                touched = true;
            }
            (false, ',') => {
                row.push(std::mem::take(&mut cell));
                touched = true;
            }
            (false, '\r') => {}
            (false, '\n') => finish_row(&mut row, &mut cell, &mut touched),
            (false, c) => {
                cell.push(c);
                touched = true;
            }
        }
    }
    finish_row(&mut row, &mut cell, &mut touched);
    (kept, total)
}

/// The first `limit` rows of a CSV or spreadsheet; None for any other file.
pub fn rows_preview(bytes: &[u8], name: &str, limit: usize) -> Option<RowsPreview> {
    let limit = limit.min(MAX_PREVIEW_ROWS);
    let extension = crate::accepted::extension_of(name).to_lowercase();
    match extension.trim_start_matches('.') {
        "csv" => {
            let (rows, total_rows) = parse_csv(&String::from_utf8_lossy(bytes), limit);
            Some(RowsPreview { truncated: total_rows > rows.len(), rows, total_rows })
        }
        "xlsx" | "xls" | "ods" => {
            use calamine::Reader;
            let mut workbook = calamine::open_workbook_auto_from_rs(Cursor::new(bytes.to_vec())).ok()?;
            let first = workbook.sheet_names().first()?.clone();
            let range = workbook.worksheet_range(&first).ok()?;
            let total_rows = range.height();
            let rows: Vec<Vec<String>> = range.rows().take(limit).map(|r| r.iter().map(|c| c.to_string()).collect()).collect();
            Some(RowsPreview { truncated: total_rows > rows.len(), rows, total_rows })
        }
        _ => None,
    }
}

/// Where a person's stored file lives in the attachment store.
pub fn storage_key(user_id: uuid::Uuid, id: uuid::Uuid) -> String {
    format!("attachments/{user_id}/{id}")
}

/// Whether page `n` (counting from 1) exists in a file of `pages` pages.
pub fn page_in_range(n: i64, pages: i64) -> bool {
    n >= 1 && n <= pages
}

#[cfg(test)]
mod tests {
    use super::*;

    fn rows(data: &[&[&str]]) -> Vec<Vec<String>> {
        data.iter().map(|r| r.iter().map(|c| c.to_string()).collect()).collect()
    }

    #[test]
    fn csv_quotes_commas_quotes_and_newlines() {
        let out = String::from_utf8(csv_bytes(&rows(&[&["a", "b,c"], &["say \"hi\"", "x\ny"]]))).unwrap();
        assert_eq!(out, "a,\"b,c\"\r\n\"say \"\"hi\"\"\",\"x\ny\"\r\n");
    }

    #[test]
    fn a_csv_preview_reads_back_quoted_cells_and_counts_all_rows() {
        let table = rows(&[&["item", "note"], &["rice", "5,000 \"big\""], &["tea", "x\ny"], &["gas", "z"]]);
        let preview = rows_preview(&csv_bytes(&table), "spend.csv", 2).unwrap();
        assert_eq!(preview.rows, table[..2].to_vec());
        assert_eq!(preview.total_rows, 4);
        assert!(preview.truncated);
        let all = rows_preview(&csv_bytes(&table), "spend.csv", 50).unwrap();
        assert_eq!(all.rows, table);
        assert!(!all.truncated);
    }

    #[test]
    fn an_xlsx_made_here_previews_like_its_rows() {
        let table = rows(&[&["item", "amount"], &["rice", "12500"], &["tea", "3000"]]);
        let bytes = xlsx_bytes(&table).unwrap();
        let preview = rows_preview(&bytes, "spend.xlsx", 20).unwrap();
        assert_eq!(preview.total_rows, 3);
        assert_eq!(preview.rows[0], vec!["item", "amount"]);
        assert_eq!(preview.rows[1][0], "rice");
        assert_eq!(preview.rows[1][1], "12500");
    }

    #[test]
    fn only_tables_have_a_rows_preview() {
        assert!(rows_preview(b"hello", "notes.txt", 5).is_none());
    }

    #[test]
    fn pages_count_from_one_to_the_last() {
        assert!(page_in_range(1, 3) && page_in_range(3, 3));
        assert!(!page_in_range(0, 3) && !page_in_range(4, 3) && !page_in_range(-1, 3) && !page_in_range(1, 0));
    }
}
