//! What reading a file came to, for the file's chip in chat: whether Nomi read all of it, part
//! of it, none of it for want of text, or couldn't (locked, damaged). Worked out from what
//! ingestion already knows about the file (see `extract` and the attachment worker).

use serde::Serialize;
use serde_json::Value;

use crate::extract::MAX_SHEET_ROWS;

const STATUS_PENDING: &str = "pending";
const STATUS_PROCESSING: &str = "processing";
const STATUS_FAILED: &str = "failed";

/// How a file was read.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Outcome {
    /// Read in full.
    Ok,
    /// Password-protected: nothing could be read.
    Locked,
    /// Opened, but there was no text in it for Nomi to read.
    NoText,
    /// Couldn't be opened or read.
    Damaged,
    /// Read up to a limit, the rest left out.
    Partial,
}

/// A part of a file that was left out, and why.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Skipped {
    pub name: String,
    pub reason: String,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct ReadResult {
    pub result: Outcome,
    /// How much was read, and how much there was (pages or slides when the file has them, else
    /// the rows or characters a limit cut).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub read: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub total: Option<u64>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub skipped: Vec<Skipped>,
}

impl ReadResult {
    fn of(result: Outcome) -> Self {
        ReadResult { result, read: None, total: None, skipped: Vec::new() }
    }
}

/// How a file was read, from its `status`, its extracted `text` and its `details`; None while it
/// is still being read.
pub fn read_result(status: &str, text: Option<&str>, details: &Value) -> Option<ReadResult> {
    if status == STATUS_PENDING || status == STATUS_PROCESSING {
        return None;
    }
    if details.get("encrypted").and_then(Value::as_bool) == Some(true) {
        return Some(ReadResult::of(Outcome::Locked));
    }
    let said_why = details.get("unreadable").is_some_and(Value::is_string) || details.get("error").is_some_and(Value::is_string);
    if said_why || status == STATUS_FAILED {
        return Some(ReadResult::of(Outcome::Damaged));
    }
    if text.is_none_or(|t| t.trim().is_empty()) {
        return Some(ReadResult::of(Outcome::NoText));
    }
    let skipped = skipped(details);
    if let Some((read, total)) = cut_by_limit(details) {
        return Some(ReadResult { result: Outcome::Partial, read: Some(read), total: Some(total), skipped });
    }
    let pages = ["pages", "slides"].iter().find_map(|key| details.get(*key).and_then(Value::as_u64)).filter(|n| *n > 0);
    Some(ReadResult { result: Outcome::Ok, read: pages, total: pages, skipped })
}

/// What a limit left out: the text cut at its cap, or spreadsheet rows past the rows read per
/// sheet, as (read, total).
fn cut_by_limit(details: &Value) -> Option<(u64, u64)> {
    if let Some(cut) = details.get("text_cut") {
        return Some((cut.get("read")?.as_u64()?, cut.get("total")?.as_u64()?));
    }
    let sheets = details.get("sheets")?.as_array()?;
    let rows: Vec<u64> = sheets.iter().filter_map(|sheet| sheet.get("rows")?.as_u64()).collect();
    let limit = MAX_SHEET_ROWS as u64;
    rows.iter().any(|r| *r > limit).then(|| (rows.iter().map(|r| (*r).min(limit)).sum(), rows.iter().sum()))
}

fn skipped(details: &Value) -> Vec<Skipped> {
    let Some(list) = details.get("skipped").and_then(Value::as_array) else { return Vec::new() };
    list.iter()
        .filter_map(|entry| Some(Skipped { name: entry.get("name")?.as_str()?.to_string(), reason: entry.get("reason")?.as_str()?.to_string() }))
        .collect()
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::*;
    use crate::extract::{MAX_SHEET_ROWS, MAX_TEXT_CHARS};

    fn result(status: &str, text: Option<&str>, details: Value) -> ReadResult {
        read_result(status, text, &details).expect("a file that isn't still being read")
    }

    #[test]
    fn a_file_still_being_read_has_no_result_yet() {
        assert_eq!(read_result("pending", None, &json!({})), None);
        assert_eq!(read_result("processing", Some("half"), &json!({})), None);
    }

    #[test]
    fn a_file_with_text_is_read_in_full_and_a_pdf_says_how_many_pages() {
        assert_eq!(result("ready", Some("Invoice 2291"), json!({ "lines": 1 })), ReadResult::of(Outcome::Ok));
        let pdf = result("ready", Some("Invoice"), json!({ "pages": 12, "scanned": false }));
        assert_eq!((pdf.result, pdf.read, pdf.total), (Outcome::Ok, Some(12), Some(12)));
        let slides = result("ready", Some("## Slide 1"), json!({ "slides": 8 }));
        assert_eq!((slides.read, slides.total), (Some(8), Some(8)));
    }

    #[test]
    fn a_protected_file_is_locked_even_when_nothing_else_is_known() {
        assert_eq!(result("ready", None, json!({ "encrypted": true, "pages": 0 })).result, Outcome::Locked);
        assert_eq!(result("failed", None, json!({ "encrypted": true })).result, Outcome::Locked);
    }

    #[test]
    fn a_file_that_would_not_open_or_failed_every_try_is_damaged() {
        assert_eq!(result("failed", None, json!({ "unreadable": "This PDF couldn't be opened." })).result, Outcome::Damaged);
        assert_eq!(result("ready", None, json!({ "unreadable": "This document couldn't be opened." })).result, Outcome::Damaged);
        assert_eq!(result("failed", None, json!({ "error": "it couldn't be read after several tries" })).result, Outcome::Damaged);
        assert_eq!(result("failed", None, json!({})).result, Outcome::Damaged);
    }

    #[test]
    fn a_file_that_opened_with_no_text_says_so() {
        assert_eq!(result("ready", None, json!({ "pages": 3, "scanned": true })).result, Outcome::NoText);
        assert_eq!(result("ready", Some("  \n "), json!({})).result, Outcome::NoText);
    }

    #[test]
    fn a_limit_that_cut_the_text_or_the_rows_makes_it_partial() {
        let cut = result("ready", Some("lots"), json!({ "text_cut": { "read": MAX_TEXT_CHARS, "total": MAX_TEXT_CHARS + 500 } }));
        assert_eq!((cut.result, cut.read, cut.total), (Outcome::Partial, Some(MAX_TEXT_CHARS as u64), Some(MAX_TEXT_CHARS as u64 + 500)));

        let rows = MAX_SHEET_ROWS + 100;
        let sheets = json!({ "sheets": [{ "name": "Sales", "rows": rows, "columns": 4 }, { "name": "Notes", "rows": 10, "columns": 1 }] });
        let sheet = result("ready", Some("## Sheet: Sales"), sheets);
        assert_eq!((sheet.result, sheet.read, sheet.total), (Outcome::Partial, Some(MAX_SHEET_ROWS as u64 + 10), Some(rows as u64 + 10)));

        let short = json!({ "sheets": [{ "name": "Sales", "rows": 20, "columns": 4 }] });
        assert_eq!(result("ready", Some("## Sheet: Sales"), short).result, Outcome::Ok);
    }

    #[test]
    fn parts_left_out_are_passed_on_with_their_reasons() {
        let skipped = json!({ "skipped": [{ "name": "disk.img", "reason": "too_large" }, { "bad": 1 }] });
        let out = result("ready", Some("x"), skipped);
        assert_eq!(out.skipped, vec![Skipped { name: "disk.img".into(), reason: "too_large".into() }]);
        let wire = serde_json::to_value(&out).unwrap();
        assert_eq!(wire["result"], "ok");
        assert!(wire.get("read").is_none());
    }
}
