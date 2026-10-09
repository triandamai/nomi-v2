//! Reads what the server can read itself, right after upload: the text of text files, PDFs,
//! Word / OpenDocument documents, spreadsheets and slides, and a small JPEG of an image (what the
//! chat shows and what models are sent). Images, audio, video and scanned PDFs are left for a
//! model (`needs_model`). Blocking: run it off the async runtime.

use std::io::{Cursor, Read};

use serde_json::{json, Value};

use crate::{Kind, Sniffed};

/// Text kept per file. Beyond this an agent is told the rest was cut.
pub const MAX_TEXT_CHARS: usize = 2_000_000;
/// Longest side of an image preview, in pixels (what Claude and GPT read at full detail).
pub const PREVIEW_MAX_SIDE: u32 = 1568;
/// Rows read from each sheet of a spreadsheet.
const MAX_SHEET_ROWS: usize = 5_000;

#[derive(Debug, Clone, Default, PartialEq)]
pub struct Extracted {
    pub text: Option<String>,
    /// Pages, sheets, slides, width and height, or why it couldn't be read.
    pub details: Value,
    pub preview_jpeg: Option<Vec<u8>>,
    /// Only a model can read it (an image, audio, video, a scanned PDF).
    pub needs_model: bool,
}

pub fn extract(bytes: &[u8], sniffed: &Sniffed) -> Extracted {
    let result = match sniffed.kind {
        Kind::Text => Ok(text_file(bytes)),
        Kind::Pdf => pdf(bytes),
        Kind::Document => document(bytes, &sniffed.mime),
        Kind::Spreadsheet => spreadsheet(bytes),
        Kind::Presentation => presentation(bytes, &sniffed.mime),
        Kind::Image => Ok(image_preview(bytes)),
        Kind::Audio | Kind::Voice | Kind::Video => Ok(Extracted { needs_model: true, details: json!({}), ..Default::default() }),
        Kind::Other => Ok(Extracted { details: json!({ "unreadable": "Nomi can't read this kind of file." }), ..Default::default() }),
    };
    match result {
        Ok(mut extracted) => {
            extracted.text = extracted.text.map(cap);
            if extracted.details.is_null() {
                extracted.details = json!({});
            }
            extracted
        }
        Err(reason) => {
            tracing::info!(mime = %sniffed.mime, reason = %reason, "couldn't read attachment");
            Extracted { details: json!({ "unreadable": reason }), ..Default::default() }
        }
    }
}

fn cap(text: String) -> String {
    let text = text.trim().to_string();
    match text.char_indices().nth(MAX_TEXT_CHARS) {
        Some((cut, _)) => text[..cut].to_string(),
        None => text,
    }
}

fn text_file(bytes: &[u8]) -> Extracted {
    let text = String::from_utf8_lossy(bytes).replace("\r\n", "\n");
    let lines = text.lines().count();
    Extracted { text: Some(text), details: json!({ "lines": lines }), ..Default::default() }
}

fn pdf(bytes: &[u8]) -> Result<Extracted, String> {
    let pages = lopdf::Document::load_mem(bytes).map(|d| d.get_pages().len()).unwrap_or(0);
    // pdf-extract panics on some malformed files; a bad PDF must never take the server down.
    let text = std::panic::catch_unwind(|| pdf_extract::extract_text_from_mem(bytes))
        .map_err(|_| "This PDF couldn't be opened.".to_string())?
        .unwrap_or_default();
    let text = collapse_blank_lines(&text);
    let letters = text.chars().filter(|c| c.is_alphanumeric()).count();
    // Scanned pages have next to no text layer: a model has to look at them.
    let scanned = letters < 40 * pages.max(1);
    Ok(Extracted {
        text: if letters == 0 { None } else { Some(text) },
        details: json!({ "pages": pages, "scanned": scanned }),
        needs_model: scanned,
        ..Default::default()
    })
}

fn collapse_blank_lines(text: &str) -> String {
    let mut out = String::with_capacity(text.len());
    let mut blank = 0;
    for line in text.lines() {
        let line = line.trim_end();
        if line.trim().is_empty() {
            blank += 1;
            if blank > 1 {
                continue;
            }
        } else {
            blank = 0;
        }
        out.push_str(line);
        out.push('\n');
    }
    out
}

fn zip_entry(archive: &mut zip::ZipArchive<Cursor<&[u8]>>, name: &str) -> Option<String> {
    let mut file = archive.by_name(name).ok()?;
    let mut xml = String::new();
    file.read_to_string(&mut xml).ok()?;
    Some(xml)
}

/// The text of an Office or OpenDocument XML part: `paragraph` elements end a line, `tab`
/// elements become tabs, `cell` elements are separated by " | ".
fn xml_text(xml: &str, paragraph: &[&[u8]], tab: &[u8], cell: &[u8]) -> String {
    use quick_xml::events::Event;
    let mut reader = quick_xml::Reader::from_str(xml);
    let mut out = String::new();
    loop {
        match reader.read_event() {
            Ok(Event::Text(t)) => {
                if let Ok(text) = t.unescape() {
                    out.push_str(&text);
                }
            }
            Ok(Event::Empty(e)) => {
                let name = e.local_name();
                if name.as_ref() == tab {
                    out.push('\t');
                } else if name.as_ref() == b"br" || name.as_ref() == b"line-break" {
                    out.push('\n');
                }
            }
            Ok(Event::End(e)) => {
                let name = e.local_name();
                if paragraph.contains(&name.as_ref()) {
                    out.push('\n');
                } else if name.as_ref() == cell {
                    out.push_str(" | ");
                }
            }
            Ok(Event::Eof) | Err(_) => break,
            _ => {}
        }
    }
    collapse_blank_lines(&out)
}

fn document(bytes: &[u8], mime: &str) -> Result<Extracted, String> {
    match mime {
        "application/msword" => Err("Old Word files (.doc) can't be read. Save it as .docx or PDF and attach it again.".to_string()),
        "application/rtf" => Ok(text_file(&strip_rtf(bytes))),
        _ => {
            let mut archive = zip::ZipArchive::new(Cursor::new(bytes)).map_err(|_| "This document couldn't be opened.".to_string())?;
            let text = if let Some(xml) = zip_entry(&mut archive, "word/document.xml") {
                xml_text(&xml, &[b"p"], b"tab", b"tc")
            } else if let Some(xml) = zip_entry(&mut archive, "content.xml") {
                xml_text(&xml, &[b"p", b"h"], b"tab", b"table-cell")
            } else {
                return Err("This document couldn't be opened.".to_string());
            };
            let words = text.split_whitespace().count();
            Ok(Extracted { text: Some(text), details: json!({ "words": words }), ..Default::default() })
        }
    }
}

/// RTF's text without its control words and groups: rough, but readable.
fn strip_rtf(bytes: &[u8]) -> Vec<u8> {
    let source = String::from_utf8_lossy(bytes);
    let mut out = String::new();
    let mut chars = source.chars().peekable();
    while let Some(c) = chars.next() {
        match c {
            '{' | '}' => {}
            '\\' => {
                let mut word = String::new();
                while let Some(&n) = chars.peek() {
                    if n.is_ascii_alphabetic() {
                        word.push(n);
                        chars.next();
                    } else {
                        break;
                    }
                }
                while let Some(&n) = chars.peek() {
                    if n.is_ascii_digit() || n == '-' {
                        chars.next();
                    } else {
                        break;
                    }
                }
                if chars.peek() == Some(&' ') {
                    chars.next();
                }
                match word.as_str() {
                    "par" | "line" => out.push('\n'),
                    "tab" => out.push('\t'),
                    "" => {
                        if let Some(n) = chars.next() {
                            if matches!(n, '\\' | '{' | '}') {
                                out.push(n);
                            }
                        }
                    }
                    _ => {}
                }
            }
            '\r' | '\n' => {}
            c => out.push(c),
        }
    }
    out.into_bytes()
}

fn presentation(bytes: &[u8], mime: &str) -> Result<Extracted, String> {
    if mime == "application/vnd.ms-powerpoint" {
        return Err("Old PowerPoint files (.ppt) can't be read. Save it as .pptx or PDF and attach it again.".to_string());
    }
    let mut archive = zip::ZipArchive::new(Cursor::new(bytes)).map_err(|_| "This presentation couldn't be opened.".to_string())?;
    if let Some(xml) = zip_entry(&mut archive, "content.xml") {
        let text = xml_text(&xml, &[b"p", b"h"], b"tab", b"table-cell");
        return Ok(Extracted { text: Some(text), details: json!({}), ..Default::default() });
    }
    let mut slides: Vec<(usize, String)> = archive
        .file_names()
        .filter_map(|n| {
            let number = n.strip_prefix("ppt/slides/slide")?.strip_suffix(".xml")?.parse().ok()?;
            Some((number, n.to_string()))
        })
        .collect();
    slides.sort();
    let mut text = String::new();
    for (number, name) in &slides {
        if let Some(xml) = zip_entry(&mut archive, name) {
            text.push_str(&format!("## Slide {number}\n"));
            text.push_str(&xml_text(&xml, &[b"p"], b"tab", b"tc"));
            text.push('\n');
        }
    }
    Ok(Extracted { text: Some(text), details: json!({ "slides": slides.len() }), ..Default::default() })
}

fn spreadsheet(bytes: &[u8]) -> Result<Extracted, String> {
    use calamine::Reader;
    let mut workbook = calamine::open_workbook_auto_from_rs(Cursor::new(bytes.to_vec())).map_err(|_| "This spreadsheet couldn't be opened.".to_string())?;
    let mut text = String::new();
    let mut sheets = Vec::new();
    for name in workbook.sheet_names() {
        let Ok(range) = workbook.worksheet_range(&name) else { continue };
        let rows = range.height();
        sheets.push(json!({ "name": name, "rows": rows, "columns": range.width() }));
        text.push_str(&format!("## Sheet: {name}\n"));
        for row in range.rows().take(MAX_SHEET_ROWS) {
            let cells: Vec<String> = row.iter().map(|c| c.to_string().replace(['\t', '\n'], " ")).collect();
            if cells.iter().all(|c| c.is_empty()) {
                continue;
            }
            text.push_str(&cells.join("\t"));
            text.push('\n');
        }
        if rows > MAX_SHEET_ROWS {
            text.push_str(&format!("({} more rows not shown)\n", rows - MAX_SHEET_ROWS));
        }
        text.push('\n');
    }
    Ok(Extracted { text: Some(text), details: json!({ "sheets": sheets }), ..Default::default() })
}

fn image_preview(bytes: &[u8]) -> Extracted {
    let decoded = match image::load_from_memory(bytes) {
        Ok(image) => image,
        // HEIC and other formats this server can't decode: a model still can.
        Err(_) => return Extracted { details: json!({}), needs_model: true, ..Default::default() },
    };
    let (width, height) = (decoded.width(), decoded.height());
    let resized = if width.max(height) > PREVIEW_MAX_SIDE { decoded.thumbnail(PREVIEW_MAX_SIDE, PREVIEW_MAX_SIDE) } else { decoded };
    let mut jpeg = Vec::new();
    let encoded = image::codecs::jpeg::JpegEncoder::new_with_quality(&mut jpeg, 82).encode_image(&resized.to_rgb8());
    Extracted {
        details: json!({ "width": width, "height": height }),
        preview_jpeg: encoded.ok().map(|_| jpeg),
        needs_model: true,
        ..Default::default()
    }
}

#[cfg(test)]
mod tests {
    use std::io::Write;

    use super::*;

    fn sniffed(kind: Kind, mime: &str) -> Sniffed {
        Sniffed { kind, mime: mime.to_string() }
    }

    fn zip_with(files: &[(&str, &str)]) -> Vec<u8> {
        let mut buffer = Cursor::new(Vec::new());
        {
            let mut writer = zip::ZipWriter::new(&mut buffer);
            for (name, content) in files {
                writer.start_file(*name, zip::write::SimpleFileOptions::default()).unwrap();
                writer.write_all(content.as_bytes()).unwrap();
            }
            writer.finish().unwrap();
        }
        buffer.into_inner()
    }

    #[test]
    fn reads_a_word_document() {
        let docx = zip_with(&[(
            "word/document.xml",
            r#"<w:document xmlns:w="w"><w:body><w:p><w:r><w:t>Trip to Bali</w:t></w:r></w:p><w:p><w:r><w:t>Day 1:</w:t><w:tab/><w:t>Ubud &amp; rice fields</w:t></w:r></w:p></w:body></w:document>"#,
        )]);
        let out = extract(&docx, &sniffed(Kind::Document, "application/vnd.openxmlformats-officedocument.wordprocessingml.document"));
        assert_eq!(out.text.as_deref(), Some("Trip to Bali\nDay 1:\tUbud & rice fields"));
        assert!(!out.needs_model);
    }

    #[test]
    fn reads_slides_in_order() {
        let pptx = zip_with(&[
            ("ppt/slides/slide10.xml", r#"<p:sld xmlns:a="a" xmlns:p="p"><a:p><a:r><a:t>Last</a:t></a:r></a:p></p:sld>"#),
            ("ppt/slides/slide2.xml", r#"<p:sld xmlns:a="a" xmlns:p="p"><a:p><a:r><a:t>Second</a:t></a:r></a:p></p:sld>"#),
        ]);
        let out = extract(&pptx, &sniffed(Kind::Presentation, "application/vnd.openxmlformats-officedocument.presentationml.presentation"));
        let text = out.text.unwrap();
        assert!(text.find("Second").unwrap() < text.find("Last").unwrap(), "{text}");
        assert_eq!(out.details["slides"], 2);
    }

    #[test]
    fn makes_a_small_jpeg_of_a_big_image() {
        let image = image::RgbImage::from_pixel(3000, 1000, image::Rgb([200, 30, 30]));
        let mut png = Vec::new();
        image::DynamicImage::ImageRgb8(image).write_to(&mut Cursor::new(&mut png), image::ImageFormat::Png).unwrap();
        let out = extract(&png, &sniffed(Kind::Image, "image/png"));
        assert_eq!(out.details["width"], 3000);
        let preview = image::load_from_memory(&out.preview_jpeg.unwrap()).unwrap();
        assert_eq!((preview.width(), preview.height()), (1568, 523));
        assert!(out.needs_model);
    }

    #[test]
    fn says_why_an_old_word_file_cant_be_read() {
        let out = extract(b"\xD0\xCF\x11\xE0", &sniffed(Kind::Document, "application/msword"));
        assert!(out.details["unreadable"].as_str().unwrap().contains(".docx"));
        assert_eq!(out.text, None);
    }

    /// A one-page PDF with a text layer, built the way real PDFs are (with a cross-reference table).
    fn pdf_saying(line: &str) -> Vec<u8> {
        use lopdf::{dictionary, Object, Stream};
        let mut doc = lopdf::Document::with_version("1.5");
        let pages_id = doc.new_object_id();
        let font_id = doc.add_object(dictionary! { "Type" => "Font", "Subtype" => "Type1", "BaseFont" => "Helvetica" });
        let content = format!("BT /F1 14 Tf 72 700 Td ({line}) Tj ET");
        let content_id = doc.add_object(Stream::new(dictionary! {}, content.into_bytes()));
        let page_id = doc.add_object(dictionary! {
            "Type" => "Page",
            "Parent" => pages_id,
            "Contents" => content_id,
            "Resources" => dictionary! { "Font" => dictionary! { "F1" => font_id } },
            "MediaBox" => vec![0.into(), 0.into(), 612.into(), 792.into()],
        });
        doc.objects.insert(pages_id, Object::Dictionary(dictionary! { "Type" => "Pages", "Kids" => vec![page_id.into()], "Count" => 1 }));
        let catalog_id = doc.add_object(dictionary! { "Type" => "Catalog", "Pages" => pages_id });
        doc.trailer.set("Root", catalog_id);
        let mut out = Vec::new();
        doc.save_to(&mut out).unwrap();
        out
    }

    #[test]
    fn reads_a_pdfs_text_layer() {
        let pdf = pdf_saying("Invoice 2291: web hosting for October, total Rp 450.000, due 20 October 2026.");
        let out = extract(&pdf, &sniffed(Kind::Pdf, "application/pdf"));
        assert!(out.text.as_deref().unwrap_or("").contains("Invoice 2291"), "{out:?}");
        assert_eq!(out.details["pages"], 1);
        assert!(!out.needs_model, "a PDF with text doesn't need a model to read it");
    }

    #[test]
    fn a_broken_pdf_is_unreadable_not_a_crash() {
        let out = extract(b"%PDF-1.4 not really", &sniffed(Kind::Pdf, "application/pdf"));
        assert!(out.text.is_none());
    }

    #[test]
    fn reads_rtf_roughly() {
        let out = extract(br"{\rtf1\ansi{\fonttbl\f0 Arial;}\f0 Hello\par World}", &sniffed(Kind::Document, "application/rtf"));
        assert!(out.text.unwrap().contains("Hello\nWorld"));
    }
}
